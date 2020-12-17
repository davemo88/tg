use std::convert::From;
use rusqlite::{params, Connection, Result};
use serde::{
    Serialize,
    Deserialize,
};
use tglib::{
    bdk::bitcoin::consensus,
    hex,
    player::PlayerName,
    payout::Payout,
};

#[derive(Debug, Clone)]
pub struct PlayerRecord {
    pub name:       PlayerName,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ContractRecord {
    pub cxid:           String,
    pub p1_name:        PlayerName,
    pub p2_name:        PlayerName,
    pub hex:            String,
    pub desc:           String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PayoutRecord {
    pub cxid:           String,
    pub psbt:           String,
    pub sig:            String,
}

impl From<Payout> for PayoutRecord {
    fn from(p: Payout) -> PayoutRecord {
        let sig = match p.script_sig {
           Some(sig) => hex::encode(sig.serialize_compact().to_vec()),
           None => "".to_string(),
        };
        PayoutRecord {
            cxid: hex::encode(p.contract.cxid()),
            psbt: hex::encode(consensus::serialize(&p.psbt)),
            sig,
        }
    }
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
                    name              TEXT PRIMARY KEY
                );
                CREATE TABLE IF NOT EXISTS contract (
                    cxid            TEXT PRIMARY KEY,
                    p1_name         TEXT NOT NULL,
                    p2_name         TEXT NOT NULL,
                    hex             TEXT NOT NULL,
                    desc            TEXT,
                    FOREIGN KEY(p1_name) REFERENCES player(name),
                    FOREIGN KEY(p2_name) REFERENCES player(name)
                );
                CREATE TABLE IF NOT EXISTS payout (
                    cxid            TEXT PRIMARY KEY,
                    psbt            TEXT NOT NULL,
                    sig             TEXT NOT NULL,
                    FOREIGN KEY(cxid) REFERENCES contract(cxid)
                );
            COMMIT;"
        )
    }

    pub fn insert_player(&self, player: PlayerRecord) -> Result<usize> {
         self.conn.execute(
            "INSERT INTO player (name) VALUES (?1)",
            params![player.name.0],
         )
    }

    pub fn insert_contract(&self, contract: ContractRecord) -> Result<usize> {
        self.conn.execute(
            "INSERT INTO contract (cxid, p1_name, p2_name, hex, desc) VALUES (?1, ?2, ?3, ?4, ?5) 
            ON CONFLICT (cxid) DO UPDATE SET p1_name=?2, p2_name=?3, hex=?4, desc=?5",
            params![contract.cxid, contract.p1_name.0, contract.p2_name.0, contract.hex, contract.desc],
        )
    }

    pub fn all_players(&self) -> Result<Vec<PlayerRecord>> {
        let mut stmt = self.conn.prepare("SELECT name FROM player")?;
        let player_iter = stmt.query_map(params![], |row| {
            Ok(PlayerRecord {
                name: PlayerName(row.get(0)?),
            })
        })?;

        let mut players = Vec::<PlayerRecord>::new();
        for p in player_iter {
            players.push(p.unwrap());
        }
        Ok(players)
    }

    pub fn delete_player(&self, name: PlayerName) -> Result<usize> {
        self.conn.execute(
            "DELETE FROM player WHERE name = ?1",
            params![name.0],
        )
    }

    pub fn all_contracts(&self) -> Result<Vec<ContractRecord>> {
        let mut stmt = self.conn.prepare("SELECT * FROM contract")?;
        let contract_iter = stmt.query_map(params![], |row| {
            Ok(ContractRecord {
                cxid: row.get(0)?, 
                p1_name: PlayerName(row.get(1)?),
                p2_name: PlayerName(row.get(2)?),
                hex: row.get(3)?,
                desc: row.get(4)?,
            })
        })?;

        let mut contracts = Vec::<ContractRecord>::new();
        for c in contract_iter {
            contracts.push(c.unwrap());
        }
        Ok(contracts)
    }

    pub fn get_contract(&self, cxid: &str) -> Option<ContractRecord> {
        let mut stmt = self.conn.prepare("SELECT * FROM contract where cxid = ?1").unwrap();
        let mut contract_iter = stmt.query_map(params![cxid], |row| {
            Ok(ContractRecord {
                cxid: row.get(0)?, 
                p1_name: PlayerName(row.get(1)?),
                p2_name: PlayerName(row.get(2)?),
                hex: row.get(3)?,
                desc: row.get(4)?,
            })
        }).unwrap();
        if let Some(cr) = contract_iter.next() {
            Some(cr.unwrap())
        } else {
            None
        }
    }

    pub fn delete_contract(&self, cxid: String) -> Result<usize> {
        self.conn.execute(
            "DELETE FROM contract WHERE cxid = ?1",
            params![cxid],
        )
    }

    pub fn add_signature(&self, cxid: String, hex: String) -> Result<usize> {
// TODO: validation
        self.conn.execute(
            "UPDATE contract SET hex = ?1 WHERE cxid = ?2",
            params![hex, cxid],
        )
    }

    pub fn insert_payout(&self, payout: PayoutRecord) -> Result<usize> {
        self.conn.execute(
            "INSERT INTO payout (cxid, psbt, sig) VALUES (?1, ?2, ?3) ON CONFLICT(cxid) DO UPDATE SET
            psbt=?2, sig=?3",
            params![payout.cxid, payout.psbt, payout.sig],
        )
    }

    pub fn all_payouts(&self) -> Result<Vec<PayoutRecord>> {
        let mut stmt = self.conn.prepare("SELECT * FROM payout")?;
        let payout_iter = stmt.query_map(params![], |row| {
            Ok(PayoutRecord {
                cxid: row.get(0)?,
                psbt: row.get(1)?,
                sig: row.get(2)?,
            })
        })?;

        let mut payouts = Vec::<PayoutRecord>::new();
        for p in payout_iter {
            payouts.push(p.unwrap());
        }
        Ok(payouts)
    }

    pub fn get_payout(&self, cxid: &str) -> Option<PayoutRecord> {
        let mut stmt = self.conn.prepare("SELECT * FROM payout where cxid = ?1").unwrap();
        let mut payout_iter = stmt.query_map(params![cxid], |row| {
            Ok(PayoutRecord {
                cxid: row.get(0)?, 
                psbt: row.get(1)?,
                sig: row.get(2)?,
            })
        }).unwrap();
        if let Some(pr) = payout_iter.next() {
            Some(pr.unwrap())
        } else {
            None
        }
    }

    pub fn delete_payout(&self, cxid: String) -> Result<usize> {
        self.conn.execute(
            "DELETE FROM payout WHERE cxid = ?1",
            params![cxid],
        )
    }
}


#[cfg(test)]
mod test {

    use super::*;
    use std::{
        env::current_dir,
        fs::remove_file,
        path::PathBuf,
    };

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
