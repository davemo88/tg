use bitcoin::{
    Amount,
    Transaction,
};
use bitcoin_wallet::{
    account::MasterAccount,
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

pub trait LocalPlayerWallet {
    fn new_local_player(&self, name: String) -> LocalPlayer;
    fn load_local_player(&self, name: String) -> LocalPlayer;
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

pub trait PayoutRequestApi {
    fn create_payout_request(contract: &Contract, payout_tx: Transaction, payout_script_sig: TgScriptSig) -> PayoutRequest;
    fn sign_payout_request(&self, payout_request: PayoutRequest) -> Result<PayoutRequest>;
    fn broadcast_payout_tx(&self, payout_request: PayoutRequest ) -> Result<()>;
}

pub struct MyWallet(MasterAccount);

impl ContractApi for MyWallet {
    fn create_contract(&self,
        p1_pkh:         PubkeyHash,
        p2_pkh:         PubkeyHash,
        arbiter_pkh:    PubkeyHash,
        amount:         Amount,
        payout_script:  TgScript,
        funding_tx:     Option<Transaction>,
    ) -> Contract {
        Contract {
            p1_pkh,
            p2_pkh,
            arbiter_pkh,
            amount,
            funding_tx: funding_tx.unwrap(),
            payout_script,
            contract_sig: ContractSignature(None),
        }
    }
    fn sign_contract(&self, _contract: Contract) -> Result<Contract> {
        Err(TgError("cannot sign contract"))
    }
    fn broadcast_funding_tx(&self, _contract: Contract) -> Result<()> {
        Err(TgError("cannot broadcast funding tx"))
    }
}

impl PayoutRequestApi for MyWallet {
    fn create_payout_request(contract: &Contract, payout_tx: Transaction, payout_script_sig: TgScriptSig) -> PayoutRequest {
        PayoutRequest {
            payout_tx,
            contract: contract.clone(),
            payout_script_sig,
        }
    }
    fn sign_payout_request(&self, _payout_request: PayoutRequest) -> Result<PayoutRequest>{
        Err(TgError("cannot sign payout request"))
    }
    fn broadcast_payout_tx(&self, _payout_request: PayoutRequest) -> Result<()> {
        Err(TgError("cannot broadcast payout tx"))
    }
}
