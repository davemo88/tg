use std::{
    str::FromStr,
    thread::sleep,
    time::Duration,
};
use bip39::Mnemonic;
use bdk::{
    bitcoin::{
        secp256k1::{
            Message,
        },
        util::{
            bip32::{
                DerivationPath,
            },
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
    let mut con = redis_client.get_async_connection().await.unwrap();
    loop {
        let r: redis::RedisResult<String> = con.lpop("contracts").await;
        if let Ok(contract_hex) = r {
            let contract = Contract::from_bytes(hex::decode(contract_hex).unwrap()).unwrap();
            let wallet = Wallet::new();
            println!("retrieved contract:\n{:?}", contract);
            if wallet.validate_contract(&contract).is_ok() {
                let sig = wallet.sign_message(Message::from_slice(&contract.cxid()).unwrap(), 
                    DerivationPath::from_str(&format!("m/{}/{}", ESCROW_SUBACCOUNT, ESCROW_KIX)).unwrap()).unwrap();
                let _: redis::RedisResult<String> = con.set(hex::encode(contract.cxid()), sig.to_string()).await;
                println!("signed contract {} and pushed sig to redis", hex::encode(contract.cxid()));
            }
        }
        sleep(Duration::from_secs(5));
    }
}
