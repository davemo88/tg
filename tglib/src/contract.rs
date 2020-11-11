use bdk::bitcoin::{
    Amount,
    Transaction,
    secp256k1::{
    Signature,
    }
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

// unnecessary
// but could become worthwhile, e.g. if contract fields lose pub
// e.g. make contract with new but only allow certain mutations like signing
#[derive(Debug, Default)]
pub struct ContractBuilder {
    p1_id:              Option<PlayerId>,
    p2_id:              Option<PlayerId>,
    arbiter_id:         Option<ArbiterId>,
    amount:             Option<Amount>,
    funding_tx:         Option<Transaction>,
    payout_script:      Option<TgScript>,
}

impl ContractBuilder {
    pub fn p1_id(&mut self, player_id: PlayerId) -> &mut Self {
        self.p1_id = Some(player_id);
        self
    }

    pub fn p2_id(&mut self, player_id: PlayerId) -> &mut Self {
        self.p2_id = Some(player_id);
        self
    }

    pub fn arbiter_id(&mut self, arbiter_id: ArbiterId) -> &mut Self {
        self.arbiter_id = Some(arbiter_id);
        self
    }

    pub fn amount(&mut self, amount: Amount) -> &mut Self {
        self.amount = Some(amount);
        self
    }

    pub fn funding_tx(&mut self, funding_tx: Transaction) -> &mut Self {
        self.funding_tx = Some(funding_tx);
        self
    }

    pub fn payout_script(&mut self, script: TgScript) -> &mut Self {
        self.payout_script = Some(script);
        self
    }

    pub fn build(&self) -> Contract {

        Contract::new(
            self.p1_id.clone().unwrap(),
            self.p2_id.clone().unwrap(),
            self.arbiter_id.clone().unwrap(),
            self.amount.clone().unwrap(),
            self.funding_tx.clone().unwrap(),
            self.payout_script.clone().unwrap(),
        )
    }

    pub fn generate_funding_tx(&self) -> Result<Transaction> {
        Err(TgError("couldn't generate transaction"))
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
