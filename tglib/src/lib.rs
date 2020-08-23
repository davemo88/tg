use std::{
    fmt,
};
use bitcoin::{
    Address,
    Amount,
    Transaction,
    util::key::{
        PublicKey,
        PrivateKey,
    },
    hashes::{
        Hash,
        HashEngine,
        sha256::HashEngine as Sha2Engine,
        sha256::Hash as Sha2Hash,
    },
};
use secp256k1::{
    Signature,
};

pub mod script;
use script::TgScript;

pub type ByteVec = Vec<u8>;
pub type PlayerId = String;
pub type ArbiterId = String;

#[derive(Default, Clone)]
pub struct TgScriptSig(pub Vec<ByteVec>);

pub struct Player {
    name:   String,
    id:     PlayerId,
}

pub type ContractSignature = Option<Signature>;

#[derive(Clone)]
pub struct Contract {
    p1_id:              PlayerId,
    p2_id:              PlayerId,
    arbiter_id:         ArbiterId,
    amount:             Amount,
    payout_script:      TgScript,
    funding_tx:         Transaction,
    contract_sig:       ContractSignature,
}

impl Contract {
    pub fn new(p1_id: PlayerId, p2_id: PlayerId, arbiter_id: ArbiterId, amount: Amount, payout_script: TgScript, funding_tx: Transaction) -> Self {
        Contract {
            p1_id,
            p2_id,
            arbiter_id,
            amount,
            payout_script,
            funding_tx,
            contract_sig: None,
        }
    }
    pub fn state(&self) -> ContractState {
        return ContractState::Invalid
    }
}

#[derive(Debug, PartialEq)]
pub enum ContractState {
    Unsigned,
    P1Signed,
    P2Signed,
    ArbiterSigned,
    Live,
    Resolved,
    Invalid,
}

#[derive(Clone)]
pub struct Payout {
    contract:               Contract,
    tx:              Transaction,
    script_sig:      TgScriptSig,
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

#[derive(Debug)]
pub struct TgError(pub &'static str);

impl fmt::Display for TgError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "TgError: {}", self.0)
    }
}

pub type Result<T> = std::result::Result<T, TgError>;

#[cfg(test)]
mod tests {
    #[test]
    fn it_works() {
        assert_eq!(2 + 2, 4);
    }
}
