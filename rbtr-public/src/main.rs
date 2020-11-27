use std::{
    str::FromStr,
    sync::{
        Arc,
    },
};
use bdk::{
    bitcoin::{
        Address,
        util::{
            bip32::{
                ExtendedPubKey,
                Fingerprint,
            }
        }
    },
};
use bip39::Mnemonic;
use hex::decode;
use redis::{
    self,
    AsyncCommands,
    FromRedisValue,
    RedisResult,
    aio::Connection,
};
use tokio::{
    sync::{
        Mutex,
        RwLock
    }
};
use warp::{
    Filter,
    Reply,
    Rejection,
};
use tglib::{
    Result,
    TgError,
    contract::Contract,
    payout::Payout,
    wallet::{
        EscrowWallet,
        SigningWallet,
    },
    mock::{
        Trezor,
        ARBITER_FINGERPRINT,
        ARBITER_XPUBKEY,
        NETWORK,
        REDIS_SERVER,
    },
};

mod wallet;
use wallet::Wallet;

type WebResult<T> = std::result::Result<T, Rejection>;
#[derive(Debug)]
struct Error;
impl warp::reject::Reject for Error {}

fn wallet() -> Wallet {
    Wallet::new(Fingerprint::from_str(ARBITER_FINGERPRINT).unwrap(), ExtendedPubKey::from_str(ARBITER_XPUBKEY).unwrap(), NETWORK)
}

async fn get_con(client: redis::Client) -> Connection {
    client
        .get_async_connection()
        .await
        .unwrap()
}

async fn push_contract(con: &mut Connection, cxid: &str, hex: &str, ttl_seconds: usize) -> RedisResult<String> {
    con.set(cxid, hex).await?;
    Ok(String::from(cxid))
}

async fn push_payout(con: &mut Connection, cxid: &str, hex: &str, ttl_seconds: usize) -> RedisResult<String> {
    con.set(cxid, hex).await?;
    Ok(String::from(cxid))
}

async fn contract_handler(contract_hex: String, client: redis::Client) -> WebResult<impl Reply> {
    let mut con = get_con(client).await;
    let cxid = push_contract(&mut con, "hello", "direct_world", 60)
    .await
    .unwrap();
    Ok(cxid)
}

async fn payout_handler(payout_hex: String, client: redis::Client) -> WebResult<impl Reply> {
    let mut con = get_con(client).await;
    let cxid = push_payout(&mut con, "hello", "direct_world", 60)
    .await
    .unwrap();
    Ok(cxid)
}

#[tokio::main]
async fn main() {

    let escrow_pubkey = wallet().get_escrow_pubkey();
    let fee_address = warp::any().map(move || Address::p2wpkh(&escrow_pubkey, NETWORK).unwrap());
    let escrow_pubkey = warp::any().map(move || escrow_pubkey.clone());
    let redis_client = redis::Client::open(REDIS_SERVER).unwrap();
    let redis_client = warp::any().map(move || redis_client.clone());

    let get_escrow_pubkey = warp::path("escrow-pubkey")
        .and(escrow_pubkey)
        .map(|e| format!("escrow_pubkey: {:?}", e)); 

    let get_fee_address = warp::path("fee-address")
        .and(fee_address)
        .map(|f| format!("fee address:   {:?}", f)); 

    let submit_contract = warp::path("submit-contract")
        .and(warp::path::param::<String>())
        .and(redis_client.clone())
        .and_then(contract_handler);

    let submit_payout = warp::path("submit-payout")
        .and(warp::path::param::<String>())
        .and(redis_client.clone())
        .and_then(payout_handler);

    let routes = get_escrow_pubkey
        .or(get_fee_address)
        .or(submit_contract)
        .or(submit_payout);

    warp::serve(routes).run(([127, 0, 0, 1], 3030)).await;
}
