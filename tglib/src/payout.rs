use bdk::bitcoin::{
    Transaction,
};
use crate::{
    TgScriptSig,
    contract::{
        Contract,
    },
};

#[derive(Clone)]
pub struct Payout {
    pub contract:        Contract,
    pub tx:              Transaction,
    pub script_sig:      TgScriptSig,
}

impl Payout {
    pub fn new(contract: &Contract, tx: Transaction, script_sig: TgScriptSig) -> Self {
        Payout {
            contract: contract.clone(),
            tx,
            script_sig,
        }
    }
}

pub enum PayoutState {
    Unsigned,
    ArbiterSigned,
    Live,
    Resolved,
    Invalid,
}

