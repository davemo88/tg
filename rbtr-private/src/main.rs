use std::{
    str::FromStr,
    thread::sleep,
    time::Duration,
};
use bip39::Mnemonic;
use bdk::{
    bitcoin::{
        consensus,
        secp256k1::{
            Message,
        },
        util::{
            bip32::{
                DerivationPath,
            },
            psbt::PartiallySignedTransaction,
        }
    }
};
use redis::{
    self,
    aio::Connection,
    AsyncCommands,
};
use tglib::{
    contract::{
        Contract,
    },
    payout::Payout,
    wallet::{
        EscrowWallet,
        SigningWallet,
    },
    mock::{
        Trezor,
        ARBITER_MNEMONIC,
        ESCROW_SUBACCOUNT,
        ESCROW_KIX,
        REDIS_SERVER,
    },
};

mod wallet;
use wallet::Wallet;

#[tokio::main]
async fn main() {
    let redis_client = redis::Client::open(REDIS_SERVER).unwrap();
    let wallet = Wallet::new();
    let mut con = redis_client.get_async_connection().await.unwrap();
    loop {
        let r: redis::RedisResult<String> = con.lpop("contracts").await;
        if let Ok(contract_hex) = r {
            let contract = Contract::from_bytes(hex::decode(contract_hex).unwrap()).unwrap();
            println!("retrieved contract:\n{:?}", contract);
            if wallet.validate_contract(&contract).is_ok() {
                let sig = wallet.sign_message(Message::from_slice(&contract.cxid()).unwrap(), 
                    DerivationPath::from_str(&format!("m/{}/{}", ESCROW_SUBACCOUNT, ESCROW_KIX)).unwrap()).unwrap();
                let _: redis::RedisResult<String> = con.set(hex::encode(contract.cxid()), sig.to_string()).await;
                println!("signed contract {} and pushed sig to redis", hex::encode(contract.cxid()));
            }
        }
        let r: redis::RedisResult<String> = con.lpop("payouts").await;
        if let Ok(payout_hex) = r {
            let payout = Payout::from_bytes(hex::decode(payout_hex).unwrap()).unwrap();
            println!("retrieved payout:\n{:?}", payout);
            if wallet.validate_payout(&payout).is_ok() {
                let tx: PartiallySignedTransaction = consensus::deserialize(&consensus::serialize(&payout.tx)).unwrap();
                let signed_tx = wallet.sign_tx(tx,"".to_string()).unwrap();
                let _: redis::RedisResult<String> = con.set(hex::encode(payout.contract.cxid()), hex::encode(consensus::serialize(&signed_tx))).await;
                println!("signed transaction for payout for contract {}", hex::encode(payout.contract.cxid()));

            }
        }
        sleep(Duration::from_secs(5));
    }
}
