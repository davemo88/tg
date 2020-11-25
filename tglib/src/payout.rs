use bdk::bitcoin::{
    Transaction,
    secp256k1::{
        Signature,
    }
};
use crate::{
    contract::{
        Contract,
    },
};

#[derive(Clone)]
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
}

pub enum PayoutState {
    Unsigned,
    PlayerSigned,
    ArbiterSigned,
    Live,
    Resolved,
    Invalid,
}

