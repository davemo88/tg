use std::convert::From;
use bdk::bitcoin::consensus;
use rusqlite::{params, Connection, Result};
use tglib::{
    player::PlayerId,
    payout::Payout,
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
    pub hex:            String,
    pub desc:           String,
}

#[derive(Debug, Clone)]
pub struct PayoutRecord {
    pub cxid:           String,
    pub tx:             String,
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
            tx: hex::encode(consensus::serialize(&p.tx)),
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
                    id              TEXT PRIMARY KEY,
                    name            TEXT
                );
                CREATE TABLE IF NOT EXISTS contract (
                    cxid            TEXT PRIMARY KEY,
                    p1_id           TEXT NOT NULL,
                    p2_id           TEXT NOT NULL,
                    hex             TEXT NOT NULL UNIQUE,
                    desc            TEXT,
                    FOREIGN KEY(p1_id) REFERENCES player(id),
                    FOREIGN KEY(p2_id) REFERENCES player(id)
                );
                CREATE TABLE IF NOT EXISTS payout (
                    cxid            TEXT PRIMARY KEY,
                    tx              TEXT NOT NULL UNIQUE,
                    sig             TEXT NOT NULL UNIQUE,
                    FOREIGN KEY(cxid) REFERENCES contract(cxid)
                );
            COMMIT;"
        )
    }

    pub fn insert_player(&self, player: PlayerRecord) -> Result<usize> {
         self.conn.execute(
            "INSERT INTO player (id, name) VALUES (?1, ?2)",
            params![player.id.0, player.name],
         )
    }

    pub fn insert_contract(&self, contract: ContractRecord) -> Result<usize> {
        self.conn.execute(
            "INSERT INTO contract (cxid, p1_id, p2_id, hex, desc) VALUES (?1, ?2, ?3, ?4, ?5)",
            params![contract.cxid, contract.p1_id.0, contract.p2_id.0, contract.hex, contract.desc],
        )
    }

    pub fn all_players(&self) -> Result<Vec<PlayerRecord>> {
        let mut stmt = self.conn.prepare("SELECT * FROM player")?;
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

    pub fn all_contracts(&self) -> Result<Vec<ContractRecord>> {
        let mut stmt = self.conn.prepare("SELECT * FROM contract")?;
        let contract_iter = stmt.query_map(params![], |row| {
            Ok(ContractRecord {
                cxid: row.get(0)?, 
                p1_id: PlayerId(row.get(1)?),
                p2_id: PlayerId(row.get(2)?),
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
                p1_id: PlayerId(row.get(1)?),
                p2_id: PlayerId(row.get(2)?),
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
            "INSERT INTO payout (cxid, tx, sig) VALUES (?1, ?2, ?3) ON CONFLICT(cxid) DO UPDATE SET
            tx=?2, sig=?3",
            params![payout.cxid, payout.tx, payout.sig],
        )
    }

    pub fn all_payouts(&self) -> Result<Vec<PayoutRecord>> {
        let mut stmt = self.conn.prepare("SELECT * FROM payout")?;
        let payout_iter = stmt.query_map(params![], |row| {
            Ok(PayoutRecord {
                cxid: row.get(0)?,
                tx: row.get(1)?,
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
                tx: row.get(1)?,
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
