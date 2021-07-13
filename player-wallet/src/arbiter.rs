use std::str::FromStr;
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
    Error,
    JsonResponse,
    Status,
    arbiter::{
        ArbiterService,
        SubmitContractBody,
        SubmitPayoutBody,
    },
    contract::Contract,
    payout::Payout,
};

type Result<T> = std::result::Result<T, Box<dyn std::error::Error>>;

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
            Err(_) => Err(Error::Adhoc("couldn't get result pubkey").into()),
        }
    }

    fn get_fee_address(&self) -> Result<Address> {
        match self.get("fee-address", None) {
            Ok(response) => Ok(Address::from_str(&response.text().unwrap()).unwrap()),
            Err(_) => Err(Error::Adhoc("couldn't get fee address").into()),
        }
    }

    fn submit_contract(&self, contract: &Contract) -> Result<Signature> {
        let body = SubmitContractBody { 
            contract_hex: hex::encode(contract.to_bytes()) 
        };
        let response = self.post("submit-contract", serde_json::to_string(&body)?)?; 
//        let sig = Signature::from_der(&hex::decode(response.text()?)?)?;
//        Ok(sig)
        let response: JsonResponse<String> = serde_json::from_str(&response.text()?)?;
        match response.status {
            Status::Success => if let Some(sig_hex) = response.data {
                Ok(Signature::from_der(&hex::decode(sig_hex)?)?)
            } else {
                Err(Box::new(Error::Adhoc("missing signature in response")))
            }
            Status::Error => {
                Err(Error::JsonResponse(response.message.unwrap_or("unknown arbiter error".to_string())).into())
            }
        }
    }

    fn submit_payout(&self, payout: &Payout) -> Result<PartiallySignedTransaction> {
        let body = SubmitPayoutBody {
            payout_hex: hex::encode(payout.to_bytes())
        };
        let response = self.post("submit-payout", serde_json::to_string(&body)?)?; 
//        let psbt = consensus::deserialize(&hex::decode(response.text()?)?)?; 
//        Ok(psbt)
        let response: JsonResponse<String> = serde_json::from_str(&response.text()?)?;
        match response.status {
            Status::Success => if let Some(tx) = response.data {
                Ok(consensus::deserialize(&hex::decode(tx)?)?)
            } else {
                Err(Box::new(Error::Adhoc("missing transaction in response")))
            }
            Status::Error => {
                Err(Error::JsonResponse(response.message.unwrap_or("unknown arbiter error".to_string())).into())
            }
        }
    }

    fn fund_address(&self, address: Address) -> Result<Txid> {
        let response = self.get("fund-address", Some(&address.to_string()))?;
        let txid = Txid::from_hash(sha256d::Hash::from_str(&response.text()?)?);
        Ok(txid)
    }
}
