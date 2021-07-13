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

// TODO: maybe these errors could be handled better
const MISSING_DATA_MSG: &'static str = "missing data";
const UNKNOWN_ERROR_MSG: &'static str = "unknown exchange error";

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
            Status::Error => Err(Error::JsonResponse(response.message.unwrap_or(UNKNOWN_ERROR_MSG.into())).into())
        }
    }

    fn get_contract_info(&self, player_name: PlayerName) -> Result<Option<PlayerContractInfo>> {
        let response = self.get("get-contract-info", Some(&hex::encode(player_name.0.as_bytes())))?;
        let response: JsonResponse<String> = serde_json::from_str(&response.text()?)?;
        match response.status {
            Status::Success => match response.data {
                Some(info) => Ok(Some(serde_json::from_str(&info)?)),
                None => Ok(None),
            }
            Status::Error => Err(Error::JsonResponse(response.message.unwrap_or(UNKNOWN_ERROR_MSG.into())).into())
        }
    }

    fn send_contract(&self, contract: TokenContractRecord, player_name: PlayerName) -> Result<()> {
        let body = SendContractBody {
            contract,
            player_name,
        };
        let response = self.post("send-contract", serde_json::to_string(&body)?)?;
        let response: Response = serde_json::from_str(&response.text()?)?;
        match response.status {
            Status::Success => Ok(()),
            Status::Error => Err(Error::JsonResponse(response.message.unwrap_or(UNKNOWN_ERROR_MSG.into())).into())
        }
    }

    fn send_payout(&self, payout: PayoutRecord, player_name: PlayerName) -> Result<()> {
        let body = SendPayoutBody {
            payout,
            player_name,
        };
        let response = self.post("send-payout", serde_json::to_string(&body)?)?; 
        let response: Response = serde_json::from_str(&response.text()?)?;
        match response.status {
            Status::Success => Ok(()),
            Status::Error => Err(Error::JsonResponse(response.message.unwrap_or(UNKNOWN_ERROR_MSG.into())).into())
        }
    }

    fn get_auth_token(&self, player_name: &PlayerName) -> Result<Vec<u8>> {
        let response = self.get("auth-token", Some(&player_name.0))?; 
        let response: JsonResponse<String> = serde_json::from_str(&response.text()?)?;
        match response.status {
            Status::Success => Ok(hex::decode(response.data.ok_or(Error::Adhoc(MISSING_DATA_MSG))?)?),
            Status::Error => Err(Error::JsonResponse(response.message.unwrap_or(UNKNOWN_ERROR_MSG.into())).into())
        }
    }

    fn receive_contract(&self, auth: AuthTokenSig) -> Result<Option<TokenContractRecord>> {
        let response = self.post("receive-contract", serde_json::to_string(&auth)?)?; 
        let response: JsonResponse<String> = serde_json::from_str(&response.text()?)?;
        match response.status {
            Status::Success => match response.data {
                Some(contract) => Ok(Some(serde_json::from_str::<TokenContractRecord>(&contract)?)),
                None => Ok(None),
            }
            Status::Error => Err(Error::JsonResponse(response.message.unwrap_or(UNKNOWN_ERROR_MSG.into())).into())
        }
    }

    fn receive_payout(&self, auth: AuthTokenSig) -> Result<Option<PayoutRecord>> {
        let response = self.post("receive-payout", serde_json::to_string(&auth)?)?;
        let response: JsonResponse<String> = serde_json::from_str(&response.text()?)?;
        match response.status {
            Status::Success => match response.data {
                Some(contract) => Ok(Some(serde_json::from_str::<PayoutRecord>(&contract)?)),
                None => Ok(None),
            }
            Status::Error => Err(Error::JsonResponse(response.message.unwrap_or(UNKNOWN_ERROR_MSG.into())).into())
        }
    }

}
