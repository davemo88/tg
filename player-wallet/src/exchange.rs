use libexchange::{
    AuthTokenSig,
    ExchangeService,
    PlayerContractInfo,
    SendContractBody,
    SendPayoutBody,
    SetContractInfoBody,
    TokenContractRecord,
    PayoutRecord,
};
use tglib::{
    Error,
    hex,
    bdk::bitcoin::{
        PublicKey,
        secp256k1::Signature,
    },
    JsonResponse,
    Status,
    player::PlayerName,
};

pub type Result<T> = std::result::Result<T, Box<dyn std::error::Error>>;
type Response = JsonResponse<()>;

pub struct ExchangeClient(String);

impl ExchangeClient {
    pub fn new (host: &str) -> Self {
        ExchangeClient(String::from(host))
    }

// lol this should be a macro
    fn get(&self, command: &str, params: Option<&str>) -> reqwest::Result<reqwest::blocking::Response> {
        let mut url = format!("{}/{}", self.0, command);
        if let Some(params) = params {
            url += &format!("/{}",params);
        }
        reqwest::blocking::get(&url)
    }

    fn post(&self, command: &str, body: String) -> reqwest::Result<reqwest::blocking::Response> {
        reqwest::blocking::Client::new().post(&format!("{}/{}", self.0, command))
            .body(body)
            .send()
    }
}

impl ExchangeService for ExchangeClient {
    fn set_contract_info(&self, contract_info: PlayerContractInfo, pubkey: PublicKey, sig: Signature) -> Result<()> {
        let body = SetContractInfoBody {
            contract_info,
            pubkey,
            sig_hex: hex::encode(sig.serialize_der()),
        };
        let response = self.post("set-contract-info", serde_json::to_string(&body)?)?; 
        let response: Response = serde_json::from_str(&response.text()?)?;
        match response.status {
            Status::Success => Ok(()),
            Status::Error => Err(Error::JsonResponse(response.message.unwrap_or("unknown exchange error".into())).into())
        }
//        Ok(())
    }

    fn get_contract_info(&self, player_name: PlayerName) -> Result<Option<PlayerContractInfo>> {
        let response = self.get("get-contract-info", Some(&hex::encode(player_name.0.as_bytes())))?;
        let response: JsonResponse<String> = serde_json::from_str(&response.text()?)?;
        match response.status {
            Status::Success => Ok(Some(serde_json::from_str(&response.data.unwrap()).unwrap())),
            Status::Error => Err(Error::JsonResponse(response.message.unwrap_or("unknown exchange error".into())).into())
        }
//        let contract_info = match serde_json::from_str::<PlayerContractInfo>(&response.text().unwrap()) {
//            Ok(info) => Some(info),
//            Err(_) => None,
//        };
//        Ok(contract_info)
    }

    fn send_contract(&self, contract: TokenContractRecord, player_name: PlayerName) -> Result<()> {
        let body = SendContractBody {
            contract,
            player_name,
        };
        let response = self.post("send-contract", serde_json::to_string(&body).unwrap())?;
//        Ok(())
        let response: Response = serde_json::from_str(&response.text()?)?;
        match response.status {
            Status::Success => Ok(()),
            Status::Error => Err(Error::JsonResponse(response.message.unwrap_or("unknown exchange error".into())).into())
        }
    }

    fn send_payout(&self, payout: PayoutRecord, player_name: PlayerName) -> Result<()> {
        let body = SendPayoutBody {
            payout,
            player_name,
        };
        let response = self.post("send-payout", serde_json::to_string(&body).unwrap())?; 
//        Ok(())
        let response: Response = serde_json::from_str(&response.text()?)?;
        match response.status {
            Status::Success => Ok(()),
            Status::Error => Err(Error::JsonResponse(response.message.unwrap_or("unknown exchange error".into())).into())
        }
    }

    fn get_auth_token(&self, player_name: &PlayerName) -> Result<Vec<u8>> {
        let response = self.get("auth-token", Some(&player_name.0))?; 
        let response: JsonResponse<String> = serde_json::from_str(&response.text()?)?;
        match response.status {
            Status::Success => Ok(hex::decode(response.data.unwrap())?),
            Status::Error => Err(Error::JsonResponse(response.message.unwrap_or("unknown exchange error".into())).into())
        }
//        let token = hex::decode(response.text()?)?.to_vec();
//        Ok(token)
    }

    fn receive_contract(&self, auth: AuthTokenSig) -> Result<Option<TokenContractRecord>> {
        let response = self.post("receive-contract", serde_json::to_string(&auth)?)?; 
//        Ok(serde_json::from_str::<TokenContractRecord>(&response.text()?).ok())
        let response: JsonResponse<String> = serde_json::from_str(&response.text()?)?;
        match response.status {
            Status::Success => Ok(Some(serde_json::from_str::<TokenContractRecord>(&response.data.unwrap())?)),
            Status::Error => Err(Error::JsonResponse(response.message.unwrap_or("unknown exchange error".into())).into())
        }
    }

    fn receive_payout(&self, auth: AuthTokenSig) -> Result<Option<PayoutRecord>> {
        let response = self.post("receive-payout", serde_json::to_string(&auth)?)?;
//        Ok(serde_json::from_str::<PayoutRecord>(&response.text()?).ok())
        let response: JsonResponse<String> = serde_json::from_str(&response.text()?)?;
        match response.status {
            Status::Success => Ok(Some(serde_json::from_str::<PayoutRecord>(&response.data.unwrap())?)),
            Status::Error => Err(Error::JsonResponse(response.message.unwrap_or("unknown exchange error".into())).into())
        }
    }

}
