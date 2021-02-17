use std::{
    thread::sleep,
    time::Duration,
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
            PublicKey,
            secp256k1::{
                Message,
                Secp256k1,
                Signature,
            },
            hashes::{
                sha256,
                HashEngine,
                Hash as BitcoinHash,
            },
        },
    },
    hex,
    player::RegisterNameBody,
    wallet::get_namecoin_address,
    mock::NETWORK,
};

mod rpc;
use rpc::{
    NamecoinRpcClient,
    NameScanOptions,
    NameShowOptions,
    NameStatus,
    STRING_ENCODING,
};

// docker-compose hostname
pub const NAMECOIN_RPC_URL: &'static str = "http://guyledouche:yodelinbabaganoush@nmcd:18443";
pub const PLAYER_NAME_PREFIX: &'static str = "player/";

type WebResult<T> = std::result::Result<T, Rejection>;

//async fn register_name_handler(name: String, pubkey: String, sig: String, nmc_rpc: NamecoinRpcClient) -> WebResult<impl Reply>{
async fn register_name_handler(body: RegisterNameBody, nmc_rpc: NamecoinRpcClient) -> WebResult<impl Reply>{
//    let name = String::from_utf8(hex::decode(name).unwrap()).unwrap();
//    let pubkey = PublicKey::from_slice(&hex::decode(pubkey).unwrap()).unwrap();
    let sig = Signature::from_compact(&hex::decode(body.sig_hex).unwrap()).unwrap();
    let mut engine = sha256::HashEngine::default();
    engine.input(body.player_name.0.as_bytes());
    let hash: &[u8] = &sha256::Hash::from_engine(engine);

    let secp = Secp256k1::new();
    if secp.verify(&Message::from_slice(hash).unwrap(), &sig, &body.pubkey.key).is_err() {
        return Err(warp::reject())
    }
    let _r = nmc_rpc.import_pubkey(&body.pubkey);
// create namecoin address from supplied pubkey
    let name_address = get_namecoin_address(&body.pubkey, NETWORK).unwrap();
    let name = format!("{}{}",PLAYER_NAME_PREFIX, body.player_name.0);
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

async fn get_player_names_handler(pubkey: String, nmc_rpc: NamecoinRpcClient) -> WebResult<impl Reply> {
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

async fn get_name_address_handler(name_hex: String, nmc_rpc: NamecoinRpcClient) -> WebResult<impl Reply> {
    let options = NameShowOptions {
        name_encoding: STRING_ENCODING.to_string(),
        value_encoding: STRING_ENCODING.to_string(),
        by_hash: "direct".to_string(),
        allow_expired: None,
    };
// TODO maybe just put namecoind in hex encoding mode
    let name = format!("{}{}",PLAYER_NAME_PREFIX, String::from_utf8(hex::decode(name_hex).unwrap()).unwrap());
//    let new_address = nmc_rpc.get_new_address().await.unwrap();
    let name_status = match nmc_rpc.name_show(&name, Some(options)).await {
        Ok(r) => match r {
            Some(name_status) => name_status,
            None => return Err(warp::reject())
        }
        Err(e) => { println!("{:?}", e); return Err(warp::reject()) },
    };
    Ok(name_status.address)
}

async fn load_wallet(nmc_rpc_client: &NamecoinRpcClient) {
    let mut wallet_loaded = false;
    while !wallet_loaded {
        if let Ok(r) = nmc_rpc_client.load_wallet("testwallet").await {
            match r.base.error {
                Some(err) => {
                    match err.code {
// wallet already loaded
                        -4|-35 => {
                            wallet_loaded = true;
                        }
// no wallet loaded, try to create it
                        -18 => nmc_rpc_client.create_wallet("testwallet").await.unwrap(),
                        _ => println!("error loading wallet: {:?}", err),
                    }
                }
                None => wallet_loaded = true,
            }

        }
        if !wallet_loaded {
            sleep(Duration::from_secs(1));
        }
    }
}

#[tokio::main]
async fn main() {

    let nmc_rpc_client = NamecoinRpcClient::new(NAMECOIN_RPC_URL);
     
    load_wallet(&nmc_rpc_client).await;
    let new_address = nmc_rpc_client.get_new_address().await.unwrap();
    let _r = nmc_rpc_client.generate_to_address(150, new_address).await;

    let nmc_rpc = warp::any().map(move || nmc_rpc_client.clone());

    let register_name = warp::path("register-name")
        .and(warp::post())
        .and(warp::body::json())
        .and(nmc_rpc.clone())
        .and_then(register_name_handler);

    let get_player_names = warp::path("get-player-names")
        .and(warp::path::param::<String>())
        .and(nmc_rpc.clone())
        .and_then(get_player_names_handler);

    let get_name_address = warp::path("get-name-address")
        .and(warp::path::param::<String>())
        .and(nmc_rpc.clone())
        .and_then(get_name_address_handler);

    let routes = register_name
        .or(get_player_names)
        .or(get_name_address);

    warp::serve(routes).run(([0, 0, 0, 0], 18420)).await;
}
