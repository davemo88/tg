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
    contract::{
        Contract,
    },
    wallet::{
        EscrowWallet,
        SigningWallet,
    },
    mock::{
        REDIS_SERVER,
    },
};

#[tokio::main]
async fn main() {
    let redis_client = redis::Client::open(REDIS_SERVER).unwrap();
    let mut con = redis_client.get_async_connection().await.unwrap();
    loop {
        let contract_hex: redis::RedisResult<String> = con.lpop("contracts").await;
        if contract_hex.is_ok() {
            let contract_hex = contract_hex.unwrap();
            println!("{:?}", contract_hex);
            let contract = Contract::from_bytes(hex::decode(contract_hex).unwrap()).unwrap();
            println!("{:?}", contract);
        }
        sleep(Duration::from_secs(5));
    }
}
