use std::collections::HashMap;
use reqwest::{
    self,
    Result as ReqwestResult,
    blocking::Response,
};
use serde::{
    Serialize,
    Deserialize,
};

pub const NAMECOIN_RPC_URL: &'static str = "http://guyledouche:yodelinbabaganoush@localhost:18443";

pub const JSONRPC_VERSION: &'static str = "1.0";
pub const JSONRPC_ID: &'static str = "nmc-id-test";

pub type TxOut = HashMap<String,f64>;

#[derive(Debug, Serialize, Deserialize)]
pub struct TxIn {
    pub txid: String,
    pub vout: u8,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct NameOp {
    pub op: String,
    pub name: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub rand: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub value: Option<String>,
}

#[derive(Clone)]
pub struct NamecoinRpcClient {
    host: String,
    client: reqwest::blocking::Client,
}

impl NamecoinRpcClient {
    pub fn new(host: &str) -> Self {
        NamecoinRpcClient {
            host: String::from(host),
            client: reqwest::blocking::Client::new(),
        }
    }

    fn build_request_body(&self, method: &str, params: &str) -> String {
        format!("{{\"jsonrpc\": \"{}\", \"id\": \"{}\", \"method\": \"{}\", \"params\": [{}]}}",
            JSONRPC_VERSION,
            JSONRPC_ID,
            method,
            params)
    }

    fn post(&self, body: String) -> ReqwestResult<Response> {
        self.client.post(NAMECOIN_RPC_URL)
            .body(body)
            .send()
    }
}

impl NamecoinRpc for NamecoinRpcClient {
    fn create_wallet(&self, name: &str) -> RpcResult<()> {
        let body = self.build_request_body("createwallet", &serde_json::to_string(name).unwrap());
        let _r = self.post(body);
        Ok(())
    }

    fn load_wallet(&self, name: &str) -> RpcResult<()> {
        let body = self.build_request_body("loadwallet", &serde_json::to_string(name).unwrap());
        let _r = self.post(body);
        Ok(())
    }

    fn get_new_address(&self) -> RpcResult<String> {
        let body = self.build_request_body("getnewaddress", "");
        let r = self.post(body);
        let r: RpcResponse = r.unwrap().json().unwrap();
        Ok(r.result.unwrap())
    }

    fn create_raw_transaction(&self, input: Vec<TxIn>, output: Vec<TxOut>) -> RpcResult<String> {
        let params = format!("{}, {}",
            &serde_json::to_string(&input).unwrap(),
            &serde_json::to_string(&output).unwrap(),
        );
        let body = self.build_request_body("createrawtransaction", &params);
        let r = self.post(body);
        let r: RpcResponse = r.unwrap().json().unwrap();
        let tx_hex = r.result.unwrap();
        Ok(tx_hex)
    }

    fn name_raw_transaction(&self, tx_hex: String, vout: u8, op: NameOp) -> RpcResult<NameResponse> {
        let params = format!("{}, {}, {}",
            serde_json::to_string(&tx_hex).unwrap(),
            serde_json::to_string(&vout).unwrap(),
            serde_json::to_string(&op).unwrap(),
        );
        let body = self.build_request_body("namerawtransaction", &params);
        let name_response: NameResponse = self.post(body).unwrap().json().unwrap();
        Ok(name_response)
    }

    fn generate_to_address(&self, nblocks: u8, address: String) -> RpcResult<()> {
        let params = format!("{}, \"{}\"", nblocks, address);
        let body = self.build_request_body("generatetoaddress", &params);
        let _r = self.post(body).unwrap();
        Ok(())
    }

    fn sign_raw_transaction_with_wallet(&self, tx_hex: String) -> RpcResult<String> {
        let body = self.build_request_body("signrawtransactionwithwallet", 
            &serde_json::to_string(&tx_hex).unwrap());
        let r = self.post(body).unwrap().text().unwrap();
        Ok(r)
    }

    fn fund_raw_transaction(&self, tx_hex: String) -> RpcResult<FundingResponse> {
        let params = format!("{}, {{\"fee_rate\":100}}",
            &serde_json::to_string(&tx_hex).unwrap(),
        );
        let body = self.build_request_body("fundrawtransaction", &params); 
//        let r = self.post(body).unwrap().text().unwrap();
        let response: FundingResponse = self.post(body).unwrap().json().unwrap();
        Ok(response)
    }

    fn send_raw_transaction(&self, tx_hex: String) -> RpcResult<String> {
        let body = self.build_request_body("sendrawtransaction", 
            &serde_json::to_string(&tx_hex).unwrap());
        let r = self.post(body).unwrap().text().unwrap();
        Ok(r)
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct RpcResponse {
    pub result: Option<String>,
    pub error: Option<String>,
    pub message: Option<String>,
    pub id: Option<String>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct NameResult {
    pub hex: String,
    pub rand: Option<String>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct NameResponse {
    pub result: Option<NameResult>,
    pub error: Option<String>,
    pub message: Option<String>,
    pub id: Option<String>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct FundingResponse {
    pub result: Option<FundingResult>,
    pub error: Option<String>,
    pub message: Option<String>,
    pub id: Option<String>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct FundingResult {
    pub hex: String,
    pub fee: f64,
    pub changepos: i32,
}

type RpcResult<T> = Result<T, String>;

pub trait NamecoinRpc {
// wallet
    fn create_wallet(&self, name: &str) -> RpcResult<()>;
    fn load_wallet(&self, name: &str) -> RpcResult<()>;
    fn get_new_address(&self) -> RpcResult<String>;
    fn sign_raw_transaction_with_wallet(&self, tx_hex: String) -> RpcResult<String>;
// raw transaction
    fn create_raw_transaction(&self, input: Vec<TxIn>, output: Vec<TxOut>) -> RpcResult<String>;
    fn name_raw_transaction(&self, tx_hex: String, vout: u8, op: NameOp) -> RpcResult<NameResponse>;
    fn fund_raw_transaction(&self, tx_hex: String) -> RpcResult<FundingResponse>;
    fn send_raw_transaction(&self, tx_hex: String) -> RpcResult<String>;
// generation
    fn generate_to_address(&self, nblocks: u8, address: String) -> RpcResult<()>;
}
