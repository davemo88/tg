use std::str::FromStr;
use reqwest;
use tglib::{
    bdk::{
        bitcoin::{
            Address,
            PublicKey,
            consensus,
            secp256k1::Signature,
            util::psbt::PartiallySignedTransaction,
        },
    },
    hex,
    Result,
    TgError,
    arbiter::ArbiterService,
    contract::Contract,
    payout::Payout,
};


pub struct ArbiterClient(String);

impl ArbiterClient {
    pub fn new (host: &str) -> Self {
        ArbiterClient(String::from(host))
    }

    fn get(&self, command: &str, params: Option<&str>) -> reqwest::Result<reqwest::blocking::Response> {
        let mut url = format!("{}/{}", self.0, command);
        if let Some(params) = params {
            url += &format!("/{}",params);
        }
        reqwest::blocking::get(&url)
    }
}

impl ArbiterService for ArbiterClient {
    fn get_escrow_pubkey(&self) -> Result<PublicKey> {
        match self.get("escrow-pubkey", None) {
            Ok(response) => Ok(PublicKey::from_str(&response.text().unwrap()).unwrap()),
            Err(_) => Err(TgError("couldn't get result pubkey")),
        }
    }

    fn get_fee_address(&self) -> Result<Address> {
        match self.get("fee-address", None) {
            Ok(response) => Ok(Address::from_str(&response.text().unwrap()).unwrap()),
            Err(_) => Err(TgError("couldn't get fee address")),
        }
    }

    fn submit_contract(&self, contract: &Contract) -> Result<Signature> {
        match self.get("submit-contract", Some(&hex::encode(contract.to_bytes()))) {
            Ok(response) => match Signature::from_compact(&hex::decode(response.text().unwrap()).unwrap()) {
                Ok(sig) => Ok(sig),
                Err(_) => Err(TgError("invalid contract"))
            }
            Err(_) => Err(TgError("couldn't submit contract"))
        }
    }

    fn submit_payout(&self, payout: &Payout) -> Result<PartiallySignedTransaction> {
        match self.get("submit-payout", Some(&hex::encode(payout.to_bytes()))) {
            Ok(response) => match consensus::deserialize(&hex::decode(response.text().unwrap()).unwrap()) {
                Ok(psbt) => Ok(psbt),
                Err(_) => Err(TgError("invalid payout")),
            }
            Err(_) => Err(TgError("couldn't submit payout")),
        }
    }
}
