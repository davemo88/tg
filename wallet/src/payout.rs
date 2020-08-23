use bitcoin::{
    Amount,
    Transaction,
};
use tglib::{
    Result,
    TgError,
    Contract,
    ContractSignature,
    LocalPlayer,
    Payout,
    TgScriptSig,
    script::TgScript,
};

pub trait PayoutApi {
    fn create_payout(contract: &Contract, payout_tx: Transaction, payout_script_sig: TgScriptSig) -> Payout;
    fn sign_payout(&self, payout: Payout) -> Result<Payout>;
    fn broadcast_payout_tx(&self, payout: Payout ) -> Result<()>;
}
