pub use bitcoin;
pub use hex;
use std::collections::HashMap;
use bitcoin::PublicKey;
use serde::{Serialize, Deserialize};

pub const UMP_PUBKEY: &'static str = "025c571f77d693246e64f01ef740064a0b024a228813c94ae7e1e4ee73e991e0ba";

#[derive(Debug)]
pub enum BaseballGameOutcome {
    HomeWins,
    AwayWins,
    Tie,
    Cancelled,
}

impl std::fmt::Display for BaseballGameOutcome {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "{:?}", self)
        // or, alternatively:
        // fmt::Debug::fmt(self, f)
    }
}


#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GameInfo {
    pub home: String,
    pub away: String,
    pub date: String,
    pub outcome_tokens: HashMap<String, (i64, String, Option<String>)>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct JsonResponse<T: Serialize> {
    pub status: Status,
    pub data: Option<T>,
    pub message: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AddSignatureBody {
    pub outcome_id: i64,
    pub sig_hex: String,
}

impl<T: Serialize> JsonResponse<T> {
    pub fn success(data: Option<T>) -> Self {
        JsonResponse {
            status: Status::Success,
            data,
            message: None,
        }
    }
    
    pub fn error(message: String, data: Option<T>) -> Self {
        JsonResponse {
            status: Status::Error,
            data,
            message: Some(message),
        }
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Status {
    Success,
    Error,
}

pub fn ump_pubkey() -> PublicKey {
    PublicKey::from_slice(&hex::decode(UMP_PUBKEY).unwrap()).unwrap()
}
