use std::time::Duration;
use redis::{
    self,
    AsyncCommands,
    RedisResult,
    aio::Connection,
};
use simple_logger::SimpleLogger;
use tokio::time::sleep;
use warp::{
    Filter,
    Reply,
    Rejection,
};

use libexchange::{
    AuthTokenSig,
    PlayerContractInfo,
    SendContractBody,
    SendPayoutBody,
    SetContractInfoBody,
};

use tglib::{
    bdk::{
        bitcoin::{
            blockdata::transaction::OutPoint,
            secp256k1::{
                Message,
                Secp256k1,
                Signature,
            },
            PublicKey,
            Script,
            Txid,
            util::psbt::Input,
        },
        electrum_client::{
            Client,
            ElectrumApi,
            ListUnspentRes,
        },
    },
    JsonResponse,
    hex,
    log::{
        debug,
        error,
        LevelFilter,
    },
    rand::{self, Rng},
    player::PlayerName,
    wallet::get_namecoin_address,
    mock::{
        ELECTRS_SERVER,
        NETWORK,
        REDIS_SERVER,
    },
};

type Response = JsonResponse<()>;

type Result<T> = std::result::Result<T, Box<dyn std::error::Error>>;
type WebResult<T> = std::result::Result<T, Rejection>;

const NAME_SERVICE_URL: &'static str = "http://nmc-id:18420";
const AUTH_TOKEN_LIFETIME: usize = 30;

async fn controls_name(pubkey: &PublicKey, player_name: &PlayerName) -> reqwest::Result<bool> {
    match reqwest::get(&format!("{}/{}/{}", NAME_SERVICE_URL, "get-name-address", hex::encode(player_name.0.as_bytes()))).await {
        Ok(response) => match response.text().await {
            Ok(name_address) => Ok(get_namecoin_address(pubkey, NETWORK) == name_address), 
            Err(e) => {
                error!("{:?}", e);
                Err(e)
            },
        },
        Err(e) => {
            error!("{:?}", e);
            Err(e)
        }
    }
}

async fn check_auth_token_sig(player_name: PlayerName, auth: AuthTokenSig, con: &mut Connection) -> Result<bool> {

    match controls_name(&auth.pubkey, &player_name).await {
        Ok(true) => (),
        _ => return Ok(false),
    }
    let secp = Secp256k1::new();
    let sig = Signature::from_der(&hex::decode(&auth.sig_hex).unwrap()).unwrap();
    let r: RedisResult<String> = con.get(format!("{}/token", player_name)).await;
    let token = match r {
        Ok(token) => token,
        Err(_) => return Ok(false),
    };
    Ok(secp.verify(&Message::from_slice(&hex::decode(token).unwrap()).unwrap(), &sig, &auth.pubkey.key).is_ok())
}

async fn set_contract_info_handler(body: SetContractInfoBody, redis_client: redis::Client) -> WebResult<impl Reply> {
    match controls_name(&body.pubkey, &body.contract_info.name).await {
        Ok(true) => (),
        Ok(false) => return Ok(warp::reply::json(&Response::error("pubkey doesn't control name".to_string(), None))),
        Err(e) => {
            error!("{:?}", e);
            return Ok(warp::reply::json(&Response::error(e.to_string(), None)))
        }
    }
 
    let secp = Secp256k1::new();
    let sig = Signature::from_der(&hex::decode(&body.sig_hex).unwrap()).unwrap();
    if secp.verify(&Message::from_slice(&body.contract_info.hash()).unwrap(), &sig, &body.pubkey.key).is_err() {
        return Ok(warp::reply::json(&Response::error("invalid signature".to_string(), None)))
    }

    let mut con = redis_client.get_async_connection().await.unwrap();
    let r: RedisResult<String> = con.set(format!("{}/info", body.contract_info.name.clone().0), &serde_json::to_string(&body.contract_info).unwrap()).await;
    match r {
        Ok(_string) => {
            Ok(warp::reply::json(&Response::success(None)))
        },
        Err(e) => {
            error!("{:?}", e);
            Ok(warp::reply::json(&Response::error(e.to_string(), None)))
        }
    }
}

async fn get_contract_info_handler(player_name: String, redis_client: redis::Client) -> WebResult<impl Reply> {
    debug!("get contract info for {}", String::from_utf8(hex::decode(&player_name).unwrap()).unwrap());
    let mut con = redis_client.get_async_connection().await.unwrap();
    let r: RedisResult<Option<String>> = con.get(format!("{}/info", &String::from_utf8(hex::decode(player_name).unwrap()).unwrap())).await;
    match r {
        Ok(Some(info)) => Ok(warp::reply::json(&JsonResponse::success(Some(info)))),
        Ok(None) => Ok(warp::reply::json(&Response::success(None))),
        Err(e) => {
            error!("{:?}", e);
            Ok(warp::reply::json(&Response::error(e.to_string(), None)))
        }
    }
}

async fn send_contract_handler(body: SendContractBody, redis_client: redis::Client) -> WebResult<impl Reply> {
    let mut con = redis_client.get_async_connection().await.unwrap();
    let r: RedisResult<i64> = con.rpush(&format!("{}/contracts", body.player_name), serde_json::to_string(&body.contract).unwrap()).await;
    match r {
        Ok(_num) => {
            Ok(warp::reply::json(&Response::success(None)))
        }
        Err(e) => {
            error!("send contract redis error: {:?}", e);
            Ok(warp::reply::json(&Response::error(e.to_string(), None)))
        }
    }
}

async fn receive_contract_handler(auth: AuthTokenSig, redis_client: redis::Client) -> WebResult<impl Reply> {
    let mut con = redis_client.get_async_connection().await.unwrap();
    match check_auth_token_sig(auth.player_name.clone(), auth.clone(), &mut con).await {
        Ok(true) => (),
        Ok(false) => {
            let message = "auth failed: invalid credentials";
            error!("{}", message);
            return Ok(warp::reply::json(&Response::error(message.into(), None)))
        },
        Err(e) => {
            let message = format!("auth failed: {:?}", e.to_string());
            error!("{}", message);
            return Ok(warp::reply::json(&Response::error(message.into(), None)))
        }
    }
    let r: RedisResult<Option<String>> = con.lpop(&format!("{}/contracts", auth.player_name.0)).await;
    match r {
        Ok(Some(contract_json)) => Ok(warp::reply::json(&JsonResponse::success(Some(contract_json)))),
        Ok(None) => Ok(warp::reply::json(&Response::success(None))),
        Err(e) => {
            let message = format!("couldn't receive contract: {:?}", e.to_string());
            error!("{}", message);
            return Ok(warp::reply::json(&Response::error(message.into(), None)))
        }
    }
}

async fn send_payout_handler(body: SendPayoutBody, redis_client: redis::Client) -> WebResult<impl Reply> {
    let mut con = redis_client.get_async_connection().await.unwrap();
    let r: RedisResult<i64> = con.rpush(&format!("{}/payouts", body.player_name.0), serde_json::to_string(&body.payout).unwrap()).await;
    match r {
        Ok(_string) => Ok(warp::reply::json(&Response::success(None))),
        Err(e) => {
            error!("send payout redis error {:?}", e);
            Ok(warp::reply::json(&Response::error(e.to_string(), None)))
        }
    }
}

async fn receive_payout_handler(auth: AuthTokenSig, redis_client: redis::Client) -> WebResult<impl Reply> {
    let mut con = redis_client.get_async_connection().await.unwrap();
    match check_auth_token_sig(auth.player_name.clone(), auth.clone(), &mut con).await {
        Ok(true) => (),
        Ok(false) => {
            let message = "auth failed: invalid credentials";
            error!("{}", message);
            return Ok(warp::reply::json(&Response::error(message.into(), None)))
        },
        Err(e) => {
            let message = format!("auth failed: {:?}", e.to_string());
            error!("{}", message);
            return Ok(warp::reply::json(&Response::error(message.into(), None)))
        }
    }

    let r: RedisResult<Option<String>> = con.lpop(&format!("{}/payouts", auth.player_name.0)).await;
    match r {
        Ok(Some(payout_json)) => Ok(warp::reply::json(&JsonResponse::success(Some(payout_json)))),
        Ok(None) => Ok(warp::reply::json(&Response::success(None))),
        Err(e) => {
            let message = format!("couldn't receive payout: {:?}", e.to_string());
            error!("{}", message);
            return Ok(warp::reply::json(&Response::error(message.into(), None)))
        }
    }
}

async fn auth_token_handler(player_name: String, redis_client: redis::Client) -> WebResult<impl Reply> {
    let token = hex::encode(rand::thread_rng().gen::<[u8; 32]>().to_vec());
    let mut con = redis_client.get_async_connection().await.unwrap();
// TODO: should we delete the token after it gets used or let them use it until it expires?
    let r: RedisResult<String> = con.set_ex(format!("{}/token", player_name), token.clone(), AUTH_TOKEN_LIFETIME).await;
    match r {
        Ok(_) => Ok(warp::reply::json(&JsonResponse::success(Some(token)))),
        Err(e) => {
            error!("{:?}", e);
            Ok(warp::reply::json(&Response::error(e.to_string(), None)))
        }
    }
}

async fn remove_stale_contract_info() -> Result<()> {
//TODO: try using script pubkey subscription instead
    let redis_client = redis_client();
    let mut con = redis_client.get_async_connection().await.unwrap();
    loop {
        sleep(Duration::from_secs(10)).await;
        let electrum_client = match Client::new(ELECTRS_SERVER) {
            Ok(client) => client,
            Err(e) => {
                error!("couldn't create electrum client: {:?}",e);
                continue
            }
        };
        let info_keys: Vec<String>  = con.keys("*/info").await?;
        for key in info_keys {
            let info: String = con.get(&key).await?;
            let info: PlayerContractInfo = serde_json::from_str(&info)?;
            let txids: Vec<Txid> = info.utxos.iter().map(|utxo| utxo.0.txid).collect();
            debug!("posted utxos: {}", info.utxos.len());
// TODO: this is failing when the txids aren't found
            let txs = electrum_client.batch_transaction_get(&txids)?;
            let utxo_scripts: Vec<Script> = txs
                .iter()
                .flat_map(|tx| {
                    tx.output
                    .iter()
                    .zip(vec![tx; tx.output.len()])
                    .enumerate()
                    .filter_map(|(vout, (output, tx))| {
                        if info.utxos.iter().find(|utxo| {
                            utxo.0.txid == tx.txid() &&
                            utxo.0.vout as usize == vout}).is_some() {
                            Some(output.clone().script_pubkey)
                        } else { 
                            None 
                        }
                    })
                })
                .collect();
            let unspent_utxos = electrum_client.batch_script_list_unspent(&utxo_scripts)?;
            let unspent_utxos: Vec<&ListUnspentRes> = unspent_utxos.iter().flatten().collect();

            debug!("unspent utxos matching posted utxos: {}", unspent_utxos.len());

            let new_info_utxos: Vec<(OutPoint, u64, Input)> = info.utxos.iter().filter(|(outpoint, amount, _)| 
                    unspent_utxos.iter().find(|list_unspent_res| {
                        list_unspent_res.tx_hash == outpoint.txid && 
                        list_unspent_res.tx_pos == outpoint.vout as usize &&
                        &list_unspent_res.value == amount
                    }
                ).is_some()
            ).cloned().collect();
            
            if !new_info_utxos.is_empty() {
                let new_info = PlayerContractInfo {
                    utxos: new_info_utxos,
                    ..info
                };
                con.set(key, serde_json::to_string(&new_info)?).await?;
            } else {
                con.del(key).await?
            }
        }
    }
}

fn redis_client() -> redis::Client {
    let mut client = redis::Client::open(REDIS_SERVER);
    while client.is_err() {
        std::thread::sleep(Duration::from_millis(100));
        client = redis::Client::open(REDIS_SERVER);
    }
    client.unwrap()
}

#[tokio::main]
async fn main() {
    SimpleLogger::new()
        .with_level(LevelFilter::Debug)
        .with_module_level("warp",LevelFilter::Warn)
        .with_module_level("hyper",LevelFilter::Warn)
        .with_module_level("reqwest",LevelFilter::Warn)
        .init()
        .unwrap();
    
    let redis_client = redis_client();
    let redis_client = warp::any().map(move || redis_client.clone());

    let set_contract_info = warp::path("set-contract-info")
        .and(warp::post())
        .and(warp::body::json())
        .and(redis_client.clone())
        .and_then(set_contract_info_handler);

    let get_contract_info = warp::path("get-contract-info")
        .and(warp::path::param::<String>())
        .and(redis_client.clone())
        .and_then(get_contract_info_handler);

    let send_contract = warp::path("send-contract")
        .and(warp::post())
        .and(warp::body::json())
        .and(redis_client.clone())
        .and_then(send_contract_handler);

    let receive_contract = warp::path("receive-contract")
        .and(warp::post())
        .and(warp::body::json())
        .and(redis_client.clone())
        .and_then(receive_contract_handler);

    let send_payout = warp::path("send-payout")
        .and(warp::post())
        .and(warp::body::json())
        .and(redis_client.clone())
        .and_then(send_payout_handler);

    let receive_payout = warp::path("receive-payout")
        .and(warp::post())
        .and(warp::body::json())
        .and(redis_client.clone())
        .and_then(receive_payout_handler);

    let auth_token = warp::path("auth-token")
        .and(warp::path::param::<String>())
        .and(redis_client.clone())
        .and_then(auth_token_handler);

    let routes = set_contract_info
        .or(get_contract_info)
        .or(send_contract)
        .or(receive_contract)
        .or(send_payout)
        .or(receive_payout)
        .or(auth_token);

    let _ = tokio::join!(
        warp::serve(routes).run(([0, 0, 0, 0], 5050)),
        remove_stale_contract_info(),
    );
}
