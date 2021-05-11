use std::{
    str::FromStr,
    time::Duration,
};
use redis::{
    self,
    AsyncCommands,
    RedisResult,
    aio::Connection,
};
use simple_logger::SimpleLogger;
use tokio::time::sleep;
use warp::{
    Filter,
    Reply,
    Rejection,
};
use bitcoincore_rpc::{
    Auth, 
    Client as RpcClient, 
    RpcApi,
    bitcoin::{
        Address as RpcAddress,
        Amount,
    },
};
use tglib::{
    bdk::{
        bitcoin::{
            Address,
            PublicKey,
            Script,
            Txid,
            blockdata::transaction::OutPoint,
            consensus,
            hashes::hex::ToHex,
            util::{
                bip32::{
                    ExtendedPubKey,
                    Fingerprint,
                },
                psbt::{
                    Input,
                    PartiallySignedTransaction,
                },
            },
            secp256k1::{
                Message,
                Secp256k1,
                Signature,
            }
        },
        blockchain::ElectrumBlockchain,
        database::MemoryDatabase,
        electrum_client::{
            Client,
            ElectrumApi,
            ListUnspentRes,
        }
    },
    hex,
    log::{
        error,
        debug,
        LevelFilter,
    },
    rand::{self, Rng},
//    Result,
    Error,
    arbiter::{
        AuthTokenSig,
        SendContractBody,
        SendPayoutBody,
        SetContractInfoBody,
        SubmitContractBody,
        SubmitPayoutBody,
    },
    contract::{
        Contract,
        PlayerContractInfo,
    },
    payout::Payout,
    player::PlayerName,
    wallet::{
        get_namecoin_address,
        EscrowWallet,
    },
    mock::{                  
        ARBITER_FINGERPRINT,
        ARBITER_XPUBKEY,
        ELECTRS_SERVER,
        NETWORK,             
        REDIS_SERVER,
    },
};
mod wallet;
use wallet::Wallet;

const BITCOIN_RPC_URL: &'static str = "http://electrs:18443";
const NAME_SERVICE_URL: &'static str = "http://nmc-id:18420";
const AUTH_TOKEN_LIFETIME: usize = 30;
type Result<T> = std::result::Result<T, Box<dyn std::error::Error>>;
type WebResult<T> = std::result::Result<T, Rejection>;

fn wallet() -> Wallet<ElectrumBlockchain, MemoryDatabase> {
    let mut client = Client::new(ELECTRS_SERVER);
    while client.is_err() {
        std::thread::sleep(Duration::from_secs(1));
        client = Client::new(ELECTRS_SERVER);
    }
//TODO: should this database be persisted?
    Wallet::<ElectrumBlockchain, MemoryDatabase>::new(Fingerprint::from_str(ARBITER_FINGERPRINT).unwrap(), ExtendedPubKey::from_str(ARBITER_XPUBKEY).unwrap(), ElectrumBlockchain::from(client.unwrap()), NETWORK)
}

fn get_escrow_pubkey() -> PublicKey {
    EscrowWallet::get_escrow_pubkey(&wallet())
}

// TODO: more functions like these for the other redis ops
async fn push_contract(con: &mut Connection, hex: &str) -> RedisResult<String> {
    con.rpush("contracts", hex).await?;
    Ok(String::from(hex))
}

async fn push_payout(con: &mut Connection, hex: &str) -> RedisResult<String> {
    con.rpush("payouts", hex).await?;
    Ok(String::from(hex))
}

async fn controls_name(pubkey: &PublicKey, player_name: &PlayerName) -> reqwest::Result<bool> {
    match reqwest::get(&format!("{}/{}/{}", NAME_SERVICE_URL, "get-name-address", hex::encode(player_name.0.as_bytes()))).await {
        Ok(response) => match response.text().await {
            Ok(name_address) => Ok(get_namecoin_address(pubkey, NETWORK) == name_address), 
            Err(e) => {
                error!("{:?}", e);
                Err(e)
            },
        },
        Err(e) => {
            error!("{:?}", e);
            Err(e)
        }
    }
}

async fn check_auth_token_sig(player_name: PlayerName, auth: AuthTokenSig, con: &mut Connection) -> Result<bool> {

    match controls_name(&auth.pubkey, &player_name).await {
        Ok(true) => (),
        _ => return Ok(false),
    }
    let secp = Secp256k1::new();
    let sig = Signature::from_der(&hex::decode(&auth.sig_hex).unwrap()).unwrap();
    let r: RedisResult<String> = con.get(format!("{}/token", player_name)).await;
    let token = match r {
        Ok(token) => token,
        Err(_) => return Ok(false),
    };
    Ok(secp.verify(&Message::from_slice(&hex::decode(token).unwrap()).unwrap(), &sig, &auth.pubkey.key).is_ok())
}

async fn submit_contract(con: &mut Connection, contract: &Contract) -> Result<Signature> {
    wallet().validate_contract(&contract)?;

    let _r = push_contract(con, &hex::encode(contract.to_bytes())).await.unwrap();
    let cxid = hex::encode(contract.cxid());
    for _ in 1..15 as u32 {
        sleep(Duration::from_secs(1)).await;
        let r: RedisResult<String> = con.get(cxid.clone()).await;
        if let Ok(sig_hex) = r {
            let _r : RedisResult<u64> = con.del(cxid).await;
            return Ok(Signature::from_der(&hex::decode(sig_hex).unwrap()).unwrap())
        }
    }
    let e = Error::InvalidContract("arbiter rejected contract");
    error!("{:?}", e);
    Err(Box::new(e))
}

async fn submit_payout(con: &mut Connection, payout: &Payout) -> Result<PartiallySignedTransaction> {
    wallet().validate_payout(&payout)?;
    let _r = push_payout(con, &hex::encode(payout.to_bytes())).await.unwrap();
    let cxid = hex::encode(payout.contract.cxid());
    for _ in 1..15 as u32 {
        sleep(Duration::from_secs(1)).await;
        let r: RedisResult<String> = con.get(cxid.clone()).await;
        if let Ok(tx) = r {
            let _r : RedisResult<u64> = con.del(cxid).await;
            return Ok(consensus::deserialize::<PartiallySignedTransaction>(&hex::decode(tx).unwrap()).unwrap())
        }
    }
    let e = Error::InvalidPayout("arbiter rejected payout");
    error!("{:?}", e);
    Err(Box::new(e))
}

//TODO: this function needs to reply more clearly when it fails
async fn set_contract_info_handler(body: SetContractInfoBody, redis_client: redis::Client) -> WebResult<impl Reply> {
//    controls_name(&body.pubkey, &body.contract_info.name).await?;
    match controls_name(&body.pubkey, &body.contract_info.name).await {
        Ok(true) => (),
        Ok(false) => return Err(warp::reject()),
        Err(e) => {
            error!("{:?}", e);
            return Err(warp::reject())
        }
    }
 
    let secp = Secp256k1::new();
    let sig = Signature::from_der(&hex::decode(&body.sig_hex).unwrap()).unwrap();
    if secp.verify(&Message::from_slice(&body.contract_info.hash()).unwrap(), &sig, &body.pubkey.key).is_err() {
// TODO: responses with proper errors and data
        return Err(warp::reject())
    }

    let mut con = redis_client.get_async_connection().await.unwrap();
    let r: RedisResult<String> = con.set(format!("{}/info", body.contract_info.name.clone().0), &serde_json::to_string(&body.contract_info).unwrap()).await;
    match r {
        Ok(_string) => {
            Ok(format!("set contract info for {}", body.contract_info.name.0))
        },
        Err(e) => {
            error!("{:?}", e);
            Err(warp::reject())
        }
    }
}

async fn get_contract_info_handler(player_name: String, redis_client: redis::Client) -> WebResult<impl Reply> {
    let mut con = redis_client.get_async_connection().await.unwrap();
    let r: RedisResult<String> = con.get(format!("{}/info", &String::from_utf8(hex::decode(player_name).unwrap()).unwrap())).await;
    match r {
        Ok(info) => Ok(info),
        Err(e) => {
            error!("{:?}", e);
            Err(warp::reject())
        }
    }
}

async fn send_contract_handler(body: SendContractBody, redis_client: redis::Client) -> WebResult<impl Reply> {
    let mut con = redis_client.get_async_connection().await.unwrap();
    let r: RedisResult<i64> = con.rpush(&format!("{}/contracts", body.player_name), serde_json::to_string(&body.contract).unwrap()).await;
    match r {
        Ok(_num) => {
            Ok("sent contract".to_string())
        }
        Err(e) => {
            error!("send contract redis error: {:?}", e);
            Err(warp::reject())
        }
    }
}

async fn receive_contract_handler(auth: AuthTokenSig, redis_client: redis::Client) -> WebResult<impl Reply> {
    let mut con = redis_client.get_async_connection().await.unwrap();
    match check_auth_token_sig(auth.player_name.clone(), auth.clone(), &mut con).await {
        Ok(true) => (),
        _ => return Err(warp::reject()),
    }
    let r: RedisResult<String> = con.lpop(&format!("{}/contracts", auth.player_name.0)).await;
    match r {
        Ok(contract_json) => Ok(contract_json),
        Err(e) => {
            error!("check auth token failed: {:?}", e);
            Err(warp::reject())
        }
    }
}

async fn send_payout_handler(body: SendPayoutBody, redis_client: redis::Client) -> WebResult<impl Reply> {
    let mut con = redis_client.get_async_connection().await.unwrap();
    let r: RedisResult<i64> = con.rpush(&format!("{}/payouts", body.player_name.0), serde_json::to_string(&body.payout).unwrap()).await;
    match r {
        Ok(_string) => Ok("sent payout".to_string()),
        Err(e) => {
            error!("send payout redis error {:?}", e);
            Err(warp::reject())
        }
    }
}

async fn receive_payout_handler(auth: AuthTokenSig, redis_client: redis::Client) -> WebResult<impl Reply> {
    let mut con = redis_client.get_async_connection().await.unwrap();
    match check_auth_token_sig(auth.player_name.clone(), auth.clone(), &mut con).await {
        Ok(true) => (),
        Ok(false) => {
            error!("check auth token failed: invalid credentials");
            return Err(warp::reject())
        },
        Err(e) => {
            error!("check auth token failed: {:?}", e);
            return Err(warp::reject())
        }
    }

    let r: RedisResult<String> = con.lpop(&format!("{}/payouts", auth.player_name.0)).await;
    match r {
        Ok(payout_json) => Ok(payout_json),
        Err(e) => {
            error!("couldn't pop from payout list: {:?}", e);
            Err(warp::reject())
        }
    }
}

async fn submit_contract_handler(body: SubmitContractBody, redis_client: redis::Client) -> WebResult<impl Reply> {
    let mut con = redis_client.get_async_connection().await.unwrap();
    let contract = Contract::from_bytes(hex::decode(body.contract_hex).unwrap()).unwrap();
    match submit_contract(&mut con, &contract).await {
        Ok(sig) => Ok(hex::encode(sig.serialize_der())),
        Err(e) => {
            error!("{:?}", e);
            Err(warp::reject())
        }
    }
}

// TODO: somehow break out of serialize(decode( hell
async fn submit_payout_handler(body: SubmitPayoutBody, redis_client: redis::Client) -> WebResult<impl Reply> {
    let mut con = redis_client.get_async_connection().await.unwrap();
    let payout = Payout::from_bytes(hex::decode(body.payout_hex).unwrap()).unwrap();
    match submit_payout(&mut con, &payout).await {
        Ok(tx) => Ok(hex::encode(consensus::serialize(&tx))),
        Err(e) => {
            error!("{:?}", e);
            Err(warp::reject())
        }
    }
}

async fn fund_address_handler(address: String) -> WebResult<impl Reply> { 
    let address = RpcAddress::from_str(&address).unwrap();
    let bitcoin_rpc_client = RpcClient::new(BITCOIN_RPC_URL.to_string(), Auth::UserPass("admin".to_string(), "passw".to_string())).unwrap();
    let coinbase_addr = bitcoin_rpc_client.get_new_address(None, None).unwrap();
    let txid = bitcoin_rpc_client.send_to_address(&address, Amount::ONE_BTC, None, None, None, None, None, None).unwrap();
    let _blockhashes = bitcoin_rpc_client.generate_to_address(150, &coinbase_addr).unwrap();
    Ok(txid.to_hex())
}

async fn auth_token_handler(player_name: String, redis_client: redis::Client) -> WebResult<impl Reply> {
    let token = hex::encode(rand::thread_rng().gen::<[u8; 32]>().to_vec());
    let mut con = redis_client.get_async_connection().await.unwrap();
// TODO: should we delete the token after it gets used or let them use it until it expires?
    let r: RedisResult<String> = con.set_ex(format!("{}/token", player_name), token.clone(), AUTH_TOKEN_LIFETIME).await;
    match r {
        Ok(_) => Ok(token),
        Err(e) => {
            error!("{:?}", e);
            Err(warp::reject())
        }
    }
}

async fn remove_stale_contract_info() -> Result<()> {
//TODO: try using script pubkey subscription instead
    let redis_client = redis_client();
    let electrum_client = Client::new(ELECTRS_SERVER)?;
    let mut con = redis_client.get_async_connection().await.unwrap();
    loop {
        sleep(Duration::from_secs(10)).await;
        let info_keys: Vec<String>  = con.keys("*/info").await?;
        for key in info_keys {
            let info: String = con.get(&key).await?;
            let info: PlayerContractInfo = serde_json::from_str(&info)?;
            let txids: Vec<Txid> = info.utxos.iter().map(|utxo| utxo.0.txid).collect();
            debug!("posted utxos: {}", info.utxos.len());
            let txs = electrum_client.batch_transaction_get(&txids)?;
            let utxo_scripts: Vec<Script> = txs
                .iter()
                .flat_map(|tx| {
                    tx.output
                    .iter()
                    .zip(vec![tx; tx.output.len()])
                    .enumerate()
                    .filter_map(|(vout, (output, tx))| {
                        if info.utxos.iter().find(|utxo| {
                            utxo.0.txid == tx.txid() &&
                            utxo.0.vout as usize == vout}).is_some() {
                            Some(output.clone().script_pubkey)
                        } else { 
                            None 
                        }
                    })
                })
                .collect();
            let unspent_utxos = electrum_client.batch_script_list_unspent(&utxo_scripts)?;
            let unspent_utxos: Vec<&ListUnspentRes> = unspent_utxos.iter().flatten().collect();

            debug!("unspent utxos matching posted utxos: {}", unspent_utxos.len());

            let new_info_utxos: Vec<(OutPoint, u64, Input)> = info.utxos.iter().filter(|(outpoint, amount, _)| 
                    unspent_utxos.iter().find(|list_unspent_res| {
                        list_unspent_res.tx_hash == outpoint.txid && 
                        list_unspent_res.tx_pos == outpoint.vout as usize &&
                        &list_unspent_res.value == amount
                    }
                ).is_some()
            ).cloned().collect();
            
            if !new_info_utxos.is_empty() {
                let new_info = PlayerContractInfo {
                    utxos: new_info_utxos,
                    ..info
                };
                con.set(key, serde_json::to_string(&new_info)?).await?;
            } else {
                con.del(key).await?
            }
        }
    }
}

fn redis_client() -> redis::Client {
    let mut client = redis::Client::open(REDIS_SERVER);
    while client.is_err() {
        std::thread::sleep(Duration::from_millis(100));
        client = redis::Client::open(REDIS_SERVER);
    }
    client.unwrap()
}

#[tokio::main]
async fn main() {

    SimpleLogger::new()
        .with_level(LevelFilter::Debug)
        .with_module_level("warp",LevelFilter::Warn)
        .with_module_level("hyper",LevelFilter::Warn)
        .with_module_level("reqwest",LevelFilter::Warn)
        .init()
        .unwrap();
    
    let escrow_pubkey = get_escrow_pubkey();
    let escrow_pubkey = warp::any().map(move || escrow_pubkey.clone());
    let fee_address = wallet().get_fee_address();
    let fee_address = warp::any().map(move || fee_address.clone());
    let redis_client = redis_client();
    let redis_client = warp::any().map(move || redis_client.clone());

    let get_escrow_pubkey = warp::path("escrow-pubkey")
        .and(escrow_pubkey)
        .map(|e: PublicKey | e.to_string()); 

    let get_fee_address = warp::path("fee-address")
        .and(fee_address)
        .map(|f: Address| f.to_string() ); 

    let set_contract_info = warp::path("set-contract-info")
        .and(warp::post())
        .and(warp::body::json())
        .and(redis_client.clone())
        .and_then(set_contract_info_handler);

    let get_contract_info = warp::path("get-contract-info")
        .and(warp::path::param::<String>())
        .and(redis_client.clone())
        .and_then(get_contract_info_handler);

    let send_contract = warp::path("send-contract")
        .and(warp::post())
        .and(warp::body::json())
        .and(redis_client.clone())
        .and_then(send_contract_handler);

    let receive_contract = warp::path("receive-contract")
        .and(warp::post())
        .and(warp::body::json())
        .and(redis_client.clone())
        .and_then(receive_contract_handler);

    let send_payout = warp::path("send-payout")
        .and(warp::post())
        .and(warp::body::json())
        .and(redis_client.clone())
        .and_then(send_payout_handler);

    let receive_payout = warp::path("receive-payout")
        .and(warp::post())
        .and(warp::body::json())
        .and(redis_client.clone())
        .and_then(receive_payout_handler);

    let submit_contract = warp::path("submit-contract")
        .and(warp::post())
        .and(warp::body::json())
        .and(redis_client.clone())
        .and_then(submit_contract_handler);

    let submit_payout = warp::path("submit-payout")
        .and(warp::post())
        .and(warp::body::json())
        .and(redis_client.clone())
        .and_then(submit_payout_handler);

// TODO: can add validation filters for path params?
    let fund_address = warp::path("fund-address")
        .and(warp::path::param::<String>())
        .and_then(fund_address_handler);

    let auth_token = warp::path("auth-token")
        .and(warp::path::param::<String>())
        .and(redis_client.clone())
        .and_then(auth_token_handler);

    let routes = get_escrow_pubkey
        .or(get_fee_address)
        .or(set_contract_info)
        .or(get_contract_info)
        .or(send_contract)
        .or(receive_contract)
        .or(send_payout)
        .or(receive_payout)
        .or(submit_contract)
        .or(submit_payout)
        .or(fund_address)
        .or(auth_token);
// TODO: add task to purge stale posted contract info
// e.g. when a posted utxo is spent
    let _ = tokio::join!(
        warp::serve(routes).run(([0, 0, 0, 0], 5000)),
        remove_stale_contract_info(),
    );
}
