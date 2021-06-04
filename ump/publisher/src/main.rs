use std::collections::{HashMap, HashSet};
use rusqlite::{params, Connection, Result};
use serde::{Serialize, Deserialize};
//use simple_logger::SimpleLogger;
use warp::Filter;
 
use tglib::{
    bdk::bitcoin::secp256k1::Signature,
    bdk::bitcoin::PublicKey,
    JsonResponse,
};

#[derive(Debug)]
enum BaseballGameOutcome {
    HomeWins,
    AwayWins,
    Tie,
    Cancelled,
}

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

impl std::fmt::Display for BaseballGameOutcome {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "{:?}", self)
    }
}

#[derive(Debug)]
pub struct Db {
    pub conn: Connection,
}

impl Db {
    pub fn new(path: &std::path::Path) -> Result<Db> {
        Ok(Db { conn: Connection::open(path)? })
    }

    pub fn init(&self) -> Result<()> {
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

    pub fn create_tables(&self) -> Result<()> {
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

    fn insert_team(&self, name: &str) -> Result<usize> {
        self.conn.execute("
            INSERT INTO team (name) VALUES 
            (?1)
        ", &[name])
    }
    
    fn insert_game(&self, home_id: &i64, away_id: &i64, date: &str) -> Result<usize> {
        self.conn.execute("
            INSERT INTO game (home_id, away_id, date) VALUES 
            (?1, ?2, ?3)
        ", params![home_id, away_id, date])
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

    fn load_schedule_csv(&self, path: &str) -> Result<(), Box<dyn std::error::Error>> {
        let mut rdr = csv::Reader::from_reader(std::fs::File::open(path)?);
        let mut games = vec![];
        let mut teams = HashSet::<String>::new();
        for result in rdr.deserialize() {
            let record: HashMap<String, String> = result?;
//            println!("{:?}", record);
            let game_teams = record.get("SUBJECT").unwrap()
                .split(" at ").collect::<Vec<&str>>();
            let (home, away) = (game_teams[1].to_owned(), game_teams[0].to_owned());
            let date = record.get("START DATE").unwrap().to_owned();
    
            teams.insert(away.to_owned());
            teams.insert(home.to_owned());
            games.push((away, home, date));
        }

        println!("num teams: {}, num games: {}", teams.len(), games.len());

        self.insert_new_teams(teams)?;
        let teams_map = self.teams_map()?;
        let outcome_variant_map = self.get_outcome_variant_map()?;

        for (away, home, date) in games {
            let away_id = teams_map.get(&away).unwrap(); 
            let home_id = teams_map.get(&home).unwrap(); 
            println!("inserting game: home: {} away: {} date: {}", away_id, home_id, date);
            match self.insert_game(home_id, away_id, &date) {
                Ok(_) => {
                    let game_id = self.get_game_id(&home_id, &away_id, &date)?;
                    for outcome in vec![BaseballGameOutcome::HomeWins, BaseballGameOutcome::AwayWins] {
                        self.insert_outcome(&game_id, &outcome_variant_map.get(&outcome.to_string()).unwrap(), 
                            &get_outcome_token(&home, &away, &date, outcome))?;
                    }
                },
                Err(e) => println!("{:?}", e),
            }
        }

        Ok(())
    
    }

    fn insert_new_teams(&self, teams: HashSet<String>) -> Result<()> {
    
        let mut stmt = self.conn.prepare("SELECT name FROM team")?;
        let known_teams: Vec<String> = stmt
            .query_map([], |row| { Ok(row.get(0)?) })?
            .map(|name| name.unwrap()).collect();

        let new_teams: Vec<&String> = teams.iter().filter(|team| !known_teams.contains(team)).collect();

        for new_team in new_teams {
            self.insert_team(new_team)?;
        }
        
        Ok(())
    }

    fn teams_map(&self) -> Result<HashMap<String, i64>> {
        let mut stmt = self.conn.prepare("SELECT id, name FROM team")?;
        let rows = stmt.query_map([], |row| { 
            Ok( TeamRecord { 
                id: row.get(0)?, 
                name: row.get(1)? 
            })
        })?;
    
        let mut map = HashMap::new();
        for row in rows {
            let row = row.unwrap();
            map.insert(row.name, row.id);
        }
        Ok(map)
    }

    fn get_game_id(&self, home_id: &i64, away_id: &i64, date: &str) -> Result<i64> {
        let mut stmt = self.conn.prepare(
            "SELECT id FROM game WHERE home_id = ? AND away_id = ? AND date = ?;"
        )?;
        let game_id: i64 = stmt.query_row(
            params![home_id, away_id, date],
            |row| { Ok(row.get(0)?) })?;
        Ok(game_id)
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
    return hex::encode(format!("H{}A{}D{}O{}", home, away, date, outcome.to_string()))
}

const ORIOLES_SCHEDULE_CSV_PATH: &'static str = "/home/hg/Downloads/EventTicketPromotionPrice.tiksrv";

use tokio::sync::mpsc::{channel, Sender};
use std::sync::Arc;
use tokio::sync::RwLock;

#[derive(Debug, Clone, Serialize, Deserialize)]
struct GameInfo {
    home: String,
    away: String,
    date: String,
    outcome_tokens: HashMap<String, (String, Option<String>)>,
}

type CachedGameInfo = Arc<RwLock<Vec<GameInfo>>>;

async fn update_cached_game_info(cache: CachedGameInfo, db_tx: &Sender<Job<Db>>) {

    let (query_tx, query_rx) = tokio::sync::oneshot::channel::<Vec<GameInfo>>();

    let query = move |db: &Db| {
        let mut stmt = db.conn.prepare(
            "SELECT game.id, team1.name, team2.name, game.date, outcome_variant.name, token, hex
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
                    let mut outcome_tokens = HashMap::<String, (String, Option<String>)>::new();
                    outcome_tokens.insert(row.get(4)?, (row.get(5)?, row.get(6)?));
                    let info = GameInfo {
                        home: row.get(1)?,
                        away: row.get(2)?,
                        date: row.get(3)?,
                        outcome_tokens,
                    };
                    map.insert(game_id, info);
                }
                Some(info) => {
                    info.outcome_tokens.insert(row.get(4)?, (row.get(5)?, row.get(6)?));
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

#[derive(Debug, Clone, Serialize, Deserialize)]
struct AddSignatureBody {
    outcome_id: i64,
    sig_hex: String,
}

async fn add_signature_handler(body: AddSignatureBody, db_tx: Sender<Job<Db>>, _cache: CachedGameInfo) -> std::result::Result<impl warp::Reply, warp::Rejection> {
    let sig = match Signature::from_der(&hex::decode(&body.sig_hex).unwrap()) {
        Ok(sig) => sig,
        Err(_e) => return Err(warp::reject()),
    };
    let (query_tx, query_rx) = tokio::sync::oneshot::channel::<Option<String>>();
// get corresponding token
    let query = move |db: &Db| {
        let mut stmt = db.conn.prepare("SELECT token FROM outcome WHERE outcome.id = ?").unwrap();
        let token: Option<String> = match stmt.query_row(params!(body.outcome_id), |row| Ok(row.get(0).unwrap())) {
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
// validate signature on token
// if valid
// insert signature
// refresh cache
// else reject
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

        match db.load_schedule_csv(ORIOLES_SCHEDULE_CSV_PATH) {
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

    println!("cached {:?}", cached_game_info.read().await);
    
//    let db_tx = warp::any().map(move || db_tx.clone());
    let cached_game_info = warp::any().map(move || cached_game_info.clone());

    let get_game_info = warp::path("game-info")
        .and(cached_game_info.clone())
        .and_then(get_game_info_handler);

    let add_signature = warp::path("signature")
        .and(warp::post())
        .and(warp::body::json())
        .and(with_sender(db_tx.clone()))
        .and(cached_game_info.clone())
        .and_then(move |body, db, cache| async move {
            add_signature_handler(body, db, cache).await
        });

    let routes = get_game_info
        .or(add_signature);

    warp::serve(routes).run(([0, 0, 0, 0], 6000)).await;
}
