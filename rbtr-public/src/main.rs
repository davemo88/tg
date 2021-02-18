use std::{
    str::FromStr,
    thread::sleep,
    time::Duration,
};
use redis::{
    self,
    AsyncCommands,
    RedisResult,
    aio::Connection,
};
use warp::{
    Filter,
    Reply,
    Rejection,
};
use bitcoincore_rpc::{
    Auth, 
    Client as RpcClient, 
    RpcApi,
};
use tglib::{
    bdk::{
        bitcoin::{
            Address,
            Amount,
            PublicKey,
            consensus,
            hashes::hex::ToHex,
            util::{
                bip32::{
                    ExtendedPubKey,
                    DerivationPath,
                    Fingerprint,
                },
                psbt::PartiallySignedTransaction,
            },
            secp256k1::{
                Message,
                Secp256k1,
                Signature,
            }
        },
        blockchain::ElectrumBlockchain,
        database::MemoryDatabase,
        electrum_client::Client,
    },
    hex,
    Result,
    TgError,
    arbiter::{
        SendContractBody,
        SendPayoutBody,
        SetContractInfoBody,
    },
    contract::Contract,
    payout::Payout,
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
type WebResult<T> = std::result::Result<T, Rejection>;

fn wallet() -> Wallet<ElectrumBlockchain, MemoryDatabase> {
    let mut client = Client::new(ELECTRS_SERVER);
    while client.is_err() {
        sleep(Duration::from_secs(1));
        client = Client::new(ELECTRS_SERVER);
    }
    Wallet::<ElectrumBlockchain, MemoryDatabase>::new(Fingerprint::from_str(ARBITER_FINGERPRINT).unwrap(), ExtendedPubKey::from_str(ARBITER_XPUBKEY).unwrap(), ElectrumBlockchain::from(client.unwrap()), NETWORK).unwrap()
}

fn get_escrow_pubkey() -> Result<PublicKey> {
    Ok(EscrowWallet::get_escrow_pubkey(&wallet()))
}

fn get_fee_address() -> Result<Address> {
    let w = wallet();
    let a = w.xpubkey.derive_pub(&Secp256k1::new(), &DerivationPath::from_str("m/0/0").unwrap()).unwrap();
    Ok(Address::p2wpkh(&a.public_key, w.network).unwrap())
}

async fn push_contract(con: &mut Connection, hex: &str) -> RedisResult<String> {
    con.rpush("contracts", hex).await?;
    Ok(String::from(hex))
}

async fn push_payout(con: &mut Connection, hex: &str) -> RedisResult<String> {
    con.rpush("payouts", hex).await?;
    Ok(String::from(hex))
}

async fn submit_contract(con: &mut Connection, contract: &Contract) -> Result<Signature> {
    if wallet().validate_contract(&contract).is_ok() {
        let cxid = push_contract(con, &hex::encode(contract.to_bytes())).await.unwrap();
        for _ in 1..15 as u32 {
            sleep(Duration::from_secs(1));
            let r: RedisResult<String> = con.get(hex::encode(contract.cxid())).await;
            if let Ok(sig) = r {
                let _r : RedisResult<String> = con.del(cxid).await;
                return Ok(Signature::from_compact(&hex::decode(sig).unwrap()).unwrap())
            }
        }
    }
    Err(TgError("invalid contract".to_string()))
}

async fn submit_payout(con: &mut Connection, payout: &Payout) -> Result<PartiallySignedTransaction> {
    if wallet().validate_payout(&payout).is_ok() {
        println!("rbtr-public validated payout");
        let _r = push_payout(con, &hex::encode(payout.to_bytes())).await.unwrap();
        let cxid = hex::encode(payout.contract.cxid());
        for _ in 1..15 as u32 {
            sleep(Duration::from_secs(1));
            let r: RedisResult<String> = con.get(cxid.clone()).await;
            if let Ok(tx) = r {
                let _r : RedisResult<String> = con.del(cxid).await;
                return Ok(consensus::deserialize::<PartiallySignedTransaction>(&hex::decode(tx).unwrap()).unwrap())
            }
        }
    }
    Err(TgError("invalid payout".to_string()))
}

async fn set_contract_info_handler(body: SetContractInfoBody, redis_client: redis::Client) -> WebResult<impl Reply> {
// make sure pubkey controls the name
    match reqwest::get(&format!("{}/{}/{}", NAME_SERVICE_URL, "get-name-address", hex::encode(body.contract_info.name.0.as_bytes()))).await {
        Ok(response) => match response.text().await {
            Ok(name_address) => if get_namecoin_address(&body.pubkey, NETWORK).unwrap()!= name_address { println!("mismatched name address: {:?} vs {:?}", get_namecoin_address(&body.pubkey, NETWORK).unwrap(), name_address); return Err(warp::reject()) },
            Err(e) => { println!("{:?}", e); return Err(warp::reject()) },
        },
        Err(e) => { println!("{:?}", e); return Err(warp::reject()) },
    }
 
    let secp = Secp256k1::new();
    let sig = Signature::from_compact(&hex::decode(&body.sig_hex).unwrap()).unwrap();
    if secp.verify(&Message::from_slice(&body.contract_info.hash()).unwrap(), &sig, &body.pubkey.key).is_err() {
        return Err(warp::reject())
    }

    let mut con = redis_client.get_async_connection().await.unwrap();
    let r: RedisResult<String> = con.set(body.contract_info.name.clone().0, &serde_json::to_string(&body.contract_info).unwrap()).await;
    match r {
        Ok(_string) => Ok(format!("set contract info for {}", body.contract_info.name.0)),
        Err(_) => Err(warp::reject()),
    }
}

async fn get_contract_info_handler(player_name: String, redis_client: redis::Client) -> WebResult<impl Reply> {
    let mut con = redis_client.get_async_connection().await.unwrap();
    let r: RedisResult<String> = con.get(&String::from_utf8(hex::decode(player_name).unwrap()).unwrap()).await;
    match r {
        Ok(info) => Ok(info),
        Err(_) => Err(warp::reject()),
    }
}

async fn send_contract_handler(body: SendContractBody, redis_client: redis::Client) -> WebResult<impl Reply> {
    let mut con = redis_client.get_async_connection().await.unwrap();
    let r: RedisResult<String> = con.rpush(&format!("{}/contracts", body.player_name), serde_json::to_string(&body.contract).unwrap()).await;
    match r {
        Ok(_string) => Ok("sent contract".to_string()),
        Err(_) => Err(warp::reject()),
    }
}

async fn receive_contract_handler(player_name: String, redis_client: redis::Client) -> WebResult<impl Reply> {
// TODO: protect this endpoint with name auth
    let mut con = redis_client.get_async_connection().await.unwrap();
    let r: RedisResult<String> = con.lpop(&format!("{}/contracts", player_name)).await;
    match r {
        Ok(contract_json) => Ok(contract_json),
        Err(_) => Err(warp::reject()),
    }
}

async fn send_payout_handler(body: SendPayoutBody, redis_client: redis::Client) -> WebResult<impl Reply> {
    let mut con = redis_client.get_async_connection().await.unwrap();
    let r: RedisResult<String> = con.rpush(&format!("{}/payouts", body.player_name), serde_json::to_string(&body.payout).unwrap()).await;
    match r {
        Ok(_string) => Ok("sent payout".to_string()),
        Err(_) => Err(warp::reject()),
    }
}

async fn receive_payout_handler(player_name: String, redis_client: redis::Client) -> WebResult<impl Reply> {
// TODO: protect this endpoint with name auth
    let mut con = redis_client.get_async_connection().await.unwrap();
    let r: RedisResult<String> = con.lpop(&format!("{}/payouts", player_name)).await;
    match r {
        Ok(payout_json) => Ok(payout_json),
        Err(_) => Err(warp::reject()),
    }
}

async fn submit_contract_handler(contract: Contract, redis_client: redis::Client) -> WebResult<impl Reply> {
    let mut con = redis_client.get_async_connection().await.unwrap();
    if let Ok(sig) = submit_contract(&mut con, &contract).await {
        Ok(hex::encode(sig.serialize_compact()))
    } else {
        Err(warp::reject())
    }
}

// TODO: somehow break out of serialize(decode( hell
async fn submit_payout_handler(payout: Payout, redis_client: redis::Client) -> WebResult<impl Reply> {
    let mut con = redis_client.get_async_connection().await.unwrap();
    if let Ok(tx) = submit_payout(&mut con, &payout).await {
        Ok(hex::encode(consensus::serialize(&tx)))
    } else {
        Err(warp::reject())
    }
}

async fn fund_address_handler(address: String) -> WebResult<impl Reply> { 
    let address = Address::from_str(&address).unwrap();
    let bitcoin_rpc_client = RpcClient::new(BITCOIN_RPC_URL.to_string(), Auth::UserPass("admin".to_string(), "passw".to_string())).unwrap();
    let coinbase_addr = bitcoin_rpc_client.get_new_address(None, None).unwrap();
    let txid = bitcoin_rpc_client.send_to_address(&address, Amount::ONE_BTC, None, None, None, None, None, None).unwrap();
    let _blockhashes = bitcoin_rpc_client.generate_to_address(150, &coinbase_addr).unwrap();
    Ok(txid.to_hex())
}

fn redis_client() -> redis::Client {
    let mut client = redis::Client::open(REDIS_SERVER);
    while client.is_err() {
        sleep(Duration::from_secs(1));
        client = redis::Client::open(REDIS_SERVER);
    }
    client.unwrap()
}

#[tokio::main]
async fn main() {
    
    let escrow_pubkey = get_escrow_pubkey().unwrap();
    let escrow_pubkey = warp::any().map(move || escrow_pubkey.clone());
    let fee_address = get_fee_address().unwrap();
    let fee_address = warp::any().map(move || fee_address.clone());
    let redis_client = redis_client();
    let redis_client = warp::any().map(move || redis_client.clone());

    let get_escrow_pubkey = warp::path("escrow-pubkey")
        .and(escrow_pubkey)
        .map(|e: PublicKey | e.to_string()); 

    let get_fee_address = warp::path("fee-address")
        .and(fee_address)
        .map(|f: Address| f.to_string()); 

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
        .and(warp::path::param::<String>())
        .and(redis_client.clone())
        .and_then(receive_contract_handler);

    let send_payout = warp::path("send-payout")
        .and(warp::post())
        .and(warp::body::json())
        .and(redis_client.clone())
        .and_then(send_payout_handler);

    let receive_payout = warp::path("receive-payout")
        .and(warp::path::param::<String>())
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
        .or(fund_address);

    warp::serve(routes).run(([0, 0, 0, 0], 5000)).await;
}
