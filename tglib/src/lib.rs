use std::{
    fmt,
};

pub use bdk;
pub use bip39;
pub use hex;
pub use nom;
pub use byteorder;
//pub use serde;

pub mod mock;

pub mod player;
pub mod arbiter;
pub mod contract;
pub mod payout;
pub mod script;
pub mod wallet;

#[derive(Debug)]
pub struct TgError(pub String);
//pub struct TgError(pub &'static str);

impl fmt::Display for TgError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "TgError: {}", self.0)
    }
}

pub type Result<T> = std::result::Result<T, TgError>;
