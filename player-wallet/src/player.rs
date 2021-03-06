use tglib::{
    bdk::bitcoin::{
        secp256k1::Signature,
        PublicKey,
    },
    hex,
    Error,
    JsonResponse,
    Status,
    player::{
        PlayerName,
        PlayerNameService,
        RegisterNameBody,
        Result,
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

    fn post(&self, command: &str, body: String) -> reqwest::Result<reqwest::blocking::Response> {
        reqwest::blocking::Client::new().post(&format!("{}/{}", self.0, command))
            .body(body)
            .send()
    }
}

impl PlayerNameService for PlayerNameClient {
    fn register_name(&self, player_name: PlayerName, pubkey: PublicKey, sig: Signature) -> Result<()> {
// TODO: require player_name isn't empty / all whitespace or something lame
        let body = RegisterNameBody {
            player_name: player_name.clone(),
            pubkey,
            sig_hex: hex::encode(sig.serialize_der()),
        };
        let response: JsonResponse<String> = serde_json::from_str(&self.post("register-name", serde_json::to_string(&body).unwrap())?.text()?)?;
        match response.status {
            Status::Success => Ok(()),
            Status::Error => Err(Box::new(Error::JsonResponse(response.message.unwrap_or("register name failed mysteriously".into())))),
        }
    }

    fn get_player_names(&self, pubkey: &PublicKey) -> Vec<PlayerName> {
        match self.get("get-player-names", &hex::encode(pubkey.to_bytes())) {
            Ok(body) => body.json::<Vec<String>>().unwrap().iter().map(|name| PlayerName(name.to_string())).collect(),
            Err(_) => Vec::new(),
        }
    }

    fn get_name_address(&self, name: PlayerName) -> Result<String> {
        match self.get("get-name-address", &hex::encode(name.0.as_bytes())) {
            Ok(response) => Ok(response.text().unwrap()),
            Err(e) => Err(Box::new(e)),
        }
    }
}
