use std::{
    thread::sleep,
    time::Duration,
};
use redis::{
    self,
    AsyncCommands,
    RedisResult,
};
use warp::{
    Filter,
    Reply,
    Rejection,
};
use serde_json;
use tglib::{
    bdk::{
        bitcoin::{
            Network,
            PublicKey,
            secp256k1::{
                Message,
                Secp256k1,
                Signature,
            },
            util::base58,
            hashes::{
                ripemd160,
                sha256,
                HashEngine,
                Hash as BitcoinHash,
            },
        },
    },
    hex,
    contract::PlayerContractInfo,
    mock::{
        NETWORK,
        REDIS_SERVER,
    }
};

mod rpc;
use rpc::{
    NamecoinRpcClient,
    NameScanOptions,
    NameStatus,
    STRING_ENCODING,
};

// docker-compose hostname
pub const NAMECOIN_RPC_URL: &'static str = "http://guyledouche:yodelinbabaganoush@nmcd:18443";
pub const PLAYER_NAME_PREFIX: &'static str = "player/";

type WebResult<T> = std::result::Result<T, Rejection>;
type NamecoinAddress = String;

// mainnet
//const NAMECOIN_VERSION_BYTE: u8 = 0x34;//52
// testnet / regtest, same as bitcoin?
const NAMECOIN_TESTNET_VERSION_BYTE: u8 = 0x6F;//111

fn get_namecoin_address(pubkey: &PublicKey, network: Network) -> Result<NamecoinAddress, String> {
    let mut sha256_engine = sha256::HashEngine::default();
    sha256_engine.input(&pubkey.key.serialize());
    let hash: &[u8] = &sha256::Hash::from_engine(sha256_engine);

    let mut ripemd160_engine = ripemd160::HashEngine::default();
    ripemd160_engine.input(hash);
    let hash = &ripemd160::Hash::from_engine(ripemd160_engine);

    let mut hash = hash.to_vec();
    match network {
        Network::Bitcoin => {
            panic!("nice try, sucker");
//            hash.insert(0,NAMECOIN_VERSION_BYTE);
        },
        Network::Regtest | Network::Testnet => {
            hash.insert(0,NAMECOIN_TESTNET_VERSION_BYTE);
        }
    }

    Ok(base58::check_encode_slice(&hash))
}

async fn register_name_handler(name: String, pubkey: String, sig: String, nmc_rpc: NamecoinRpcClient) -> WebResult<impl Reply>{
    let name = String::from_utf8(hex::decode(name).unwrap()).unwrap();
    let pubkey = PublicKey::from_slice(&hex::decode(pubkey).unwrap()).unwrap();
    let sig = Signature::from_compact(&hex::decode(sig).unwrap()).unwrap();
    let mut engine = sha256::HashEngine::default();
    engine.input(name.as_bytes());
    let hash: &[u8] = &sha256::Hash::from_engine(engine);

    let secp = Secp256k1::new();
    if secp.verify(&Message::from_slice(hash).unwrap(), &sig, &pubkey.key).is_err() {
        return Err(warp::reject())
    }
    let _r = nmc_rpc.import_pubkey(&pubkey);
// create namecoin address from supplied pubkey
    let name_address = get_namecoin_address(&pubkey, NETWORK).unwrap();
//    println!("name_address: {}", name_address);
    let name = format!("{}{}",PLAYER_NAME_PREFIX, name);
    let new_address = nmc_rpc.get_new_address().await.unwrap();
    match nmc_rpc.name_new(&name, &new_address).await {
        Ok((name_new_txid, rand)) => {
// need 12 blocks on top of the name_new
            let _r = nmc_rpc.generate_to_address(13, new_address.clone()).await;
//            let _name_firstupdate_txid = nmc_rpc.name_firstupdate(&name, &rand, &name_new_txid, Some("hello world"), &name_address).await.unwrap();
            match nmc_rpc.name_firstupdate(&name, &rand, &name_new_txid, Some("hello world"), &name_address).await {
                Ok(_txid) => (),
                Err(e) => return Ok(e.to_string())
            }
            let _r = nmc_rpc.generate_to_address(1, new_address).await;
// TODO: confirm name_firstupdate_txid in the chain
            Ok(name)
        }
        Err(msg) => {
            Ok(msg)
        }
    }
}

async fn get_contract_info_handler(player_name: String, redis: redis::Client) -> WebResult<impl Reply>{
    let mut con = redis.get_async_connection().await.unwrap();
    let r: RedisResult<String> = con.get(&String::from_utf8(hex::decode(player_name).unwrap()).unwrap()).await;
    match r {
        Ok(info) => Ok(info),
        Err(_) => Err(warp::reject()),
    }
}

async fn set_contract_info_handler(contract_info: String, pubkey: String, sig: String, redis: redis::Client) -> WebResult<impl Reply>{
    let contract_info: PlayerContractInfo = serde_json::from_str(&String::from_utf8(hex::decode(&contract_info).unwrap()).unwrap()).unwrap();
    let pubkey = PublicKey::from_slice(&hex::decode(pubkey).unwrap()).unwrap();
    let sig = Signature::from_compact(&hex::decode(sig).unwrap()).unwrap();
    let secp = Secp256k1::new();
    if secp.verify(&Message::from_slice(&contract_info.hash()).unwrap(), &sig, &pubkey.key).is_err() {
        return Err(warp::reject())
    }

    let mut con = redis.get_async_connection().await.unwrap();
    let r: RedisResult<String> = con.set(contract_info.name.clone().0, &serde_json::to_string(&contract_info).unwrap()).await;
    match r {
        Ok(_string) => Ok(format!("set contract info for {}", contract_info.name.0)),
        Err(_) => Err(warp::reject()),
    }
}

async fn get_player_names_handler(pubkey: String, nmc_rpc: NamecoinRpcClient) -> WebResult<impl Reply>{
    let options = NameScanOptions {
        name_encoding: STRING_ENCODING.to_string(),
        value_encoding: STRING_ENCODING.to_string(),
        min_conf: None,
        max_conf: 99999,
        prefix: format!("{}", PLAYER_NAME_PREFIX),
        regexp: "".to_string(),
    };

    let players = nmc_rpc.name_scan(None, None, Some(options)).await.unwrap();
    let pubkey = PublicKey::from_slice(&hex::decode(pubkey).unwrap()).unwrap();
    let namecoin_address = get_namecoin_address(&pubkey, NETWORK).unwrap();
    let controlled_players: Vec<NameStatus> = players.iter().filter(|p| p.address == namecoin_address).cloned().collect();
    Ok(serde_json::to_string::<Vec<String>>(&controlled_players.iter().map(|p| p.name.replace(PLAYER_NAME_PREFIX,"")).collect()).unwrap())
}

fn redis_client() -> redis::Client {
    let mut client = redis::Client::open(REDIS_SERVER);
    while client.is_err() {
        sleep(Duration::from_secs(1));
        client = redis::Client::open(REDIS_SERVER);
    }
    client.unwrap()
}

async fn load_wallet(nmc_rpc_client: &NamecoinRpcClient) {
    let mut wallet_loaded = false;
    while !wallet_loaded {
        if let Ok(r) = nmc_rpc_client.load_wallet("testwallet").await {
            match r.base.error {
                Some(err) => {
                    match err.code {
// wallet already loaded
                        -4 => {
                            wallet_loaded = true;
                        }
// no wallet loaded, try to create it
                        -18 => nmc_rpc_client.create_wallet("testwallet").await.unwrap(),
                        _ => ()
                    }
                }
                None => wallet_loaded = true,
            }

        }
        sleep(Duration::from_secs(1));
    }
}

#[tokio::main]
async fn main() {

    let nmc_rpc_client = NamecoinRpcClient::new(NAMECOIN_RPC_URL);
     
    load_wallet(&nmc_rpc_client).await;
    let new_address = nmc_rpc_client.get_new_address().await.unwrap();
    let _r = nmc_rpc_client.generate_to_address(150, new_address).await;

    let redis_client = redis_client();
    let redis = warp::any().map(move || redis_client.clone()); 
    let nmc_rpc = warp::any().map(move || nmc_rpc_client.clone());

    let register_name = warp::path("register-name")
        .and(warp::path::param::<String>())
        .and(warp::path::param::<String>())
        .and(warp::path::param::<String>())
        .and(nmc_rpc.clone())
        .and_then(register_name_handler);

    let set_contract_info = warp::path("set-contract-info")
        .and(warp::path::param::<String>())
        .and(warp::path::param::<String>())
        .and(warp::path::param::<String>())
        .and(redis.clone())
        .and_then(set_contract_info_handler);

    let get_contract_info = warp::path("get-contract-info")
        .and(warp::path::param::<String>())
        .and(redis.clone())
        .and_then(get_contract_info_handler);

    let get_player_name = warp::path("get-player-names")
        .and(warp::path::param::<String>())
        .and(nmc_rpc.clone())
        .and_then(get_player_names_handler);

    let routes = register_name
        .or(set_contract_info)
        .or(get_contract_info)
        .or(get_player_name);

    warp::serve(routes).run(([0, 0, 0, 0], 18420)).await;
}

#[cfg(test)]
mod tests {

    use super::*;
    use tglib::hex;

    const PUBKEY: &'static str = "02123e6a7816f2149f90cca1ea1ba41b73e77db44cd71f01c184defd10961d03fc";
    const TESTNET_ADDRESS_FROM_NAMECOIND: &'static str = "mfuf8qvMsMJMgBqtEGBt8aCQPQi1qgANzo";

    #[test]
    fn test_get_namecoin_address() {
        let pubkey = PublicKey::from_slice(&hex::decode(PUBKEY).unwrap()).unwrap();
        let namecoin_address = get_namecoin_address(&pubkey, Network::Testnet).unwrap();
        assert_eq!(namecoin_address,TESTNET_ADDRESS_FROM_NAMECOIND)
    }
}
