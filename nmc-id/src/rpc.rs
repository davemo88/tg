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
use tglib::bdk::bitcoin::PublicKey;

pub const NAMECOIN_RPC_URL: &'static str = "http://guyledouche:yodelinbabaganoush@localhost:18443";
pub const NAME_ENCODING: &'static str = "ascii";

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

    fn sign_raw_transaction_with_wallet(&self, tx_hex: String) -> RpcResult<SignResult> {
        let body = self.build_request_body("signrawtransactionwithwallet", 
            &serde_json::to_string(&tx_hex).unwrap());
        let r: SignResponse = self.post(body).unwrap().json().unwrap();
        Ok(r.result.unwrap())
    }

    fn import_pubkey(&self, pubkey: &PublicKey) -> RpcResult<()> {
        let body = self.build_request_body("importpubkey", 
            &serde_json::to_string(&pubkey.to_string()).unwrap());
        let _r = self.post(body).unwrap();
        Ok(())
    }

    fn fund_raw_transaction(&self, tx_hex: String) -> RpcResult<FundResponse> {
        let params = format!("{}, {{\"fee_rate\":100}}",
            &serde_json::to_string(&tx_hex).unwrap(),
        );
        let body = self.build_request_body("fundrawtransaction", &params); 
        let response: FundResponse = self.post(body).unwrap().json().unwrap();
        Ok(response)
    }

    fn send_raw_transaction(&self, tx_hex: String) -> RpcResult<String> {
        let body = self.build_request_body("sendrawtransaction", 
            &serde_json::to_string(&tx_hex).unwrap());
        let r = self.post(body).unwrap().text().unwrap();
        Ok(r)
    }

    fn decode_raw_transaction(&self, tx_hex: String, is_witness: bool) -> RpcResult<DecodeResponse> {
        let params = format!("{}, {}",
            &serde_json::to_string(&tx_hex).unwrap(),
            &serde_json::to_string(&is_witness).unwrap(),
        );
        let body = self.build_request_body("decoderawtransaction", &params);
        let r = self.post(body).unwrap().json().unwrap();
        Ok(r)
    }

    fn get_raw_transaction(&self, txid: &str, _verbose: bool) -> RpcResult<String> {
        let body = self.build_request_body("decoderawtransaction", txid);
        Ok(self.post(body).unwrap().text().unwrap())
    }

    fn name_new(&self, name: &str, dest_address: &str) -> RpcResult<(String, String)> {
        let params = format!("{}, {{\"destAddress\":{}, \"nameEncoding\":{}}}",
            serde_json::to_string(name).unwrap(),
            serde_json::to_string(dest_address).unwrap(),
            serde_json::to_string(NAME_ENCODING).unwrap(),
        );
        let body = self.build_request_body("name_new", &params);
        let r: NameNewResponse = self.post(body).unwrap().json().unwrap();
        let result = r.result.unwrap();
        Ok((result[0].clone(), result[1].clone()))
    }

    fn name_firstupdate(&self, name: &str, rand: &str, txid: &str, value: Option<&str>, dest_address: &str) -> RpcResult<String> {
        let params = format!("{}, {}, {}, {}, {{\"destAddress\":{}, \"nameEncoding\":{}}}",
            serde_json::to_string(name).unwrap(),
            serde_json::to_string(rand).unwrap(),
            serde_json::to_string(txid).unwrap(),
            serde_json::to_string(value.unwrap_or_default()).unwrap(),
            serde_json::to_string(dest_address).unwrap(),
            serde_json::to_string(NAME_ENCODING).unwrap(),
        );
        let body = self.build_request_body("name_firstupdate", &params);
//        let r = self.post(body.clone()).unwrap().text().unwrap();
//        println!("first update response: {}",r);
        let r: RpcResponse = self.post(body).unwrap().json().unwrap();
        Ok(r.result.unwrap())
    }

    fn name_list(&self, name: Option<&str>) -> RpcResult<Vec<NameStatus>> {
        let params = if let Some(name) = name {
            serde_json::to_string(name).unwrap()
        } else {
            "".to_string()
        };
        let body = self.build_request_body("name_list", &params);
//        let r = self.post(body.clone()).unwrap().text().unwrap();
//        println!("{}", r);
//        Ok(Vec::new())
        let r: NameListResponse = self.post(body).unwrap().json().unwrap();
        Ok(r.result)
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct BaseResponse {
    pub error: Option<String>,
    pub message: Option<String>,
    pub id: Option<String>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct DecodeResult {
pub     txid: String,
pub     hash: String,
pub     size: u64,
pub     vsize: u64,
pub     weight: u64,
pub     version: u64,
pub     locktime: u64,
        #[serde(skip)]
pub     vin: Vec<String>,
        #[serde(skip)]
pub     vout: Vec<String>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct DecodeResponse {
    pub result: Option<DecodeResult>,
    #[serde(flatten)]
    pub base: BaseResponse,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct NameFirstupdateResponse {
    pub result: Option<String>,
    pub error: Option<String>,
    pub id: Option<String>,
}


#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct RpcResponse {
    pub result: Option<String>,
    #[serde(flatten)]
    pub base: BaseResponse,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct NameResult {
    pub hex: String,
    pub rand: Option<String>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct NameResponse {
    pub result: Option<NameResult>,
    #[serde(flatten)]
    pub base: BaseResponse,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct NameNewResponse {
    pub result: Option<Vec<String>>,
    #[serde(flatten)]
    pub base: BaseResponse,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct FundResult {
    pub hex: String,
    pub fee: f64,
    pub changepos: i32,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct FundResponse {
    pub result: Option<FundResult>,
    #[serde(flatten)]
    pub base: BaseResponse,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct SignResult {
    pub hex: String,
    pub complete: bool,
    pub errors: Option<String>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct SignResponse {
    pub result: Option<SignResult>,
    #[serde(flatten)]
    pub base: BaseResponse,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct NameListResponse {
    pub result: Vec<NameStatus>,
    #[serde(flatten)]
    pub base: BaseResponse,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct NameStatus {
    pub name: String,
    pub name_encoding: String,
    pub name_error: Option<String>,
    pub value: String,
    pub value_encoding: String,
    pub value_error: Option<String>,
    pub txid: String,
    pub vout: u8,
    pub address: String,
    pub ismine: bool,
    pub height: u64,
    pub expires_in: i64,
    pub expired: bool,
}


type RpcResult<T> = Result<T, String>;

pub trait NamecoinRpc {
// wallet
    fn create_wallet(&self, name: &str) -> RpcResult<()>;
    fn load_wallet(&self, name: &str) -> RpcResult<()>;
    fn get_new_address(&self) -> RpcResult<String>;
    fn sign_raw_transaction_with_wallet(&self, tx_hex: String) -> RpcResult<SignResult>;
    fn import_pubkey(&self, pubkey: &PublicKey) -> RpcResult<()>;
// raw transaction
    fn create_raw_transaction(&self, input: Vec<TxIn>, output: Vec<TxOut>) -> RpcResult<String>;
    fn name_raw_transaction(&self, tx_hex: String, vout: u8, op: NameOp) -> RpcResult<NameResponse>;
    fn fund_raw_transaction(&self, tx_hex: String) -> RpcResult<FundResponse>;
    fn send_raw_transaction(&self, tx_hex: String) -> RpcResult<String>;
    fn decode_raw_transaction(&self, tx_hex: String, is_witness: bool) -> RpcResult<DecodeResponse>;
    fn get_raw_transaction(&self, txid: &str, verbose: bool) -> RpcResult<String>;
// generation
    fn generate_to_address(&self, nblocks: u8, address: String) -> RpcResult<()>;
// names
    fn name_new(&self, name: &str, dest_address: &str) -> RpcResult<(String, String)>;
    fn name_firstupdate(&self, name: &str, rand: &str, txid: &str, value: Option<&str>, dest_address: &str) -> RpcResult<String>;
    fn name_list(&self, name: Option<&str>) -> RpcResult<Vec<NameStatus>>;
}
