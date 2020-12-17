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
        match self.post(body).await {
            Ok(_) => Ok(()),
            Err(e) => Err(e.to_string())
        }
    }

    pub async fn load_wallet(&self, name: &str) -> RpcResult<LoadWalletResponse> {
        let body = self.build_request_body("loadwallet", &serde_json::to_string(name).unwrap());
        match self.post(body).await {
            Ok(r) => Ok(r.json::<LoadWalletResponse>().await.unwrap()),
            Err(e) => Err(e.to_string())
        }
//        let r: LoadWalletResponse = self.post(body).await.unwrap().json().await.unwrap();
//        Ok(r)
    }

    pub async fn get_new_address(&self) -> RpcResult<String> {
        let body = self.build_request_body("getnewaddress", "");
        match self.post(body).await {
            Ok(r) => Ok(r.json::<RpcResponse>().await.unwrap().result.unwrap()),
            Err(e) => Err(e.to_string())
        }
    }

    pub async fn generate_to_address(&self, nblocks: u8, address: String) -> RpcResult<()> {
        let params = format!("{}, \"{}\"", nblocks, address);
        let body = self.build_request_body("generatetoaddress", &params);
        match self.post(body).await {
            Ok(_) => Ok(()),
            Err(e) => Err(e.to_string())
        }
    }

    pub async fn import_pubkey(&self, pubkey: &PublicKey) -> RpcResult<()> {
        let body = self.build_request_body("importpubkey", 
            &serde_json::to_string(&pubkey.to_string()).unwrap());
        match self.post(body).await {
            Ok(_) => Ok(()),
            Err(e) => Err(e.to_string())
        }
    }

    pub async fn name_new(&self, name: &str, dest_address: &str) -> RpcResult<(String, String)> {
        let params = format!("{}, {{\"destAddress\":{}, \"nameEncoding\":{}}}",
            serde_json::to_string(name).unwrap(),
            serde_json::to_string(dest_address).unwrap(),
            serde_json::to_string(NAME_ENCODING).unwrap(),
        );
        let body = self.build_request_body("name_new", &params);
        match self.post(body).await {
            Ok(r) => {
                let r = r.json::<NameNewResponse>().await.unwrap();
                if let Some(name_result) = r.result {
                    Ok((name_result[0].clone(), name_result[1].clone()))
                } else {
                    let rpc_error = r.base.error.unwrap();
                    let e = format!("error {}: {}", rpc_error.code, rpc_error.message);
                    println!("name new error: {}",e);
                    Err(e)
                }
            },
            Err(e) => Err(e.to_string()),
        }
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
        match self.post(body).await {
            Ok(r) => Ok(r.json::<RpcResponse>().await.unwrap().result.unwrap()),
            Err(e) => Err(e.to_string()),
        }
    }

    pub async fn _name_list(&self, name: Option<&str>) -> RpcResult<Vec<NameStatus>> {
        let params = serde_json::to_string(&name).unwrap();
        let body = self.build_request_body("name_list", &params);
        match self.post(body).await {
            Ok(r) => Ok(r.json::<NameListResponse>().await.unwrap().result),
            Err(e) => Err(e.to_string()),
        }
    }

    pub async fn name_scan(&self, start: Option<String>, count: Option<u32>, options: Option<NameScanOptions>) -> RpcResult<Vec<NameStatus>> {
        let params = format!("{}, {}, {}",
            serde_json::to_string(&start.unwrap_or("player/".to_string())).unwrap(),
            serde_json::to_string(&count.unwrap_or(50)).unwrap(),
            serde_json::to_string(&options).unwrap(),
        );
        let body = self.build_request_body("name_scan", &params);
        match self.post(body).await {
            Ok(r) => Ok(r.json::<NameScanResponse>().await.unwrap().result),
            Err(e) => Err(e.to_string()),
        }
    }
}

type RpcResult<T> = Result<T, String>;

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct RpcError {
    pub code: i64,
    pub message: String,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct BaseResponse {
    pub error: Option<RpcError>,
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
pub struct LoadWalletResponse {
    pub result: Option<LoadWalletResult>,
    #[serde(flatten)]
    pub base: BaseResponse,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct LoadWalletResult {
    name: String,
    warning: String,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct NameNewResponse {
    pub result: Option<Vec<String>>,
    #[serde(flatten)]
    pub base: BaseResponse,
}

type NameScanResponse = NameListResponse;

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

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct NameScanOptions {
    pub name_encoding: String,
    pub value_encoding: String,
    pub min_conf: Option<i64>,
    pub max_conf: i64,
    pub prefix: String,
    pub regexp: String,
}
