use reqwest::{
    self,
    Result as ReqwestResult,
    Client,
    Response,
};
use serde::{
    Serialize,
    Deserialize,
};
use tglib::bdk::bitcoin::PublicKey;

pub const NAME_ENCODING: &'static str = "ascii";

pub const JSONRPC_VERSION: &'static str = "1.0";
pub const JSONRPC_ID: &'static str = "nmc-id-test";

#[derive(Clone)]
pub struct NamecoinRpcClient {
    host: String,
    client: Client,
}

impl NamecoinRpcClient {
    pub fn new(host: &str) -> Self {
        NamecoinRpcClient {
            host: String::from(host),
            client: Client::new(),
        }
    }

    fn build_request_body(&self, method: &str, params: &str) -> String {
        format!("{{\"jsonrpc\": \"{}\", \"id\": \"{}\", \"method\": \"{}\", \"params\": [{}]}}",
            JSONRPC_VERSION,
            JSONRPC_ID,
            method,
            params)
    }

    async fn post(&self, body: String) -> ReqwestResult<Response> {
        self.client.post(&self.host)
            .body(body)
            .send().await
    }

    pub async fn create_wallet(&self, name: &str) -> RpcResult<()> {
        let body = self.build_request_body("createwallet", &serde_json::to_string(name).unwrap());
        let _r = self.post(body).await;
        Ok(())
    }

    pub async fn load_wallet(&self, name: &str) -> RpcResult<()> {
        let body = self.build_request_body("loadwallet", &serde_json::to_string(name).unwrap());
        let _r = self.post(body).await;
        Ok(())
    }

    pub async fn get_new_address(&self) -> RpcResult<String> {
        let body = self.build_request_body("getnewaddress", "");
        let r = self.post(body).await;
        println!("{:?}",r);
        let r: RpcResponse = r.unwrap().json().await.unwrap();
        Ok(r.result.unwrap())
    }

    pub async fn generate_to_address(&self, nblocks: u8, address: String) -> RpcResult<()> {
        let params = format!("{}, \"{}\"", nblocks, address);
        let body = self.build_request_body("generatetoaddress", &params);
        let _r = self.post(body).await.unwrap();
        Ok(())
    }

    pub async fn import_pubkey(&self, pubkey: &PublicKey) -> RpcResult<()> {
        let body = self.build_request_body("importpubkey", 
            &serde_json::to_string(&pubkey.to_string()).unwrap());
        let _r = self.post(body).await.unwrap();
        Ok(())
    }

    pub async fn name_new(&self, name: &str, dest_address: &str) -> RpcResult<(String, String)> {
        let params = format!("{}, {{\"destAddress\":{}, \"nameEncoding\":{}}}",
            serde_json::to_string(name).unwrap(),
            serde_json::to_string(dest_address).unwrap(),
            serde_json::to_string(NAME_ENCODING).unwrap(),
        );
        let body = self.build_request_body("name_new", &params);
        let r: NameNewResponse = self.post(body).await.unwrap().json().await.unwrap();
        let result = r.result.unwrap();
        Ok((result[0].clone(), result[1].clone()))
    }

    pub async fn name_firstupdate(&self, name: &str, rand: &str, txid: &str, value: Option<&str>, dest_address: &str) -> RpcResult<String> {
        let params = format!("{}, {}, {}, {}, {{\"destAddress\":{}, \"nameEncoding\":{}}}",
            serde_json::to_string(name).unwrap(),
            serde_json::to_string(rand).unwrap(),
            serde_json::to_string(txid).unwrap(),
            serde_json::to_string(value.unwrap_or_default()).unwrap(),
            serde_json::to_string(dest_address).unwrap(),
            serde_json::to_string(NAME_ENCODING).unwrap(),
        );
        let body = self.build_request_body("name_firstupdate", &params);
        let r: RpcResponse = self.post(body).await.unwrap().json().await.unwrap();
        Ok(r.result.unwrap())
    }

    pub async fn name_list(&self, name: Option<&str>) -> RpcResult<Vec<NameStatus>> {
        let params = if let Some(name) = name {
            serde_json::to_string(name).unwrap()
        } else {
            "".to_string()
        };
        let body = self.build_request_body("name_list", &params);
        let r: NameListResponse = self.post(body).await.unwrap().json().await.unwrap();
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
pub struct RpcResponse {
    pub result: Option<String>,
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
