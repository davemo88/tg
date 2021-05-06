use std::{
    thread::sleep,
    time::Duration,
};
use redis::{
    self,
    aio::Connection,
    AsyncCommands,
};
use simple_logger::SimpleLogger;
use tglib::{
    bdk::bitcoin::{
       consensus,
        secp256k1::Signature,
        util::psbt::PartiallySignedTransaction,
    },
    hex,
    log::{
        LevelFilter,
        debug,
        error,
    },
    secrecy::Secret,
    contract::Contract,
    payout::Payout,
    wallet::{
        sign_contract,
        EscrowWallet,
    },
    mock::REDIS_SERVER,
};

mod wallet;
use wallet::Wallet;

const ARBITER_PW: &'static str = "somebogusarbiterpwweeee4j14hrkqj3htlkj";

async fn maybe_sign_contract(con: &mut Connection, wallet: &Wallet) {
    if let Some(contract) = next_contract(con).await {
        if wallet.validate_contract(&contract).is_ok() {
            if let Ok(sig) = sign_contract(wallet, &contract, Secret::new(ARBITER_PW.to_owned())) {
                let _r = set_contract_signature(con, contract, sig).await;
            }
        }
    }
}

async fn next_contract(con: &mut Connection) -> Option<Contract> {
    let r: redis::RedisResult<String> = con.lpop("contracts").await;
    Contract::from_bytes(hex::decode(r.ok()?).ok()?).ok()
}

async fn set_contract_signature(con: &mut Connection, contract: Contract, sig: Signature) -> redis::RedisResult<String> {
    con.set(hex::encode(contract.cxid()), hex::encode(sig.serialize_der())).await
}

async fn maybe_sign_payout(con: &mut Connection, wallet: &Wallet) {
    if let Some(payout) = next_payout(con).await {
        if wallet.validate_payout(&payout).is_ok() {
            debug!("payout validation succeeded");
            if let Ok(psbt) = wallet.sign_payout(payout.clone(), Secret::new(ARBITER_PW.to_owned())) {
                let _r = set_payout_psbt(con, payout, psbt).await;
            }
        }
    }
}

async fn next_payout(con: &mut Connection) -> Option<Payout> {
    let r: redis::RedisResult<String> = con.lpop("payouts").await;
    Payout::from_bytes(hex::decode(r.ok()?).ok()?).ok()
}

async fn set_payout_psbt(con: &mut Connection, payout: Payout, psbt: PartiallySignedTransaction) -> redis::RedisResult<String> {
    con.set(hex::encode(payout.contract.cxid()), hex::encode(consensus::serialize(&psbt))).await
}

#[tokio::main]
async fn main() {
    SimpleLogger::new().with_level(LevelFilter::Debug).init().unwrap();
    let redis_client = redis::Client::open(REDIS_SERVER).unwrap();
    let wallet = Wallet::new(Secret::new(ARBITER_PW.to_owned()));
    let mut waiting_time = Duration::from_secs(1);
    loop {
        match redis_client.get_async_connection().await {
            Ok(mut con) => {
                maybe_sign_contract(&mut con, &wallet).await;
                maybe_sign_payout(&mut con, &wallet).await;
                waiting_time = Duration::from_secs(1);
            }
            Err(e) => {
                error!("{}",e);
                waiting_time = waiting_time * 2;
            }
        }
        sleep(waiting_time);
    }
}
