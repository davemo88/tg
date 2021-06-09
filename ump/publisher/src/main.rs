use std::collections::HashMap;
use std::sync::Arc;
use rusqlite::{params, Connection, Result};
use tokio::sync::mpsc::{channel, Sender};
use tokio::sync::RwLock;
//use simple_logger::SimpleLogger;
use warp::Filter;

use ump::{
    bitcoin::secp256k1::{Message, Signature, Secp256k1},
    bitcoin::hashes::{
        Hash as BitcoinHash,
        HashEngine,
        sha256::Hash as ShaHash,
        sha256::HashEngine as ShaHashEngine,
    },
    chrono::{Date, Duration, Local},
    hex,
    AddSignatureBody,
    BaseballGameOutcome,
    GameInfo,
    JsonResponse,
    ump_pubkey,
    mlb_api::{
        MlbSchedule,
        MlbTeams,
        get_teams,
        get_schedule,
    }
};

#[derive(Debug)]
struct TeamRecord {
    id: i64,
    name: String,
}

#[derive(Debug)]
struct OutcomeVariant {
    id: i64,
    name: String,
}

#[derive(Debug)]
pub struct Db {
    pub conn: Connection,
}

impl Db {
    fn new(path: &std::path::Path) -> Result<Db> {
        Ok(Db { conn: Connection::open(path)? })
    }

    fn init(&self) -> Result<()> {
        self.create_tables()?;
        match self.init_outcome_variants() {
            Ok(_) => (),
            Err(rusqlite::Error::SqliteFailure(code, Some(msg))) => {
                if msg != "UNIQUE constraint failed: outcome_variant.name" {
                    return Err(rusqlite::Error::SqliteFailure(code, Some(msg)))
                }
            }
            Err(e) => return Err(e)
        };
        Ok(())
    }

    fn create_tables(&self) -> Result<()> {
        self.conn.execute_batch(
            "BEGIN;
                CREATE TABLE IF NOT EXISTS team (
                    id                  INTEGER PRIMARY KEY,  
                    name                TEXT UNIQUE
                );
                CREATE TABLE IF NOT EXISTS game (
                    id                  INTEGER PRIMARY KEY,  
                    home_id             INTEGER,
                    away_id             INTEGER,
                    date                TEXT,
                    UNIQUE(home_id, away_id, date),
                    FOREIGN KEY(home_id) REFERENCES team(id),
                    FOREIGN KEY(away_id) REFERENCES team(id)
                );
                CREATE TABLE IF NOT EXISTS outcome_variant (
                    id                  INTEGER PRIMARY KEY,  
                    name                TEXT UNIQUE
                );
                CREATE TABLE IF NOT EXISTS outcome (
                    id                  INTEGER PRIMARY KEY,  
                    game_id             INTEGER,
                    variant_id          INTEGER,
                    token               TEXT UNIQUE,
                    UNIQUE(game_id, variant_id),
                    FOREIGN KEY(game_id) REFERENCES game(id),
                    FOREIGN KEY(variant_id) REFERENCES outcome_variant(id)
                );
                CREATE TABLE IF NOT EXISTS signature (
                    id                  INTEGER PRIMARY KEY,  
                    outcome_id          INTEGER UNIQUE,
                    hex                 TEXT,
                    FOREIGN KEY(outcome_id) REFERENCES outcome(id)
                );
            COMMIT;"
        )
    }

    pub fn init_outcome_variants(&self) -> Result<usize> {
        self.conn.execute("
            INSERT INTO outcome_variant (name) VALUES
            (?1),
            (?2),
            (?3),
            (?4);
            ",
            params![
                BaseballGameOutcome::HomeWins.to_string(),
                BaseballGameOutcome::AwayWins.to_string(),
                BaseballGameOutcome::Tie.to_string(),
                BaseballGameOutcome::Cancelled.to_string(),
            ]
        ) 
    }

    fn insert_team(&self, id: &i64, name: &str) -> Result<usize> {
        self.conn.execute("
            INSERT INTO team (id, name) VALUES 
            (?1, ?2)
        ", params![id, name])
    }
    
    fn insert_game(&self, game_id: &i64, home_id: &i64, away_id: &i64, date: &str) -> Result<usize> {
        self.conn.execute("
            INSERT INTO game (id, home_id, away_id, date) VALUES 
            (?1, ?2, ?3, ?4)
        ", params![game_id, home_id, away_id, date])
    }
    
    fn insert_outcome(&self, game_id: &i64, variant_id: &i64, token: &str) -> Result<usize> {
        self.conn.execute("
            INSERT INTO outcome (game_id, variant_id, token) VALUES 
            (?1, ?2, ?3)
        ", params![game_id, variant_id, token])
    }
    
    fn insert_signature(&self, outcome_id: &i64, hex: &str, ) -> Result<usize> {
        self.conn.execute("
            INSERT INTO signature (outcome_id, hex) VALUES 
            (?1, ?2)
        ", params![outcome_id, hex])
    }

    fn load_schedule(&self, start_date: Date<Local>, end_date: Date<Local>) -> std::result::Result<(), Box<dyn std::error::Error>> {

        let outcome_variant_map = self.get_outcome_variant_map()?;
        let response = get_schedule(start_date, end_date, None)?.text()?;
        let schedule: MlbSchedule = serde_json::from_str(&response)?;
        for date in schedule.dates {
            for game in date.games {
                let home = game.teams.home.team;
                let away = game.teams.away.team;
                self.insert_game(
                    &game.id, 
                    &home.id,
                    &away.id,
                    &date.date,
                )?;
                for outcome in vec![BaseballGameOutcome::HomeWins, BaseballGameOutcome::AwayWins] {
                    self.insert_outcome(
                        &game.id, 
                        &outcome_variant_map.get(&outcome.to_string()).unwrap(), 
                        &get_outcome_token(&home.name, &away.name, &date.date, outcome)
                    )?;
                }
            }
        }

        Ok(()) 
    }
    
    fn load_teams(&self) -> std::result::Result<(), Box<dyn std::error::Error>> {
        let response = get_teams()?.text()?;
        let teams: MlbTeams = serde_json::from_str(&response)?;
        for team in teams.teams {
            self.insert_team(&team.id, &team.name)?;
        }
        Ok(())
    }

    fn get_outcome_variant_map(&self) -> Result<HashMap<String,i64>> {
        let mut stmt = self.conn.prepare("SELECT id, name FROM outcome_variant")?;
        let rows = stmt.query_map([], |row| { 
            Ok(OutcomeVariant { 
                id: row.get(0)?, 
                name: row.get(1)?,
            })
        })?;

        let mut map = HashMap::new();
        for row in rows {
            let row = row.unwrap();
            map.insert(row.name, row.id);
        }
        Ok(map)
    }
}

fn get_outcome_token(home: &str, away: &str, date: &str, outcome: BaseballGameOutcome) -> String {
    let mut engine = ShaHashEngine::default();
    engine.input(format!("H{}A{}D{}O{}", home, away, date, outcome.to_string()).as_bytes());
    let hash: &[u8] = &ShaHash::from_engine(engine);
    hex::encode(hash)
}

type CachedGameInfo = Arc<RwLock<Vec<GameInfo>>>;

async fn update_cached_game_info(cache: CachedGameInfo, db_tx: &Sender<Job<Db>>) {

    let (query_tx, query_rx) = tokio::sync::oneshot::channel::<Vec<GameInfo>>();

    let query = move |db: &Db| {
        let mut stmt = db.conn.prepare(
            "SELECT game.id, team1.name, team2.name, game.date, outcome_variant.name, token, hex, outcome.id
            FROM game JOIN outcome ON game.id = outcome.game_id
            JOIN outcome_variant ON outcome_variant.id = outcome.variant_id 
            LEFT JOIN signature ON signature.outcome_id = outcome.id
            JOIN team AS team1 ON game.home_id = team1.id
            JOIN team AS team2 on game.away_id = team2.id"
        ).unwrap();
        
        let mut map = HashMap::<i64, GameInfo>::new();

        let _rows: Vec<Result<()>> = stmt.query_map([], |row| { 
            let game_id = row.get(0)?;
            match map.get_mut(&game_id) {
                None => {
                    let mut outcome_tokens = HashMap::<String, (i64, String, Option<String>)>::new();
                    outcome_tokens.insert(row.get(4)?, (row.get(7)?, row.get(5)?, row.get(6)?));
                    let info = GameInfo {
                        home: row.get(1)?,
                        away: row.get(2)?,
                        date: row.get(3)?,
                        outcome_tokens,
                    };
                    map.insert(game_id, info);
                }
                Some(info) => {
                    info.outcome_tokens.insert(row.get(4)?, (row.get(7)?, row.get(5)?, row.get(6)?));
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
        let token: Option<String> = match stmt.query_row(params!(outcome_id), |row| Ok(row.get(0).unwrap())) {
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
            let (query_tx, query_rx) = tokio::sync::oneshot::channel::<Result<()>>();
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
        .and_then(get_game_info_handler);

    let add_signature = warp::path("signature")
        .and(warp::post())
        .and(warp::body::json())
        .and(with_sender(db_tx.clone()))
        .and(cached_game_info.clone())
        .and_then(add_signature_handler);

    let routes = get_game_info
        .or(add_signature);

    warp::serve(routes).run(([0, 0, 0, 0], 6000)).await;
}
