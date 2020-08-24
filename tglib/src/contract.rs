use bitcoin::{
    Amount,
    Transaction,
};
use secp256k1::{
    Signature,
};

use crate::player::PlayerId;
use crate::arbiter::ArbiterId;
use crate::script::TgScript;

pub type ContractSignature = Option<Signature>;

#[derive(Clone)]
pub struct Contract {
    p1_id:              PlayerId,
    p2_id:              PlayerId,
    arbiter_id:         ArbiterId,
    amount:             Amount,
pub payout_script:      TgScript,
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


#[derive(Default)]
pub struct ContractBuilder {
    p1_id:              Option<PlayerId>,
    p2_id:              Option<PlayerId>,
    arbiter_id:         Option<ArbiterId>,
    amount:             Option<Amount>,
    payout_script:      Option<TgScript>,
    funding_tx:         Option<Transaction>,
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

    pub fn payout_script(&mut self, script: TgScript) -> &mut Self {
        self.payout_script = Some(script);
        self
    }

    pub fn funding_tx(&mut self, funding_tx: Transaction) -> &mut Self {
        self.funding_tx = Some(funding_tx);
        self
    }

    pub fn build(&self) -> Contract {
        Contract::new(
            self.p1_id.clone().unwrap(),
            self.p2_id.clone().unwrap(),
            self.arbiter_id.clone().unwrap(),
            self.amount.clone().unwrap(),
            self.payout_script.clone().unwrap(),
            self.funding_tx.clone().unwrap(),
        )
    }
}
