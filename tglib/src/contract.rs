use std::convert::AsRef;
use bdk::{
    bitcoin::{
        Address,
        Amount,
        Transaction,
        PublicKey,
        secp256k1::{
            Signature,
        },
        hashes::{
            Hash,
            HashEngine,
            sha256::Hash as ShaHash,
            sha256::HashEngine as ShaHashEngine,
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

#[derive(Clone, Debug)]
pub struct Contract {
    pub p1_pubkey:          PublicKey,
    pub p2_pubkey:          PublicKey,
    pub amount:             Amount,
    pub funding_tx:         Transaction,
    pub payout_script:      TgScript,
    pub sigs:               Vec<Signature>, 
}

impl Contract {
    pub fn new(p1_pubkey: PublicKey, p2_pubkey: PublicKey, amount: Amount, funding_tx: Transaction, payout_script: TgScript) -> Self {
        Contract {
            p1_pubkey,
            p2_pubkey,
            amount,
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
