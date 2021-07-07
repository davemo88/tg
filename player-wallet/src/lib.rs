pub use sled;
pub use rusqlite;

pub mod arbiter;
pub mod db;
pub mod exchange;
pub mod player;
pub mod ui;
pub mod wallet;

use std::{
    convert::From,
    fmt,
    sync::Arc,
};
use serde::{Serialize, Deserialize};

pub type Result<T> = std::result::Result<T, Error>;

#[derive(Debug)]
pub enum Error {
    Adhoc(&'static str),
    Database(Arc<rusqlite::Error>),
    Io(Arc<std::io::Error>),
    Reqwest(Arc<reqwest::Error>),
    Tglib(Arc<tglib::Error>),
    ElectrumClient(Arc<tglib::bdk::electrum_client::Error>),
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Error::Adhoc(message) => write!(f, "Adhoc({})", message),
            Error::Database(error) => write!(f, "Database({})", error),
            Error::Io(error) => write!(f, "Io({})", error),
            Error::Reqwest(error) => write!(f, "Reqwest({})", error),
            Error::Tglib(error) => write!(f, "Tglib({})", error),
            Error::ElectrumClient(error) => write!(f, "ElectrumClient({})", error),
        }
    }
}

impl std::error::Error for Error {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            Error::Adhoc(_) => None,
            Error::Database(error) => Some(error.as_ref()),
            Error::Io(error) => Some(error.as_ref()),
            Error::Reqwest(error) => Some(error.as_ref()),
            Error::Tglib(error) => Some(error.as_ref()),
            Error::ElectrumClient(error) => Some(error.as_ref()),
        }
    }
}

impl From<rusqlite::Error> for Error {
    fn from(error: rusqlite::Error) -> Self {
        Error::Database(Arc::new(error))
    }
}

impl From<std::io::Error> for Error {
    fn from(error: std::io::Error) -> Self {
        Error::Io(Arc::new(error))
    }
}

impl From<reqwest::Error> for Error {
    fn from(error: reqwest::Error) -> Self {
        Error::Reqwest(Arc::new(error))
    }
}

impl From<tglib::Error> for Error {
    fn from(error: tglib::Error) -> Self {
        Error::Tglib(Arc::new(error))
    }
}

impl From<tglib::bdk::Error> for Error {
    fn from(error: tglib::bdk::Error) -> Self {
        Error::Tglib(Arc::new(tglib::Error::Bdk(error)))
    }
}

impl From<tglib::bdk::electrum_client::Error> for Error {
    fn from(error: tglib::bdk::electrum_client::Error) -> Self {
        Error::ElectrumClient(Arc::new(error))
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Event {
    desc: String,
    oracle_pubkey: tglib::bdk::bitcoin::PublicKey,
    outcomes: Vec<Outcome>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Outcome {
    desc: String,
    token: String,
}
