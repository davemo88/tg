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

    fn get(&self, command: &str, params: &str) -> reqwest::Result<reqwest::blocking::Response>{
        reqwest::blocking::get(&format!("{}/{}/{}", self.0, command, params))
    }
}

impl PlayerNameService for PlayerNameClient {
    fn register_name(&self, name: PlayerName, pubkey: PublicKey, sig: Signature) -> Result<(), String> {
        let params = format!("{}/{}/{}", 
            hex::encode(&name.0.as_bytes()),
            hex::encode(pubkey.to_bytes()),
            hex::encode(sig.serialize_compact()),
        );
        match self.get("register-name", &params) {
            Ok(response) => {
// response contrains name because the endpoint has to implement
// the warp::Reply trait, and so can't return () for success
                let msg = response.text().unwrap();
                if msg == format!("player/{}", name.0) {
                    Ok(())
                } else {
                    Err(msg)
                }
            }
            Err(_) => Err("rpc call to namecoind failed".to_string())
        }
    }

    fn set_contract_info(&self, info: PlayerContractInfo, pubkey: PublicKey, sig: Signature) -> Result<(), String> {
        let params = format!("{}/{}/{}",
            hex::encode(serde_json::to_string(&info).unwrap().as_bytes()),
            hex::encode(pubkey.key.serialize()),
            hex::encode(sig.serialize_compact()),
        );
        match self.get("set-contract-info", &params) {
            Ok(_success_message) => Ok(()),
            Err(e) => Err(e.to_string())
        }
    }

    fn get_contract_info(&self, player_name: PlayerName) -> Option<PlayerContractInfo> {
        match self.get("get-contract-info", &hex::encode(player_name.0.as_bytes())) {
            Ok(response) => {
                match serde_json::from_str::<PlayerContractInfo>(&response.text().unwrap()) {
                    Ok(info) => Some(info),
                    Err(_) => None,
                }
            },
            Err(_) => None,
        }
    }

    fn get_player_names(&self, pubkey: &PublicKey) -> Vec<PlayerName> {
        match self.get("get-player-names", &hex::encode(pubkey.to_bytes())) {
            Ok(body) => body.json::<Vec<String>>().unwrap().iter().map(|name| PlayerName(name.to_string())).collect(),
            Err(_) => Vec::new(),
        }
    }
}
