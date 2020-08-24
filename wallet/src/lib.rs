use bitcoin::{
    Amount,
    Transaction,
};
use bitcoin_wallet::{
    account::{
        MasterAccount,
        Seed,
    },
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

mod contract;
mod payout;

use contract::{
    ContractApi,
};
use payout::{
    PayoutApi,
};

pub struct PlayerWallet(MasterAccount);

impl ContractApi for PlayerWallet {

    fn create_contract(&self,
        p1_id:         PlayerId,
        p2_id:         PlayerId,
        arbiter_id:    ArbiterId,
        amount:         Amount,
        payout_script:  TgScript,
        funding_tx:     Option<Transaction>,
    ) -> Contract {
        Contract::new(
            p1_id, 
            p2_id, 
            arbiter_id, 
            amount,
            payout_script,
            funding_tx.unwrap(),
        )
    }
    fn sign_contract(&self, _contract: Contract) -> Result<Contract> {
        Err(TgError("cannot sign contract"))
    }
    fn broadcast_funding_tx(&self, _contract: Contract) -> Result<()> {
        Err(TgError("cannot broadcast funding tx"))
    }
}

impl PayoutApi for PlayerWallet {
    fn create_payout(contract: &Contract, payout_tx: Transaction, payout_script_sig: TgScriptSig) -> Payout {
        Payout::new(
            &contract,
            payout_tx,
            payout_script_sig,
        )
    }
    fn sign_payout(&self, _payout: Payout) -> Result<Payout>{
        Err(TgError("cannot sign payout request"))
    }
    fn broadcast_payout_tx(&self, _payout: Payout) -> Result<()> {
        Err(TgError("cannot broadcast payout tx"))
    }
}
