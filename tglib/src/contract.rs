use bdk::{
    bitcoin::{
        Address,
        Amount,
        Transaction,
        PublicKey,
        secp256k1::{
            Signature,
        }
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

pub type ContractSignature = Option<Signature>;

#[derive(Clone)]
pub struct Contract {
    pub p1_id:              PlayerId,
    pub p2_id:              PlayerId,
    pub arbiter_id:         ArbiterId,
    pub amount:             Amount,
    pub funding_tx:         Transaction,
    pub payout_script:      TgScript,
    pub contract_sig:       ContractSignature,
}

impl Contract {
    pub fn new(p1_id: PlayerId, p2_id: PlayerId, arbiter_id: ArbiterId, amount: Amount, funding_tx: Transaction, payout_script: TgScript) -> Self {
        Contract {
            p1_id,
            p2_id,
            arbiter_id,
            amount,
            funding_tx,
            payout_script,
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

pub struct PlayerContractInfo {
    pub escrow_pubkey: PublicKey,
    pub payout_address: Address,
    pub change_address: Address,
    pub utxos: Vec<UTXO>,
}
