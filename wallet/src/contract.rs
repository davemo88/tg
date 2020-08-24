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

pub struct ContractMetaDetails {
    title: String,
    note: String,
}

pub trait ContractApi {
    fn create_contract(&self,
        p1_pkh:         PlayerId,
        p2_pkh:         PlayerId,
        arbiter_pkh:    ArbiterId,
        amount:         Amount,
        payout_script:  TgScript,
        funding_tx:     Option<Transaction>,
    ) -> Contract;
    fn sign_contract(&self, contract: Contract) -> Result<Contract>;
    fn broadcast_funding_tx(&self, contract: Contract ) -> Result<()>;
}
