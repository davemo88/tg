use bdk::bitcoin::{
    Address,
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
    mock::{
        NETWORK,
        PAYOUT_VERSION,
    }
};

#[derive(Clone, Debug)]
pub struct Payout {
    pub contract:        Contract,
    pub tx:              Transaction,
    pub script_sig:      Option<Signature>,
    pub version:         u8,
}

impl Payout {
    pub fn new(contract: Contract, tx: Transaction) -> Self {
        Payout {
            contract,
            tx,
            script_sig: None,
            version: PAYOUT_VERSION,
        }
    }

    pub fn from_bytes(bytes: Vec<u8>) -> Result<Payout> {
        Err(TgError("couldn't parse payout"))
    }

    pub fn address(&self) -> Result<Address> {
        let amount = self.contract.amount()?;
        for txout in self.tx.output.clone() {
            if txout.value == amount.as_sat() {
                return Ok(Address::from_script(&txout.script_pubkey, NETWORK).unwrap())
            }
        };
        Err(TgError("couldn't determine payout address"))
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

