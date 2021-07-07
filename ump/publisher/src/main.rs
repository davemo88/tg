use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::mpsc::{channel, Sender};
use tokio::sync::RwLock;
//use simple_logger::SimpleLogger;
use warp::Filter;

mod db;

use db::Db;

use ump::{
    bitcoin::secp256k1::{Message, Signature, Secp256k1},
    chrono::{Duration, Local},
    hex,
    AddSignatureBody,
    GameInfo,
    JsonResponse,
    Team,
    ump_pubkey,
    UMP_PUBKEY,
};

type CachedGameInfo = Arc<RwLock<Vec<GameInfo>>>;

async fn update_cached_game_info(cache: CachedGameInfo, db_tx: &Sender<Job<Db>>) {

    let (query_tx, query_rx) = tokio::sync::oneshot::channel::<Vec<GameInfo>>();

    let query = move |db: &Db| {
        let mut stmt = db.conn.prepare(
            "SELECT 
                game.id                 AS game_id, 
                outcome.id              AS outcome_id, 
                home.id                 AS home_id, 
                home.name               AS home_name, 
                home.location           AS home_location, 
                away.id                 AS away_id, 
                away.name               AS away_name, 
                away.location           AS away_location, 
                game.date               AS date, 
                outcome_variant.name    AS outcome_variant, 
                token                   AS token, 
                hex                     AS sig_hex
            FROM game JOIN outcome ON game.id = outcome.game_id
            JOIN outcome_variant ON outcome_variant.id = outcome.variant_id 
            LEFT JOIN signature ON signature.outcome_id = outcome.id
            JOIN team AS home ON game.home_id = home.id
            JOIN team AS away on game.away_id = away.id"
        ).unwrap();
        
        let mut map = HashMap::<i64, GameInfo>::new();

        let _rows: Vec<rusqlite::Result<()>> = stmt.query_map([], |row| { 
            let game_id = row.get("game_id")?;
            match map.get_mut(&game_id) {
                None => {
                    let mut outcome_tokens = HashMap::<String, (i64, String, Option<String>)>::new();
// outcome variant -> outcome_id, token, hex
                    outcome_tokens.insert(row.get("outcome_variant")?, (row.get("outcome_id")?, row.get("token")?, row.get("sig_hex")?));
                    let info = GameInfo {
                        home: Team {
                            id: row.get("home_id")?,
                            name: row.get("home_name")?,
                            location: row.get("home_location")? 
                        },
                        away: Team {
                            id: row.get("away_id")?,
                            name: row.get("away_name")?,
                            location: row.get("away_location")? 
                        },
                        date: row.get("date")?,
                        outcome_tokens,
                    };
                    map.insert(game_id, info);
                }
                Some(info) => {
                    info.outcome_tokens.insert(row.get("outcome_variant")?, (row.get("outcome_id")?, row.get("token")?, row.get("sig_hex")?));
                }
            }
            Ok(())
        }).unwrap().collect();

        let _r = query_tx.send(map.values().cloned().collect());
    };

    let _r = db_tx.send(Box::new(query)).await;

    let new_cache = query_rx.await.unwrap();
    let mut w = cache.write().await; 
    *w = new_cache;
}

async fn get_game_info_handler(cache: CachedGameInfo) -> std::result::Result<impl warp::Reply, warp::Rejection> {
    Ok(serde_json::to_string(&JsonResponse::success(Some(cache.read().await.clone()))).unwrap())
}

async fn add_signature_handler(body: AddSignatureBody, db_tx: Sender<Job<Db>>, cache: CachedGameInfo) -> std::result::Result<impl warp::Reply, warp::Rejection> {
    let sig = match Signature::from_der(&hex::decode(&body.sig_hex).unwrap()) {
        Ok(sig) => sig,
        Err(_e) => return Err(warp::reject()),
    };
    let (query_tx, query_rx) = tokio::sync::oneshot::channel::<Option<String>>();
// get corresponding token
    let outcome_id = body.outcome_id;
    let query = move |db: &Db| {
        let mut stmt = db.conn.prepare("SELECT token FROM outcome WHERE outcome.id = ?").unwrap();
        let token: Option<String> = match stmt.query_row(rusqlite::params!(outcome_id), |row| Ok(row.get(0).unwrap())) {
            Ok(token) => Some(token),
            Err(_e) => None,
        };
        query_tx.send(token).unwrap();
    };
    match db_tx.send(Box::new(query)).await {
        Ok(()) => (),
        Err(_e) => panic!("send error"),
    };
    let token = match query_rx.await.unwrap() {
        Some(token) => token,
        None => return Err(warp::reject()),
    };
    
    let ump_pubkey = ump_pubkey();
    let secp = Secp256k1::new();
// validate signature on token
    match secp.verify(&Message::from_slice(&hex::decode(token).unwrap()).unwrap(), &sig, &ump_pubkey.key) {
        Ok(()) => {
            let (query_tx, query_rx) = tokio::sync::oneshot::channel::<Result<(), rusqlite::Error>>();
// insert signature.unwrap()
            let query = move |db: &Db| {
                db.insert_signature(&body.outcome_id, &body.sig_hex).unwrap();
                query_tx.send(Ok(())).unwrap();
            };
            let _r = db_tx.send(Box::new(query)).await; 
// refresh cache
            match query_rx.await.unwrap() {
                Ok(()) => update_cached_game_info(cache, &db_tx).await,
                Err(_) => return Err(warp::reject()),
            }
        },
// else reject
        Err(e) => {
            println!("{:?}", e);
            return Err(warp::reject())
        }
    }
    Ok(warp::http::StatusCode::OK)
}

type Job<T> = Box<dyn FnOnce(&T) + Send>;

fn with_sender<T: Send>(
    sender: Sender<T>,
) -> impl Filter<Extract = (Sender<T>,), Error = std::convert::Infallible> + Clone
{
    warp::any().map(move || sender.clone())
}

#[tokio::main]
async fn main() {

// sqlite thread
    let (db_tx, mut db_rx) = channel::<Job<Db>>(100);

    let _join_handle = std::thread::spawn(move || {
        let mut db_path = std::env::current_dir().unwrap();
        db_path.push("publisher.db");
        let db = Db::new(&db_path).expect("couldn't open db");
        db.init().unwrap();

        let today = Local::today();
        let yesterday = today - Duration::days(1);


        match db.load_teams() {
            Ok(_) => println!("loaded teams successfully"),
            Err(e) =>  println!("{:?}", e),
        };
        match db.load_schedule(yesterday, today) {
            Ok(_) => println!("loaded schedule successfully"),
            Err(e) =>  println!("{:?}", e),
        };

        loop {
            match db_rx.blocking_recv() {
                Some(x) => x(&db),
                None => {
                    break
                },
            }
        }
    });

    let cached_game_info: CachedGameInfo = Arc::new(RwLock::new(vec!()));
    update_cached_game_info(cached_game_info.clone(), &db_tx).await;

//    println!("cached {:?}", cached_game_info.read().await);
    
    let cached_game_info = warp::any().map(move || cached_game_info.clone());

    let get_game_info = warp::path("game-info")
        .and(cached_game_info.clone())
        .and_then(get_game_info_handler)
        .with(warp::cors().allow_any_origin());

    let add_signature = warp::path("signature")
        .and(warp::post())
        .and(warp::body::json())
        .and(with_sender(db_tx.clone()))
        .and(cached_game_info.clone())
        .and_then(add_signature_handler);

    let get_ump_pubkey = warp::path("ump-pubkey")
        .map(|| serde_json::to_string(&JsonResponse::success(Some(UMP_PUBKEY))).unwrap())
        .with(warp::cors().allow_any_origin());


    let routes = get_game_info
        .or(get_ump_pubkey)
        .or(add_signature);

    warp::serve(routes).run(([0, 0, 0, 0], 6969)).await;
}
