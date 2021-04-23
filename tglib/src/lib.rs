pub use age;
pub use argon2;
pub use bdk;
pub use bip39;
pub use byteorder;
pub use hex;
pub use log;
pub use nom;
pub use rand;
pub use secrecy;

pub mod mock;

pub mod player;
pub mod arbiter;
pub mod contract;
pub mod payout;
pub mod script;
pub mod wallet;

use std::fmt;
use serde::{
    Serialize,
    Deserialize,
};

pub type Result<T> = std::result::Result<T, Error>;

#[derive(Debug)]
pub enum Error {
    Adhoc(&'static str),
    Bdk(bdk::Error),
    WrongPassword,
    InvalidContract(&'static str),
    InvalidPayout(&'static str),
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Error::Adhoc(message) => write!(f, "Adhoc({})", message),
            Error::Bdk(error) => write!(f, "Bdk({})", error),
            Error::WrongPassword => write!(f, "WrongPassword"),
            Error::InvalidContract(message) => write!(f, "InvalidContract({})", message),
            Error::InvalidPayout(message) => write!(f, "InvalidPayout({})", message),
        }
    }
}

impl std::error::Error for Error {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            Error::Adhoc(_) => None,
            Error::Bdk(e) => Some(e),
            Error::WrongPassword => None,
            Error::InvalidPayout(_) => None,
            Error::InvalidContract(_) => None,
        }
    }
}

impl From<bdk::Error> for Error {
    fn from(error: bdk::Error) -> Self {
        Error::Bdk(error)
    }
}

impl From<bdk::wallet::signer::SignerError> for Error {
    fn from(error: bdk::wallet::signer::SignerError) -> Self {
        Error::Bdk(error.into())
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Status {
    Success,
    Error,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct JsonResponse<T: Serialize> {
    status: Status,
    data: Option<T>,
    message: Option<String>,
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
