use std::{
    path::{
        Path, 
        PathBuf
    },
    env::current_dir,
    fs::remove_file,
};
use rusqlite::{params, Connection, Result};
use tglib::{
    player::PlayerId,
};

#[derive(Debug)]

pub struct PlayerRecord {
    id:             PlayerId,
    name:           String,
}

pub struct ContractRecord {
    cxid:           String,
    p1_id:          PlayerId,
    p2_id:          PlayerId,
    funding_txid:   String,
    hex:            String,
    desc:           String,
}

pub struct PayoutRecord {
    cxid:           String,
    hex:            String,
}

pub struct DB {
    pub conn: Connection,
}

impl DB {
    pub fn new(path: &std::path::Path) -> Result<DB> {
        Ok(DB { conn: Connection::open(path)? })
    }

    pub fn create_tables(&self) -> Result<()> {
        self.conn.execute_batch(
            "BEGIN;
                CREATE TABLE IF NOT EXISTS player (
                    id              TEXT PRIMARY KEY,
                    name            TEXT
                );
                CREATE TABLE IF NOT EXISTS contract (
                    cxid            TEXT PRIMARY KEY,
                    p1_id           TEXT NOT NULL,
                    p2_id           TEXT NOT NULL,
                    funding_txid    TEXT NOT NULL UNIQUE,
                    hex             TEXT NOT NULL UNIQUE,
                    desc            TEXT,
                    FOREIGN KEY(p1_id) REFERENCES player(id),
                    FOREIGN KEY(p2_id) REFERENCES player(id)
                );
                CREATE TABLE IF NOT EXISTS payout (
                    cxid            TEXT PRIMARY KEY,
                    hex             TEXT NOT NULL UNIQUE,
                    FOREIGN KEY(cxid) REFERENCES contract(cxid)
                );
            COMMIT;"
        )
    }

    pub fn insert_player(&self, player: PlayerRecord) -> Result<()> {
        self.conn.execute(
            "INSERT INTO player (id, name) VALUES (?1, ?2)",
            params![player.id.0, player.name],
        )?;
        Ok(())
    }

    pub fn insert_contract(&self, contract: ContractRecord) -> Result<()> {
        self.conn.execute(
            "INSERT INTO contract (cxid, p1_id, p2_id, funding_txid, hex, desc) VALUES (?1, ?2, ?3, ?4, ?5, ?6)",
            params![contract.cxid, contract.p1_id.0, contract.p2_id.0, contract.funding_txid, contract.hex, contract.desc],
        )?;
        Ok(())
    }

    pub fn insert_payout(&self, payout: PayoutRecord) -> Result<()> {
        self.conn.execute(
            "INSERT INTO payout (cxid, hex) VALUES (?1, ?2)",
            params![payout.cxid, payout.hex],
        )?;
        Ok(())
    }
}


#[cfg(test)]
mod test {

    use super::*;

    #[test]
    fn test_create_tables() -> Result<()> {
        let mut db_path: PathBuf = current_dir().unwrap();
        db_path.push("test_create_tables.db");
        let db = DB::new(&db_path)?;
        match db.create_tables() {
            Ok(()) => (),
            Err(e) => println!("{}",e),
        }
        db.conn.close().unwrap();
        remove_file(db_path).unwrap();
        Ok(())
    }
}
