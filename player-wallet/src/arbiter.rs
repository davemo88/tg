use std::str::FromStr;
use serde_json;
use bdk::{
    bitcoin::{
        Address,
        PublicKey,
        Transaction,
        secp256k1::Signature,
    },
    blockchain::noop_progress,
};
use bip39::Mnemonic;
use reqwest;
use tglib::{
    Result,
    TgError,
    arbiter::ArbiterService,
    contract::{
        Contract,
        PlayerContractInfo,
    },
    payout::Payout,
    player::PlayerId,
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

    fn submit_payout(&self, payout: &Payout) -> Result<Transaction> {
        Err(TgError("invalid payout"))
    }

    fn get_player_info(&self, player_id: PlayerId) -> Result<PlayerContractInfo> {
        let response = reqwest::blocking::get(&format!("{}/info/{}", self.0, player_id.0)).unwrap();
        let body = String::from(response.text().unwrap());
        let info: PlayerContractInfo = serde_json::from_str(&body).unwrap();
        Ok(info)
    }
}
