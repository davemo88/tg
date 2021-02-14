use std::str::FromStr;
use reqwest;
use tglib::{
    bdk::{
        bitcoin::{
            hashes::sha256d,
            Address,
            PublicKey,
            consensus,
            hash_types::Txid,
            secp256k1::Signature,
            util::psbt::PartiallySignedTransaction,
        },
    },
    hex,
    Result,
    TgError,
    arbiter::ArbiterService,
    contract::{
        Contract,
        PlayerContractInfo,
    },
    payout::Payout,
    player::PlayerName,
};


pub struct ArbiterClient(String);

impl ArbiterClient {
    pub fn new (host: &str) -> Self {
        ArbiterClient(String::from(host))
    }

    fn get(&self, command: &str, params: Option<&str>) -> reqwest::Result<reqwest::blocking::Response> {
        let mut url = format!("{}/{}", self.0, command);
        if let Some(params) = params {
            url += &format!("/{}",params);
        }
        reqwest::blocking::get(&url)
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

    fn set_contract_info(&self, info: PlayerContractInfo, pubkey: PublicKey, sig: Signature) -> Result<()> {
        let params = format!("{}/{}/{}",
            hex::encode(serde_json::to_string(&info).unwrap().as_bytes()),
            hex::encode(pubkey.key.serialize()),
            hex::encode(sig.serialize_compact()),
        );
        match self.get("set-contract-info", Some(&params)) {
            Ok(_success_message) => Ok(()),
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

    fn submit_contract(&self, contract: &Contract) -> Result<Signature> {
        match self.get("submit-contract", Some(&hex::encode(contract.to_bytes()))) {
            Ok(response) => match Signature::from_compact(&hex::decode(response.text().unwrap()).unwrap()) {
                Ok(sig) => Ok(sig),
                Err(_) => Err(TgError("invalid contract".to_string()))
            }
            Err(_) => Err(TgError("couldn't submit contract".to_string()))
        }
    }

    fn submit_payout(&self, payout: &Payout) -> Result<PartiallySignedTransaction> {
        match self.get("submit-payout", Some(&hex::encode(payout.to_bytes()))) {
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
