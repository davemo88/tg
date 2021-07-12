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
    contract::Contract,
    payout::Payout,
};

pub type Result<T> = std::result::Result<T, Box<dyn std::error::Error>>;

pub trait ArbiterService {
    fn get_escrow_pubkey(&self) -> Result<PublicKey>;
    fn get_fee_address(&self) -> Result<Address>;
    fn submit_contract(&self, contract: &Contract) -> Result<Signature>;
    fn submit_payout(&self, payout: &Payout) -> Result<PartiallySignedTransaction>;
// testnet
    fn fund_address(&self, address: Address) -> Result<Txid>;
}

#[derive(Debug, Serialize, Deserialize)]
pub struct SubmitContractBody {
    pub contract_hex: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct SubmitPayoutBody {
    pub payout_hex: String,
}
