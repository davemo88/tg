use std::convert::AsRef;
use serde::{Serialize, Deserialize,};
use bdk::{
    bitcoin::{
        Address,
        Amount,
        Transaction,
        PublicKey,
        consensus::serialize,
        hashes::{
            Hash,
            HashEngine,
            sha256::Hash as ShaHash,
            sha256::HashEngine as ShaHashEngine,
        },
        secp256k1::{
            Signature,
        },
    },
    UTXO,
};

use crate::{
    Result,
    TgError,
    arbiter::ArbiterId,
    player::PlayerId,
    script::TgScript,
};

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Contract {
    pub p1_pubkey:          PublicKey,
    pub p2_pubkey:          PublicKey,
    pub arbiter_pubkey:     PublicKey,
    pub funding_tx:         Transaction,
    pub payout_script:      TgScript,
    pub sigs:               Vec<Signature>, 
}

impl Contract {
    pub fn new(p1_pubkey: PublicKey, p2_pubkey: PublicKey, arbiter_pubkey: PublicKey, funding_tx: Transaction, payout_script: TgScript) -> Self {
        Contract {
            p1_pubkey,
            p2_pubkey,
            arbiter_pubkey,
            funding_tx,
            payout_script,
            sigs: Vec::new(),
        }
    }

    pub fn cxid(&self) -> Vec<u8> {
        let mut engine = ShaHashEngine::default();
        engine.input(&Vec::from(self.payout_script.clone()));
        let hash: &[u8] = &ShaHash::from_engine(engine);
        hash.to_vec()
    }

    pub fn state(&self) -> ContractState {
        return ContractState::Invalid
    }
}

// should implement encode / decode?
impl Into<Vec<u8>> for Contract {
    fn into(self) -> Vec<u8> {
        let mut v: Vec<u8> = Vec::new();
        v.extend(self.p1_pubkey.to_bytes());
        v.extend(self.p2_pubkey.to_bytes());
        v.extend(self.arbiter_pubkey.to_bytes());
        v.extend(serialize(&self.funding_tx));
        v.extend(Vec::from(self.payout_script));
        v
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

pub struct PlayerContractInfo {
    pub escrow_pubkey: PublicKey,
    pub change_address: Address,
    pub utxos: Vec<UTXO>,
}
