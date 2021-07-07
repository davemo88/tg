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
    arbiter::{
        Result,
        ArbiterService,
        SubmitContractBody,
        SubmitPayoutBody,
    },
    contract::Contract,
    payout::Payout,
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
            Err(_) => Err(Error::Adhoc("couldn't get result pubkey").into()),
        }
    }

    fn get_fee_address(&self) -> Result<Address> {
        match self.get("fee-address", None) {
            Ok(response) => Ok(Address::from_str(&response.text().unwrap()).unwrap()),
            Err(_) => Err(Error::Adhoc("couldn't get fee address").into()),
        }
    }

//    fn set_contract_info(&self, contract_info: PlayerContractInfo, pubkey: PublicKey, sig: Signature) -> Result<()> {
//        let body = SetContractInfoBody {
//            contract_info,
//            pubkey,
//            sig_hex: hex::encode(sig.serialize_der()),
//        };
//        let _response = self.post("set-contract-info", serde_json::to_string(&body)?)?; 
//        Ok(())
//    }
//
//    fn get_contract_info(&self, player_name: PlayerName) -> Result<Option<PlayerContractInfo>> {
//        let response = self.get("get-contract-info", Some(&hex::encode(player_name.0.as_bytes())))?;
//        let contract_info = match serde_json::from_str::<PlayerContractInfo>(&response.text().unwrap()) {
//            Ok(info) => Some(info),
//            Err(_) => None,
//        };
//        Ok(contract_info)
//    }
//
//    fn send_contract(&self, contract: ContractRecord, player_name: PlayerName) -> Result<()> {
//        let body = SendContractBody {
//            contract,
//            player_name,
//        };
//        self.post("send-contract", serde_json::to_string(&body).unwrap())?;
//        Ok(())
//    }
//
//    fn send_payout(&self, payout: PayoutRecord, player_name: PlayerName) -> Result<()> {
//        let body = SendPayoutBody {
//            payout,
//            player_name,
//        };
//        self.post("send-payout", serde_json::to_string(&body).unwrap())?; 
//        Ok(())
//    }
//
//    fn get_auth_token(&self, player_name: &PlayerName) -> Result<Vec<u8>> {
//        let response = self.get("auth-token", Some(&player_name.0))?; 
//        let token = hex::decode(response.text()?)?.to_vec();
//        Ok(token)
//    }
//
//    fn receive_contract(&self, auth: AuthTokenSig) -> Result<Option<ContractRecord>> {
//        let response = self.post("receive-contract", serde_json::to_string(&auth)?)?; 
//        Ok(serde_json::from_str::<ContractRecord>(&response.text()?).ok())
//    }
//
//    fn receive_payout(&self, auth: AuthTokenSig) -> Result<Option<PayoutRecord>> {
//        let response = self.post("receive-payout", serde_json::to_string(&auth)?)?;
//        Ok(serde_json::from_str::<PayoutRecord>(&response.text()?).ok())
//    }

    fn submit_contract(&self, contract: &Contract) -> Result<Signature> {
        let body = SubmitContractBody { 
            contract_hex: hex::encode(contract.to_bytes()) 
        };
        let response = self.post("submit-contract", serde_json::to_string(&body)?)?; 
        let sig = Signature::from_der(&hex::decode(response.text()?)?)?;
        Ok(sig)
    }

    fn submit_payout(&self, payout: &Payout) -> Result<PartiallySignedTransaction> {
        let body = SubmitPayoutBody {
            payout_hex: hex::encode(payout.to_bytes())
        };
        let response = self.post("submit-payout", serde_json::to_string(&body)?)?; 
        let psbt = consensus::deserialize(&hex::decode(response.text()?)?)?; 
        Ok(psbt)
    }

    fn fund_address(&self, address: Address) -> Result<Txid> {
        let response = self.get("fund-address", Some(&address.to_string()))?;
        let txid = Txid::from_hash(sha256d::Hash::from_str(&response.text()?)?);
        Ok(txid)
    }
}
