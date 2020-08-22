use bitcoin::{
    Amount,
    Transaction,
};
use tglib::{
    Result,
    TgError,
    Contract,
    ContractSignature,
    PubkeyHash,
    LocalPlayer,
    PayoutRequest,
    TgScriptSig,
    script::TgScript,
};

pub trait PayoutRequestApi {
    fn create_payout_request(contract: &Contract, payout_tx: Transaction, payout_script_sig: TgScriptSig) -> PayoutRequest;
    fn sign_payout_request(&self, payout_request: PayoutRequest) -> Result<PayoutRequest>;
    fn broadcast_payout_tx(&self, payout_request: PayoutRequest ) -> Result<()>;
}
