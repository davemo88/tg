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
}

impl ArbiterService for ArbiterClient {
    fn get_escrow_pubkey(&self) -> Result<PublicKey> {
        let response = reqwest::blocking::get(&format!("{}/escrow-pubkey", self.0)).unwrap();
        let body = String::from(response.text().unwrap());
        Ok(PublicKey::from_str(&body).unwrap())
    }

    fn get_fee_address(&self) -> Result<Address> {
        let response = reqwest::blocking::get(&format!("{}/fee-address", self.0)).unwrap();
        let body = String::from(response.text().unwrap());
        Ok(Address::from_str(&body).unwrap())
    }

    fn submit_contract(&self, contract: &Contract) -> Result<Signature> {
        let response = reqwest::blocking::get(&format!("{}/submit-contract/{}", self.0, hex::encode(contract.to_bytes()))).unwrap();
        let body = String::from(response.text().unwrap());
        if let Ok(sig) = Signature::from_compact(&hex::decode(body).unwrap()) {
            Ok(sig)
        } else {
            Err(TgError("invalid contract"))
        }
    }

    fn submit_payout(&self, payout: &Payout) -> Result<PartiallySignedTransaction> {
        let response = reqwest::blocking::get(&format!("{}/submit-payout/{}", self.0, hex::encode(payout.to_bytes()))).unwrap();
        let body = String::from(response.text().unwrap());
        if let Ok(psbt) = consensus::deserialize(&hex::decode(body).unwrap()) {
            Ok(psbt)
        } else {
            Err(TgError("invalid payout"))
        }
    }
}
