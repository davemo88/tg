use std::{
    path::{
        Path, 
        PathBuf
    },
    fs::remove_file,
};
use rusqlite::{params, Connection, Result};
use tglib::{
    player::PlayerId,
};

#[derive(Debug, Clone)]
pub struct PlayerRecord {
    pub id:             PlayerId,
    pub name:           String,
}

#[derive(Debug, Clone)]
pub struct ContractRecord {
    pub cxid:           String,
    pub p1_id:          PlayerId,
    pub p2_id:          PlayerId,
    pub funding_txid:   String,
    pub hex:            String,
    pub desc:           String,
}

#[derive(Debug, Clone)]
pub struct PayoutRecord {
    pub cxid:           String,
    pub hex:            String,
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
         match self.conn.execute(
            "INSERT INTO player (id, name) VALUES (?1, ?2)",
            params![player.id.0, player.name],
         ) {
             Ok(_) => Ok(()),
             Err(e) => Err(e),
         }
    }

    pub fn insert_contract(&self, contract: ContractRecord) -> Result<()> {
        self.conn.execute(
            "INSERT INTO contract (cxid, p1_id, p2_id, funding_txid, hex, desc) VALUES (?1, ?2, ?3, ?4, ?5, ?6)",
            params![contract.cxid, contract.p1_id.0, contract.p2_id.0, contract.funding_txid, contract.hex, contract.desc],
        )?;
        Ok(())
    }
    pub fn all_contracts(&self) -> Result<Vec<ContractRecord>> {
        let mut stmt = self.conn.prepare("SELECT * FROM contract")?;
        let contract_iter = stmt.query_map(params![], |row| {
            Ok(ContractRecord {
                cxid: row.get(0)?, 
                p1_id: PlayerId(row.get(1)?),
                p2_id: PlayerId(row.get(2)?),
                funding_txid: row.get(3)?,
                hex: row.get(4)?,
                desc: row.get(5)?,
            })
        })?;

        let mut contracts = Vec::<ContractRecord>::new();
        for c in contract_iter {
            contracts.push(c.unwrap());
        }
        Ok(contracts)
    }

    pub fn delete_contract(&self, cxid: String) -> Result<usize> {
        self.conn.execute(
            "DELETE FROM contract WHERE cxid = ?1",
            params![cxid],
        )
    }

    pub fn insert_payout(&self, payout: PayoutRecord) -> Result<()> {
        self.conn.execute(
            "INSERT INTO payout (cxid, hex) VALUES (?1, ?2)",
            params![payout.cxid, payout.hex],
        )?;
        Ok(())
    }

    pub fn all_players(&self) -> Result<Vec<PlayerRecord>> {
        let mut stmt = self.conn.prepare("SELECT id, name FROM player")?;
        let player_iter = stmt.query_map(params![], |row| {
            Ok(PlayerRecord {
                id: PlayerId(row.get(0)?),
                name: row.get(1)?,
            })
        })?;

        let mut players = Vec::<PlayerRecord>::new();
        for p in player_iter {
            players.push(p.unwrap());
        }
        Ok(players)
    }

    pub fn delete_player(&self, id: PlayerId) -> Result<usize> {
        self.conn.execute(
            "DELETE FROM player WHERE id = ?1",
            params![id.0],
        )
    }
}


#[cfg(test)]
mod test {

    use super::*;
    use std::env::current_dir;

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
