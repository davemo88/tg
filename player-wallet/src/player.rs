use reqwest;
use serde_json;
use tglib::{
    bdk::bitcoin::{
        secp256k1::Signature,
        PublicKey,
    },
    hex,
    contract::PlayerContractInfo,
    player::{
        PlayerName,
        PlayerNameService,
    },
};

pub struct PlayerNameClient(String);

impl PlayerNameClient {
    pub fn new (host: &str) -> Self {
        PlayerNameClient(String::from(host))
    }
}

impl PlayerNameService for PlayerNameClient {
    fn get_player_name(&self, pubkey: &PublicKey) -> Option<PlayerName> {
        let response = reqwest::blocking::get(&format!("{}/get-player-id/{}", self.0, hex::encode(pubkey.to_bytes()))).unwrap();
        let body = String::from(response.text().unwrap());
        Some(PlayerName(body))
    }

    fn get_contract_info(&self, player_name: PlayerName) -> Option<PlayerContractInfo> {
        let response = reqwest::blocking::get(&format!("{}/get-player-info/{}", self.0, player_name.0)).unwrap();
        let body = String::from(response.text().unwrap());
        let info: PlayerContractInfo = serde_json::from_str(&body).unwrap();
        Some(info)
    }

    fn set_contract_info(&self, info: PlayerContractInfo, pubkey: PublicKey, sig: Signature) -> Result<(), String> {
        Ok(())
    }

    fn register_name(&self, name: PlayerName, pubkey: PublicKey, sig: Signature) -> Result<(), String> {
        Ok(())
    }
}
