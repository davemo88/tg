use std::{
    thread::sleep,
    time::Duration,
};
use redis::{
    self,
    aio::Connection,
    AsyncCommands,
};
use tglib::{
    bdk::bitcoin::{
       consensus,
        secp256k1::Signature,
        util::psbt::PartiallySignedTransaction,
    },
    hex,
    secrecy::Secret,
    contract::Contract,
    payout::Payout,
    wallet::{
        sign_contract,
        sign_payout_psbt,
        EscrowWallet,
    },
    mock::REDIS_SERVER,
};

mod wallet;
use wallet::Wallet;

const ARBITER_PW: &'static str = "somebogusarbiterpwweeee4j14hrkqj3htlkj";

async fn maybe_sign_contract(con: &mut Connection, wallet: &Wallet) {
    if let Some(mut contract) = next_contract(con).await {
        println!("retrieved contract:\n{:?}", contract);
        if wallet.validate_contract(&contract).is_ok() {
            if let Ok(sig) = sign_contract(wallet, &mut contract, Secret::new(ARBITER_PW.to_owned())) {
                println!("signed contract {}", hex::encode(contract.cxid()));
                let _r = set_contract_signature(con, contract, sig).await;
            }
        }
    }
}

async fn next_contract(con: &mut Connection) -> Option<Contract> {
    let r: redis::RedisResult<String> = con.lpop("contracts").await;
    if let Ok(contract_hex) = r {
       Some(Contract::from_bytes(hex::decode(contract_hex).unwrap()).unwrap())
    } else {
        None
    }
}

async fn set_contract_signature(con: &mut Connection, contract: Contract, sig: Signature) -> redis::RedisResult<String> {
    con.set(hex::encode(contract.cxid()), hex::encode(sig.serialize_compact())).await
}

async fn maybe_sign_payout(con: &mut Connection, wallet: &Wallet) {
    if let Some(payout) = next_payout(con).await {
        println!("retrieved payout:\n{:?}", payout);
        if wallet.validate_payout(&payout).is_ok() {
            if let Ok(psbt) = sign_payout_psbt(wallet, payout.psbt.clone(), Secret::new(ARBITER_PW.to_owned())) {
                println!("signed transaction for payout for contract {}", hex::encode(payout.contract.cxid()));
                let _r = set_payout_psbt(con, payout, psbt).await;
            }
        }
    }
}

async fn next_payout(con: &mut Connection) -> Option<Payout> {
    let r: redis::RedisResult<String> = con.lpop("payouts").await;
    if let Ok(payout_hex) = r {
       Some(Payout::from_bytes(hex::decode(payout_hex).unwrap()).unwrap())
    } else {
        None
    }
}

async fn set_payout_psbt(con: &mut Connection, payout: Payout, psbt: PartiallySignedTransaction) -> redis::RedisResult<String> {
    con.set(hex::encode(payout.contract.cxid()), hex::encode(consensus::serialize(&psbt))).await
}

fn redis_client() -> redis::Client {
    sleep(Duration::from_secs(1));
    let mut client = redis::Client::open(REDIS_SERVER);
    while client.is_err() {
        sleep(Duration::from_secs(1));
        client = redis::Client::open(REDIS_SERVER);
    }
    client.unwrap()
}

#[tokio::main]
async fn main() {
    let redis_client = redis_client();
    let mut con = redis_client.get_async_connection().await.unwrap();
    let wallet = Wallet::new(Secret::new(ARBITER_PW.to_owned()));
    loop {
        maybe_sign_contract(&mut con, &wallet).await;
        maybe_sign_payout(&mut con, &wallet).await;
        sleep(Duration::from_secs(5));
    }
}
