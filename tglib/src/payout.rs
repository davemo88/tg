use bdk::bitcoin::{
    Transaction,
    secp256k1::{
        Signature,
    }
};
use crate::{
    Result,
    TgError,
    contract::{
        Contract,
    },
};

#[derive(Clone, Debug)]
pub struct Payout {
    pub contract:        Contract,
    pub tx:              Transaction,
    pub script_sig:      Option<Signature>,
}

impl Payout {
    pub fn new(contract: Contract, tx: Transaction) -> Self {
        Payout {
            contract,
            tx,
            script_sig: None,
        }
    }

    pub fn from_bytes(bytes: Vec<u8>) -> Result<Payout> {
        Err(TgError("couldn't parse payout"))
    }
}

pub enum PayoutState {
    Unsigned,
    PlayerSigned,
    ArbiterSigned,
    Live,
    Resolved,
    Invalid,
}

