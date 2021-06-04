use std::collections::{HashMap, HashSet};
use std::iter::FromIterator;
use rusqlite::{params, Connection, Result};
use serde::{Serialize, Deserialize};
use simple_logger::SimpleLogger;
use warp::{Filter, Reply, Rejection};

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

use std::sync::mpsc::{channel, Sender};
use std::sync::Arc;
use tokio::sync::RwLock;

#[derive(Debug, Clone)]
struct GameInfo {
    home: String,
    away: String,
    date: String,
    outcome_tokens: HashMap<String, (String, Option<String>)>,
}

type CachedGameInfo = Arc<RwLock<Vec<GameInfo>>>;

async fn update_cached_game_info(cache: CachedGameInfo, db_tx: Sender<Job<Db>>) {

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

    let _r = db_tx.send(Box::new(query));

    let new_cache = query_rx.await.unwrap();
    let mut w = cache.write().await; 
    *w = new_cache;
}

type Job<T> = Box<dyn FnOnce(&T) + Send >;

#[tokio::main]
async fn main() {

// sqlite thread
    let (db_tx, db_rx) = channel::<Job<Db>>();

    let join_handle = std::thread::spawn(move || {
        let mut db_path = std::env::current_dir().unwrap();
        db_path.push("publisher.db");
        let db = Db::new(&db_path).expect("couldn't open db");
        db.init().unwrap();

        match db.load_schedule_csv(ORIOLES_SCHEDULE_CSV_PATH) {
            Ok(_) => println!("loaded schedule successfully"),
            Err(e) =>  println!("{:?}", e),
        };

        loop {
            match db_rx.recv() {
                Ok(x) => x(&db),
                Err(e) => {
                    println!("recv error: {:?}", e);
                    break
                },
            }
        }
    });

    let cached_game_info: CachedGameInfo = Arc::new(RwLock::new(vec!()));
    update_cached_game_info(cached_game_info.clone(), db_tx.clone()).await;
    
    loop {
        let r1 = cached_game_info.read().await;
        if !r1.is_empty() {
            println!("updated cached game info: {:?}", r1);
            break
        }
    }

//    let sql = "SELECT * FROM team WHERE name = ?";
//    let params = "Orioles";
//
//    let (query_tx, query_rx) = tokio::sync::oneshot::channel::<Vec<Result<TeamRecord>>>();
//
//    let some_query = move |db: &Db| {
//        let mut stmt = match db.conn.prepare(&sql) {
//            Ok(stmt) => stmt,
//            Err(e) => panic!("couldn't prepare sql statement: {:?}", e)
//        };
//        let rows = stmt.query_map(params!(params), |row| { 
//            Ok(TeamRecord { 
//                id: row.get(0)?, 
//                name: row.get(1)?, 
//            })
//        }).unwrap().collect::<Vec<Result<TeamRecord>>>();
//
//        let _r = query_tx.send(rows);
//    };
//
//    let _r = db_tx.send(Box::new(some_query));
//
//    let rows = query_rx.await.unwrap();
//    
    drop(db_tx);
//
    let _r = join_handle.join();
//
//    println!("{:?}", rows);
}
