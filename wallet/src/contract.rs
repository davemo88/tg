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

pub struct ContractMetaDetails {
    title: String,
    note: String,
}

pub trait ContractApi {
    fn create_contract(&self,
        p1_pkh:         PubkeyHash,
        p2_pkh:         PubkeyHash,
        arbiter_pkh:    PubkeyHash,
        amount:         Amount,
        payout_script:  TgScript,
        funding_tx:     Option<Transaction>,
    ) -> Contract;
    fn sign_contract(&self, contract: Contract) -> Result<Contract>;
    fn broadcast_funding_tx(&self, contract: Contract ) -> Result<()>;
}
