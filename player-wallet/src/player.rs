use tglib::{
    bdk::bitcoin::{
        secp256k1::Signature,
        PublicKey,
    },
    hex,
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

    fn get_player_names(&self, pubkey: &PublicKey) -> Vec<PlayerName> {
        match self.get("get-player-names", &hex::encode(pubkey.to_bytes())) {
            Ok(body) => body.json::<Vec<String>>().unwrap().iter().map(|name| PlayerName(name.to_string())).collect(),
            Err(_) => Vec::new(),
        }
    }

    fn get_name_address(&self, name: PlayerName) -> Result<String, &'static str> {
        match self.get("get-name-address", &hex::encode(name.0.as_bytes())) {
            Ok(response) => Ok(response.text().unwrap()),
            Err(_) => Err("couldn't get address"),
        }
    }
}
