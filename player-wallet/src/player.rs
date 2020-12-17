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
    fn register_name(&self, name: PlayerName, pubkey: PublicKey, sig: Signature) -> Result<(), String> {
        let body = format!("{}/register-name/{}/{}/{}", self.0, 
            &name.0,
            hex::encode(pubkey.to_bytes()),
            hex::encode(sig.serialize_compact()),
        );
        let response = reqwest::blocking::get(&body).unwrap();
        let body = String::from(response.text().unwrap());
// response contrains name because the endpoint has to implement
// the warp::Reply trait, and so can't return () for success
        if body == name.0 {
            Ok(())
        } else {
            Err(body)
        }
    }
    fn set_contract_info(&self, info: PlayerContractInfo, pubkey: PublicKey, sig: Signature) -> Result<(), String> {
        let body = format!("{}/set-contract-info/{}/{}/{}", self.0, 
            serde_json::to_string(&info).unwrap(),
            hex::encode(pubkey.key.serialize()),
            hex::encode(sig.serialize_compact()),
        );
        let response = reqwest::blocking::get(&body).unwrap();
        let _body = String::from(response.text().unwrap());
        Ok(())
    }

    fn get_contract_info(&self, player_name: PlayerName) -> Option<PlayerContractInfo> {
        let response = reqwest::blocking::get(&format!("{}/get-contract-info/{}", self.0, player_name.0)).unwrap();
        let body = String::from(response.text().unwrap());
        let info: PlayerContractInfo = serde_json::from_str(&body).unwrap();
        Some(info)
    }

    fn get_player_names(&self, pubkey: &PublicKey) -> Vec<PlayerName> {
        let response = reqwest::blocking::get(&format!("{}/get-player-name/{}", self.0, hex::encode(pubkey.to_bytes())));
        match response {
            Ok(body) => body.json().unwrap(),
            Err(_) => Vec::new(),
        }
//        response.json().unwrap()
    }

}
