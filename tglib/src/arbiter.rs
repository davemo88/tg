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
    contract::{
        Contract,
        PlayerContractInfo,
    },
    payout::{
        Payout,
        PayoutRecord,
    },
    player::PlayerName,
};

pub type Result<T> = std::result::Result<T, Box<dyn std::error::Error>>;

//pub type Result<T> = std::result::Result<T, Error>;
//
//#[derive(Debug)]
//#[allow(dead_code)]
//pub enum Error {
//    Adhoc(&'static str),
//    Reqwest(reqwest::Error),
//    Tglib(crate::Error),
//} 
//
//impl std::fmt::Display for Error {
//    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
//        match self {
//            Error::Adhoc(message) => write!(f, "Adhoc({})", message),
//            Error::Reqwest(error) => write!(f, "Reqwest({})", error),
//            Error::Tglib(error) => write!(f, "Tglib({})", error),
//        }
//    }
//}
//
//impl std::error::Error for Error {
//    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
//        match self {
//            Error::Adhoc(_) => None,
//            Error::Reqwest(error) => Some(error),
//            Error::Tglib(error) => Some(error),
//        }
//    }
//}
//
//impl From<crate::Error> for Error {
//    fn from(error: crate::Error) -> Self {
//        Error::Tglib(error)
//    }
//}
//
//impl From<reqwest::Error> for Error {
//    fn from(error: reqwest::Error) -> Self {
//        Error::Reqwest(error)
//    }
//}

pub trait ArbiterService {
    fn get_escrow_pubkey(&self) -> Result<PublicKey>;
    fn get_fee_address(&self) -> Result<Address>;
    fn set_contract_info(&self, info: PlayerContractInfo, pubkey: PublicKey, sig: Signature) -> Result<()>;
    fn get_contract_info(&self, player_name: PlayerName) -> Result<Option<PlayerContractInfo>>;
//    fn send_contract(&self, contract: ContractRecord, player_name: PlayerName) -> Result<()>;
//    fn send_payout(&self, payout: PayoutRecord, player_name: PlayerName) -> Result<()>;
//    fn get_auth_token(&self, player_name: &PlayerName) -> Result<Vec<u8>>;
//    fn receive_contract(&self, auth: AuthTokenSig) -> Result<Option<ContractRecord>>;
//    fn receive_payout(&self, auth: AuthTokenSig) -> Result<Option<PayoutRecord>>;
    fn submit_contract(&self, contract: &Contract) -> Result<Signature>;
    fn submit_payout(&self, payout: &Payout) -> Result<PartiallySignedTransaction>;
// testnet
    fn fund_address(&self, address: Address) -> Result<Txid>;
}

// TODO: should this use auth token sig scheme? probably
#[derive(Debug, Serialize, Deserialize)]
pub struct SetContractInfoBody {
    pub contract_info: PlayerContractInfo,
    pub pubkey: PublicKey,
    pub sig_hex: String,
}

//#[derive(Debug, Serialize, Deserialize)]
//pub struct SendContractBody {
//    pub contract: ContractRecord,
//    pub player_name: PlayerName,
//}
//
//#[derive(Debug, Serialize, Deserialize)]
//pub struct SendPayoutBody {
//    pub payout: PayoutRecord,
//    pub player_name: PlayerName,
//}

#[derive(Debug, Serialize, Deserialize)]
pub struct SubmitContractBody {
    pub contract_hex: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct SubmitPayoutBody {
    pub payout_hex: String,
}

//#[derive(Clone, Debug, Serialize, Deserialize)]
//pub struct AuthTokenSig {
//    pub player_name: PlayerName,
//    pub pubkey: PublicKey,
//    pub sig_hex: String,
//}
