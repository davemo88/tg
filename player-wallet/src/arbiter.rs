use std::str::FromStr;
use reqwest;
use tglib::{
    bdk::bitcoin::{
        hashes::sha256d,
        Address,
        PublicKey,
        consensus,
        hash_types::Txid,
        secp256k1::Signature,
        util::psbt::PartiallySignedTransaction,
    },
    hex,
    Result,
    TgError,
    arbiter::{
        ArbiterService,
        AuthTokenSig,
        SendContractBody,
        SendPayoutBody,
        SetContractInfoBody,
    },
    contract::{
        Contract,
        ContractRecord,
        PlayerContractInfo,
    },
    payout::{
        Payout,
        PayoutRecord,
    },
    player::PlayerName,
};


pub struct ArbiterClient(String);

impl ArbiterClient {
    pub fn new (host: &str) -> Self {
        ArbiterClient(String::from(host))
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

impl ArbiterService for ArbiterClient {
    fn get_escrow_pubkey(&self) -> Result<PublicKey> {
        match self.get("escrow-pubkey", None) {
            Ok(response) => Ok(PublicKey::from_str(&response.text().unwrap()).unwrap()),
            Err(_) => Err(TgError("couldn't get result pubkey".to_string())),
        }
    }

    fn get_fee_address(&self) -> Result<Address> {
        match self.get("fee-address", None) {
            Ok(response) => Ok(Address::from_str(&response.text().unwrap()).unwrap()),
            Err(_) => Err(TgError("couldn't get fee address".to_string())),
        }
    }

    fn set_contract_info(&self, contract_info: PlayerContractInfo, pubkey: PublicKey, sig: Signature) -> Result<()> {
        let body = SetContractInfoBody {
            contract_info,
            pubkey,
            sig_hex: hex::encode(sig.serialize_compact()),
        };
        match self.post("set-contract-info", serde_json::to_string(&body).unwrap()) {
            Ok(_reply) => Ok(()),
            Err(e) => Err(TgError(e.to_string()))
        }
    }

    fn get_contract_info(&self, player_name: PlayerName) -> Option<PlayerContractInfo> {
        match self.get("get-contract-info", Some(&hex::encode(player_name.0.as_bytes()))) {
            Ok(response) => {
                match serde_json::from_str::<PlayerContractInfo>(&response.text().unwrap()) {
                    Ok(info) => Some(info),
                    Err(_) => None,
                }
            },
            Err(_) => None,
        }
    }

    fn send_contract(&self, contract: ContractRecord, player_name: PlayerName) -> Result<()> {
        let body = SendContractBody {
            contract,
            player_name,
        };
        match self.post("send-contract", serde_json::to_string(&body).unwrap()) {
            Ok(_) => Ok(()),
            Err(e) => Err(TgError(format!("couldn't send contract: {:?}", e))), 
        }
    }

    fn send_payout(&self, payout: PayoutRecord, player_name: PlayerName) -> Result<()> {
        let body = SendPayoutBody {
            payout,
            player_name,
        };
        match self.post("send-payout", serde_json::to_string(&body).unwrap()) {
            Ok(_) => Ok(()),
            Err(e) => Err(TgError(format!("couldn't send payout: {:?}", e))), 
        }
    }

    fn get_auth_token(&self, player_name: &PlayerName) -> Result<Vec<u8>> {
        match self.get("auth-token", Some(&player_name.0)) {
            Ok(response) => Ok(hex::decode(response.text().unwrap()).unwrap().to_vec()),
            Err(e) => Err(TgError(format!("couldn't get auth token: {:?}", e))), 
        }
    }

    fn receive_contract(&self, auth: AuthTokenSig) -> Result<Option<ContractRecord>> {
        match self.post("receive-contract", serde_json::to_string(&auth).unwrap()) {
            Ok(response) => match serde_json::from_str::<ContractRecord>(&response.text().unwrap()) {
                Ok(contract_record) => Ok(Some(contract_record)),
                Err(_) => Ok(None),
            }
            Err(e) => Err(TgError(format!("couldn't receive contract: {:?}", e))), 
        }
    }

    fn receive_payout(&self, auth: AuthTokenSig) -> Result<Option<PayoutRecord>> {
        match self.post("receive-payout", serde_json::to_string(&auth).unwrap()) {
            Ok(response) => match serde_json::from_str::<PayoutRecord>(&response.text().unwrap()) {
                Ok(payout_record) => Ok(Some(payout_record)),
                Err(_) => Ok(None),
            }
            Err(e) => Err(TgError(format!("couldn't receive payout: {:?}", e))), 
        }
    }

    fn submit_contract(&self, contract: &Contract) -> Result<Signature> {
        match self.post("submit-contract", serde_json::to_string(contract).unwrap()) {
            Ok(response) => match Signature::from_compact(&hex::decode(response.text().unwrap()).unwrap()) {
                Ok(sig) => Ok(sig),
                Err(_) => Err(TgError("invalid contract".to_string()))
            }
            Err(_) => Err(TgError("couldn't submit contract".to_string()))
        }
    }

    fn submit_payout(&self, payout: &Payout) -> Result<PartiallySignedTransaction> {
        match self.post("submit-payout", serde_json::to_string(payout).unwrap()) {
            Ok(response) => match consensus::deserialize(&hex::decode(response.text().unwrap()).unwrap()) {
                Ok(psbt) => Ok(psbt),
                Err(_) => Err(TgError("invalid payout".to_string())),
            }
            Err(_) => Err(TgError("couldn't submit payout".to_string())),
        }
    }

    fn fund_address(&self, address: Address) -> Result<Txid> {
        match self.get("fund-address", Some(&address.to_string())) {
            Ok(response) => {
                Ok(Txid::from_hash(sha256d::Hash::from_str(&response.text().unwrap()).unwrap()))
            },
            Err(_) => Err(TgError("couldn't fund address".to_string())),
        }
    }
}
