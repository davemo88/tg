use std::collections::HashMap;
use rusqlite::{params, Connection, Result};
use ump::{
    chrono::{Date, Local},
    BaseballGameOutcome,
    get_outcome_token,
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
    pub fn new(path: &std::path::Path) -> Result<Db> {
        Ok(Db { conn: Connection::open(path)? })
    }

    pub fn init(&self) -> Result<()> {
        self.create_tables()?;
        match self.init_outcome_variants() {
            Ok(_) => (),
            Err(rusqlite::Error::SqliteFailure(code, Some(msg))) => {
//TODO: get rid of this or move it into the called function
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
                    name                TEXT UNIQUE,
                    location            TEXT
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

    fn init_outcome_variants(&self) -> Result<usize> {
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

    pub fn insert_team(&self, id: &i64, name: &str, location: &str) -> Result<usize> {
        self.conn.execute("
            INSERT INTO team (id, name, location) VALUES 
            (?1, ?2, ?3)
        ", params![id, name, location])
    }
    
    pub fn insert_game(&self, game_id: &i64, home_id: &i64, away_id: &i64, date: &str) -> Result<usize> {
        self.conn.execute("
            INSERT INTO game (id, home_id, away_id, date) VALUES 
            (?1, ?2, ?3, ?4)
        ", params![game_id, home_id, away_id, date])
    }
    
    pub fn insert_outcome(&self, game_id: &i64, variant_id: &i64, token: &str) -> Result<usize> {
        self.conn.execute("
            INSERT INTO outcome (game_id, variant_id, token) VALUES 
            (?1, ?2, ?3)
        ", params![game_id, variant_id, token])
    }
    
    pub fn insert_signature(&self, outcome_id: &i64, hex: &str, ) -> Result<usize> {
        self.conn.execute("
            INSERT INTO signature (outcome_id, hex) VALUES 
            (?1, ?2)
        ", params![outcome_id, hex])
    }

    pub fn load_schedule(&self, start_date: Date<Local>, end_date: Date<Local>) -> std::result::Result<(), Box<dyn std::error::Error>> {

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
    
    pub fn load_teams(&self) -> std::result::Result<(), Box<dyn std::error::Error>> {
        let response = get_teams()?.text()?;
        let teams: MlbTeams = serde_json::from_str(&response)?;
        for team in teams.teams {
            self.insert_team(&team.id, &team.name, &team.location)?;
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

