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

pub struct LocalPlayer {
    name: String,
    pkh: PubkeyHash,
}

pub type ByteVec = Vec<u8>;

#[derive(Debug, PartialEq, Clone)]
pub struct PubkeyHash(pub ByteVec);
#[derive(Default, Clone)]
pub struct TgScriptSig(pub Vec<ByteVec>);
#[derive(Clone)]
pub struct ContractSignature(pub Option<Signature>);

#[derive(Clone)]
pub struct Contract {
    pub p1_pkh:             PubkeyHash,
    pub p2_pkh:             PubkeyHash,
    pub arbiter_pkh:        PubkeyHash,
    pub amount:             Amount,
    pub payout_script:      TgScript,
    pub funding_tx:         Transaction,
    pub contract_sig:       ContractSignature,
}

impl Contract {
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
pub struct PayoutRequest {
    pub contract:               Contract,
    pub payout_tx:              Transaction,
    pub payout_script_sig:      TgScriptSig,
}

pub enum PayoutRequestState {
    Unsigned,
    OnePlayerSigned,
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
