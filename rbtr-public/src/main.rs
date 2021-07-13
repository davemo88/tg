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
            consensus,
            hashes::hex::ToHex,
            util::{
                bip32::{
                    ExtendedPubKey,
                    Fingerprint,
                },
                psbt::PartiallySignedTransaction,
            },
            secp256k1::Signature,
        },
        blockchain::ElectrumBlockchain,
        database::MemoryDatabase,
        electrum_client::Client,
    },
    hex,
    log::{
        error,
        LevelFilter,
    },
    Error,
    JsonResponse,
    arbiter::{
        SubmitContractBody,
        SubmitPayoutBody,
    },
    contract::Contract,
    payout::Payout,
    wallet::EscrowWallet,
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

type Response = JsonResponse<String>;

const BITCOIN_RPC_URL: &'static str = "http://electrs:18443";
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

async fn submit_contract_handler(body: SubmitContractBody, redis_client: redis::Client) -> WebResult<impl Reply> {
    let mut con = redis_client.get_async_connection().await.unwrap();
    let contract = Contract::from_bytes(hex::decode(body.contract_hex).unwrap()).unwrap();
    match submit_contract(&mut con, &contract).await {
        Ok(sig) => Ok(warp::reply::json(&Response::success(Some(hex::encode(sig.serialize_der()))))),
        Err(e) => {
            error!("{:?}", e);
            Ok(warp::reply::json(&Response::error(e.to_string(),None)))
        }
    }
}

// TODO: somehow break out of serialize(decode( hell
async fn submit_payout_handler(body: SubmitPayoutBody, redis_client: redis::Client) -> WebResult<impl Reply> {
    let mut con = redis_client.get_async_connection().await.unwrap();
    let payout = Payout::from_bytes(hex::decode(body.payout_hex).unwrap()).unwrap();
    match submit_payout(&mut con, &payout).await {
        Ok(tx) => Ok(warp::reply::json(&Response::success(Some(hex::encode(consensus::serialize(&tx)))))),
        Err(e) => {
            error!("{:?}", e);
            Ok(warp::reply::json(&Response::error(e.to_string(),None)))
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
        .or(submit_contract)
        .or(submit_payout)
        .or(fund_address);
    warp::serve(routes).run(([0, 0, 0, 0], 5000)).await;
}
