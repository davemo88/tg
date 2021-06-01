use std::fmt;
use rusqlite::{params, Connection, Result};
use serde::{ Serialize, Deserialize, };
use simple_logger::SimpleLogger;
use warp::{ Filter, Reply, Rejection, };

#[derive(Debug)]
enum BaseballGameOutcome {
    HomeWins,
    AwayWins,
    Tie,
    Cancelled,
}

impl fmt::Display for BaseballGameOutcome {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
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
                if msg != "UNIQUE constraint failed: outcome_variant.id" {
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
            INSERT INTO outcome_variant (id, name) VALUES
            (?1, ?2),
            (?3, ?4),
            (?5, ?6),
            (?7, ?8);
            ",
            params![
                1.to_string(), BaseballGameOutcome::HomeWins.to_string(),
                2.to_string(), BaseballGameOutcome::AwayWins.to_string(),
                3.to_string(), BaseballGameOutcome::Tie.to_string(),
                4.to_string(), BaseballGameOutcome::Cancelled.to_string(),
            ]
        ) 
    }
}

fn load_schedule_csv(path: std::path::PathBuf) {
}

fn insert_team(name: &str) {
}

fn insert_game(home_id: u64, away_id: &u64, date: &str) {
}

fn insert_outcome(game_id: u64, variant: u32, ) {
}

fn insert_signature(outcome_id: u64, hex: u32, ) {
}

fn main() {
    let mut db_path = std::env::current_dir().unwrap();
    db_path.push("publisher.db");
    let db = Db::new(&db_path).unwrap();
    db.init().unwrap();
}
