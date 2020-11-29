use std::{
    str::FromStr,
    sync::{
        Arc,
    },
    thread::sleep,
    time::Duration,
};
use serde_json;
use bdk::{
    bitcoin::{
        Address,
        PublicKey,
        util::{
            bip32::{
                ExtendedPubKey,
                Fingerprint,
            }
        }
    },
    blockchain::{
        ElectrumBlockchain,
    },
    database::{
        MemoryDatabase,
    },
    electrum_client::Client,
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
    arbiter::ArbiterService,
    contract::Contract,
    payout::Payout,
    player::PlayerId,
    wallet::{
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

type WebResult<T> = std::result::Result<T, Rejection>;

fn wallet() -> Wallet<ElectrumBlockchain, MemoryDatabase> {
    let mut client = Client::new(ELECTRS_SERVER, None);
    while client.is_err() {
        println!("connection to electrs failed");
        sleep(Duration::from_secs(1));
        client = Client::new(ELECTRS_SERVER, None);
    }
    println!("connection to electrs succeeded");
    Wallet::<ElectrumBlockchain, MemoryDatabase>::new(Fingerprint::from_str(ARBITER_FINGERPRINT).unwrap(), ExtendedPubKey::from_str(ARBITER_XPUBKEY).unwrap(), ElectrumBlockchain::from(client.unwrap()), NETWORK).unwrap()
}

async fn contract_handler(contract_hex: String, client: redis::Client) -> WebResult<impl Reply> {
    let contract = Contract::from_bytes(hex::decode(contract_hex.clone()).unwrap()).unwrap();
    if wallet().validate_contract(&contract).is_ok() {
        let mut con = get_con(client).await;
        let cxid = push_contract(&mut con, &contract_hex, 60).await.unwrap();
        for i in 1..15 as u32 {
            let sig: RedisResult<String> = con.get(hex::encode(contract.cxid())).await;
            if sig.is_ok() {
                return Ok(sig.unwrap())
            }
            sleep(Duration::from_secs(1));
        }
    }
    Err(warp::reject())
}

async fn push_contract(con: &mut Connection, hex: &str, ttl_seconds: usize) -> RedisResult<String> {
    con.rpush("contracts", hex).await?;
    Ok(String::from(hex))
}

async fn get_con(client: redis::Client) -> Connection {
    client.get_async_connection().await.unwrap()
}

async fn payout_handler(payout_hex: String, client: redis::Client) -> WebResult<impl Reply> {
    let payout = Payout::from_bytes(hex::decode(payout_hex.clone()).unwrap()).unwrap();
    if wallet().validate_payout(&payout).is_ok() {
        let mut con = get_con(client).await;
        let cxid = push_payout(&mut con, &hex::encode(payout.contract.cxid()), &payout_hex, 60).await.unwrap();
        Ok(cxid)
    } else {
        Err(warp::reject())
    }
}

async fn push_payout(con: &mut Connection, cxid: &str, hex: &str, ttl_seconds: usize) -> RedisResult<String> {
    con.set(cxid, hex).await?;
    Ok(String::from(cxid))
}

async fn player_info_handler(player_id: String) -> WebResult<impl Reply> {
    let info = wallet().get_player_info(PlayerId(player_id)).unwrap();
    Ok(serde_json::to_string(&info).unwrap())
}

#[tokio::main]
async fn main() {

    let wallet = wallet();
    let escrow_pubkey = EscrowWallet::get_escrow_pubkey(&wallet);
    let fee_address = wallet.get_fee_address().unwrap();
    let fee_address = warp::any().map(move || fee_address.clone());
    let escrow_pubkey = warp::any().map(move || escrow_pubkey.clone());
    let redis_client = redis::Client::open(REDIS_SERVER).unwrap();
    let redis_client = warp::any().map(move || redis_client.clone());

    let get_escrow_pubkey = warp::path("escrow-pubkey")
        .and(escrow_pubkey)
        .map(|e: PublicKey | e.to_string()); 

    let get_fee_address = warp::path("fee-address")
        .and(fee_address)
        .map(|f: Address| f.to_string()); 

    let submit_contract = warp::path("submit-contract")
        .and(warp::path::param::<String>())
        .and(redis_client.clone())
        .and_then(contract_handler);

    let submit_payout = warp::path("submit-payout")
        .and(warp::path::param::<String>())
        .and(redis_client.clone())
        .and_then(payout_handler);

    let player_info = warp::path("info")
        .and(warp::path::param::<String>())
        .and_then(player_info_handler);

    let routes = get_escrow_pubkey
        .or(get_fee_address)
        .or(submit_contract)
        .or(submit_payout)
        .or(player_info);

    warp::serve(routes).run(([0, 0, 0, 0], 5000)).await;
}
