use bdk::bitcoin::{
    Amount,
    Transaction,
    secp256k1::{
        Secp256k1,
        Message,
        Signature,
        All,
    },
};
use crate::{
    arbiter::ArbiterId,
    contract::Contract,
    payout::Payout,
    player::PlayerId,
    script::TgScript,
    TgScriptSig,
    Result as TgResult,
};

pub trait Creation {
    fn create_contract(&self,
//        p1_id:         PlayerId,
        p2_id:         PlayerId,
//        arbiter_id:    ArbiterId,
        amount:         Amount,
//        payout_script:  TgScript,
//        funding_tx:     Option<Transaction>,
    ) -> Contract;

    fn create_payout(&self,
        contract:           &Contract, 
        payout_tx:          Transaction, 
        payout_script_sig:  TgScriptSig
    ) -> Payout;
}

pub trait Signing {
    fn sign_contract(&self, contract: Contract) -> TgResult<Contract>;
    fn sign_payout(&self, payout: Payout) -> TgResult<Payout>;
}

