use std::{
    fmt,
};

pub mod player;
pub mod arbiter;
pub mod contract;
pub mod payout;
pub mod script;
pub mod wallet;

pub type TgScriptSig = Vec<Vec<u8>>;

#[derive(Debug)]
pub struct TgError(pub &'static str);

impl fmt::Display for TgError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "TgError: {}", self.0)
    }
}

pub type Result<T> = std::result::Result<T, TgError>;

#[cfg(test)]
mod tests {
    #[test]
    fn it_works() {
        assert_eq!(2 + 2, 4);
    }
}
