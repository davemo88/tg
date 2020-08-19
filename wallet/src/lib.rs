use bitcoin::{
    Amount,
    Transaction,
};
use tglib::{
    Result,
    Contract,
    PubkeyHash,
    LocalPlayer,
    PayoutRequest,
    script::TgScript,
};

pub trait WalletApi {
    fn new_local_player(&self, name: String) -> LocalPlayer;
    fn create_contract(&self,
        p1_pkh:         PubkeyHash,
        p2_pkh:         PubkeyHash,
        arbiter_pkh:    PubkeyHash,
        amount:         Amount,
        payout_script:  TgScript,
        funding_tx:     Option<Transaction>,
    ) -> Contract;
    fn create_payout_request(payout_tx: Transaction, contract: &Contract) -> PayoutRequest;
    fn sign_contract(&self, contract: Contract) -> Option<Contract>;
    fn sign_payout_request(&self, payout_request: PayoutRequest) -> Option<PayoutRequest>;
    fn broadcast_funding_tx(&self, contract: Contract ) -> Result<()>;
    fn broadcast_payout_tx(&self, payout_request: PayoutRequest ) -> Result<()>;
}

#[allow(dead_code)]
struct BitcoinWallet {
    passphrase: String,
}

//impl WalletApi for BitcoinWallet {
//    fn new_local_player(&self, name: String) -> LocalPlayer {
//        LocalPlayer {
//            name,
//// actually need the wallet for this
//            pkh: 
//        }
//    }
//    fn create_contract(&self,
//        p1_pkh:         PubkeyHash,
//        p2_pkh:         PubkeyHash,
//        arbiter_pkh:    PubkeyHash,
//        amount:         Amount,
//        payout_script:  TgScript,
//        funding_tx:     Option<Transaction>,
//    ) -> Contract {
//        Contract {
//            p1_pkh,
//            p2_pkh,
//            arbiter_pkh,
//            amount,
//            funding_tx: funding_tx.unwrap(),
//            payout_script,
//            contract_sig: ContractSignature(None),
//        }
//    }
//    fn create_payout_request(payout_tx: Transaction, contract: &Contract) -> PayoutRequest {}
//    fn sign_contract(&self, contract: Contract) -> Result<Contract> {}
//        Err(TgError("cannot sign contract"))
//    fn sign_payout_request(&self, payout_request: PayoutRequest) -> Result<PayoutRequest>{
//        Err(TgError("cannot sign payout request"))
//    }
//    fn broadcast_funding_tx(&self, contract) -> Result<()> {
//        Ok(())
//    }
//    fn broadcast_payout_tx(&self, payout_request) -> Result<()> {
//        Ok(())
//    }
//}
