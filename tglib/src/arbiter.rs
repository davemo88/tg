use serde::{
    Deserialize,
    Serialize,
};
use bdk::bitcoin::{
    hash_types::Txid,
    Address,
    PublicKey,
    secp256k1::Signature,
    util::psbt::PartiallySignedTransaction,
};
use crate::{
    Result,
    contract::{
        Contract,
        ContractRecord,
        PlayerContractInfo,
    },
    payout::{
        Payout,
        PayoutRecord,
    },
    player::PlayerName,
};

pub trait ArbiterService {
    fn get_escrow_pubkey(&self) -> Result<PublicKey>;
    fn get_fee_address(&self) -> Result<Address>;
    fn set_contract_info(&self, info: PlayerContractInfo, pubkey: PublicKey, sig: Signature) -> Result<()>;
    fn get_contract_info(&self, player_name: PlayerName) -> Option<PlayerContractInfo>;
    fn send_contract(&self, contract: ContractRecord, player_name: PlayerName) -> Result<()>;
    fn send_payout(&self, payout: PayoutRecord, player_name: PlayerName) -> Result<()>;
    fn get_auth_token(&self, player_name: &PlayerName) -> Result<Vec<u8>>;
    fn receive_contract(&self, auth: AuthTokenSig) -> Result<Option<ContractRecord>>;
    fn receive_payout(&self, auth: AuthTokenSig) -> Result<Option<PayoutRecord>>;
    fn submit_contract(&self, contract: &Contract) -> Result<Signature>;
    fn submit_payout(&self, payout: &Payout) -> Result<PartiallySignedTransaction>;
// testnet
    fn fund_address(&self, address: Address) -> Result<Txid>;
}

#[derive(Debug, Serialize, Deserialize)]
pub struct SetContractInfoBody {
    pub contract_info: PlayerContractInfo,
    pub pubkey: PublicKey,
    pub sig_hex: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct SendContractBody {
    pub contract: ContractRecord,
    pub player_name: PlayerName,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct SendPayoutBody {
    pub payout: PayoutRecord,
    pub player_name: PlayerName,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct AuthTokenSig {
    pub player_name: PlayerName,
    pub pubkey: PublicKey,
    pub sig_hex: String,
}
