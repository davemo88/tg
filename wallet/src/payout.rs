use bitcoin::{
    Amount,
    Transaction,
};
use tglib::{
    Result,
    TgError,
    TgScriptSig,
    arbiter::{
        ArbiterId,
    },
    contract::{
        Contract,
        ContractSignature,
    },
    payout::{
        Payout,
    },
    player::{
        PlayerId,
    },
    script::TgScript,
};

pub trait PayoutApi {
    fn create_payout(contract: &Contract, payout_tx: Transaction, payout_script_sig: TgScriptSig) -> Payout;
    fn sign_payout(&self, payout: Payout) -> Result<Payout>;
    fn broadcast_payout_tx(&self, payout: Payout ) -> Result<()>;
}
