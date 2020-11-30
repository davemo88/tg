use std::{
    thread::sleep,
    time::Duration,
};
use bdk::bitcoin::{
    Transaction,
    consensus,
    secp256k1::Signature,
};
use redis::{
    self,
    aio::Connection,
    AsyncCommands,
};
use tglib::{
    contract::Contract,
    payout::Payout,
    wallet::EscrowWallet,
    mock::REDIS_SERVER,
};

mod wallet;
use wallet::Wallet;

async fn maybe_sign_contract(con: &mut Connection, wallet: &Wallet) {
    if let Some(contract) = next_contract(con).await {
        println!("retrieved contract:\n{:?}", contract);
        if wallet.validate_contract(&contract).is_ok() {
            if let Ok(sig) = wallet.sign_contract(&contract) {
                println!("signed contract {}", hex::encode(contract.cxid()));
                let _ = set_contract_signature(con, contract, sig).await;
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
            if let Ok(tx) = wallet.sign_payout(&payout) {
                println!("signed transaction for payout for contract {}", hex::encode(payout.contract.cxid()));
                let _ = set_payout_tx(con, payout, tx).await;
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

async fn set_payout_tx(con: &mut Connection, payout: Payout, tx: Transaction) -> redis::RedisResult<String> {
    con.set(hex::encode(payout.contract.cxid()), hex::encode(consensus::serialize(&tx))).await
}

#[tokio::main]
async fn main() {
    let redis_client = redis::Client::open(REDIS_SERVER).unwrap();
    let mut con = redis_client.get_async_connection().await.unwrap();
    let wallet = Wallet::new();
    loop {
        maybe_sign_contract(&mut con, &wallet).await;
        maybe_sign_payout(&mut con, &wallet).await;
        sleep(Duration::from_secs(5));
    }
}
