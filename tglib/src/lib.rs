use std::fmt;

pub use age;
pub use argon2;
pub use bdk;
pub use bip39;
pub use byteorder;
pub use hex;
pub use log;
pub use nom;
pub use rand;
pub use reqwest;
pub use secrecy;
//pub use serde;

pub mod mock;

pub mod player;
pub mod arbiter;
pub mod contract;
pub mod payout;
pub mod script;
pub mod wallet;

pub type Result<T> = std::result::Result<T, Error>;

#[derive(Clone, Debug)]
pub enum Error {
    Adhoc(&'static str),
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Error::Adhoc(message) => write!(f, "Adhoc({})", message),
        }
    }
}

impl std::error::Error for Error {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            Error::Adhoc(_) => None,
        }
    }
}
