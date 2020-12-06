use reqwest;
use serde_json;
use tglib::{
    bdk::bitcoin::PublicKey,
    hex,
    contract::PlayerContractInfo,
    player::{
        PlayerId,
        PlayerIdService,
    },
};

pub struct PlayerIdClient(String);

impl PlayerIdClient {
    pub fn new (host: &str) -> Self {
        PlayerIdClient(String::from(host))
    }
}

impl PlayerIdService for PlayerIdClient {
    fn get_player_id(&self, pubkey: &PublicKey) -> Option<PlayerId> {
        let response = reqwest::blocking::get(&format!("{}/get-player-id/{}", self.0, hex::encode(pubkey.to_bytes()))).unwrap();
        let body = String::from(response.text().unwrap());
        Some(PlayerId(body))
    }

    fn get_player_info(&self, player_id: PlayerId) -> Option<PlayerContractInfo> {
        let response = reqwest::blocking::get(&format!("{}/get-player-info/{}", self.0, player_id.0)).unwrap();
        let body = String::from(response.text().unwrap());
        let info: PlayerContractInfo = serde_json::from_str(&body).unwrap();
        Some(info)
    }
}
