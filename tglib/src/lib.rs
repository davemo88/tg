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

#[derive(Debug, PartialEq)]
pub struct PubkeyHash(ByteVec);
pub struct ScriptSig(Vec<ByteVec>);
pub struct ContractSignature(Option<Signature>);

pub struct Contract {
    p1_pkh:             PubkeyHash,
    p2_pkh:             PubkeyHash,
    arbiter_pkh:        PubkeyHash,
    amount:             Amount,
    payout_script:      TgScript,
    funding_tx:         Transaction,
    contract_sig:       ContractSignature,
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

pub struct PayoutRequest {
    contract:               Contract,
    payout_tx:              Transaction,
    payout_script_sig:      ScriptSig,
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
