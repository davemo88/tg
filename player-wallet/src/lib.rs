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

#[cfg(test)]
mod tests {
    use crate::wallet::PlayerWallet;
    use tglib::{
        mock::{
            get_referee_signature,
            ARBITER_MNEMONIC,
            ESCROW_KIX,
            ESCROW_SUBACCOUNT,
            NETWORK,
            PLAYER_1_MNEMONIC,
            PLAYER_2_MNEMONIC,
        },
    };

    const ELECTRUM_URL: &'static str = "tcp://localhost:60401";
    const NAME_URL: &'static str = "http://localhost:18420";
    const ARBITER_URL: &'static str = "http://localhost:5000";
    const EXCHANGE_URL: &'static str = "http://localhost:5050";

    const DIR_1: &'static str = "/tmp/p1wallet";
    const DIR_2: &'static str = "/tmp/p2wallet";
    const DIR_A: &'static str = "/tmp/arbiterwallet";

    fn all_sign(contract: &mut Contract) {
        let p1_wallet = PlayerWallet::new();
        let p2_wallet = PlayerWallet::new();
        let arbiter_wallet = PlayerWallet::new();
        let sig = sign_contract(&p1_wallet, contract).unwrap();
        contract.sigs.push(sig);
        let sig = sign_contract(&p2_wallet, contract).unwrap();
        contract.sigs.push(sig);
        let sig = sign_contract(&arbiter_wallet, contract).unwrap();
        contract.sigs.push(sig);
    }

    #[test]
    fn pass_p1_payout() {
        let mut contract = Contract::from_bytes(hex::decode(CONTRACT).unwrap()).unwrap();
        all_sign(&mut contract);

        let address = Address::p2wpkh(&contract.p1_pubkey, NETWORK).unwrap();
        let mut payout = create_payout(&contract, &address);
        let wallet = Trezor::new(Mnemonic::parse(PLAYER_1_MNEMONIC).unwrap());
        payout.psbt = sign_payout_psbt(&wallet, &mut payout).unwrap();
        let arbiter_wallet = Trezor::new(Mnemonic::parse(ARBITER_MNEMONIC).unwrap());
        let msg = Message::from_slice(&payout.psbt.clone().extract_tx().txid()).unwrap();
        payout.script_sig = Some(get_referee_signature(msg));

        assert!(arbiter_wallet.validate_payout(&payout).is_ok())
    }

    #[test]
    fn pass_p2_payout() {
        let mut contract = Contract::from_bytes(hex::decode(CONTRACT).unwrap()).unwrap();
        all_sign(&mut contract);

        let address = Address::p2wpkh(&contract.p2_pubkey, NETWORK).unwrap();
        let mut payout = create_payout(&contract, &address);
        let wallet = Trezor::new(Mnemonic::parse(PLAYER_2_MNEMONIC).unwrap());
        payout.psbt = sign_payout_psbt(&wallet, &mut payout).unwrap();
        let arbiter_wallet = Trezor::new(Mnemonic::parse(ARBITER_MNEMONIC).unwrap());
        let msg = Message::from_slice(&payout.psbt.clone().extract_tx().txid()).unwrap();
        payout.script_sig = Some(get_referee_signature(msg));

        let r = arbiter_wallet.validate_payout(&payout);
        assert!(r.is_ok())
    }

    #[test]
    fn fail_unsigned_contract() {
        let contract = Contract::from_bytes(hex::decode(CONTRACT).unwrap()).unwrap();

        let address = Address::p2wpkh(&contract.p1_pubkey, NETWORK).unwrap();
        let mut payout = create_payout(&contract, &address);
        let wallet = Trezor::new(Mnemonic::parse(PLAYER_1_MNEMONIC).unwrap());
        payout.psbt = sign_payout_psbt(&wallet, &mut payout).unwrap();
        let arbiter_wallet = Trezor::new(Mnemonic::parse(ARBITER_MNEMONIC).unwrap());
        let msg = Message::from_slice(&payout.psbt.clone().extract_tx().txid()).unwrap();
        payout.script_sig = Some(get_referee_signature(msg));

        assert!(arbiter_wallet.validate_payout(&payout).is_err())
    }

    #[test]
    fn fail_no_script_sig() {
        let mut contract = Contract::from_bytes(hex::decode(CONTRACT).unwrap()).unwrap();
        all_sign(&mut contract);

        let address = Address::p2wpkh(&contract.p1_pubkey, NETWORK).unwrap();
        let mut payout = create_payout(&contract, &address);
        let wallet = Trezor::new(Mnemonic::parse(PLAYER_1_MNEMONIC).unwrap());
        payout.psbt = sign_payout_psbt(&wallet, &mut payout).unwrap();
        let arbiter_wallet = Trezor::new(Mnemonic::parse(ARBITER_MNEMONIC).unwrap());

        assert!(arbiter_wallet.validate_payout(&payout).is_err())
    }

    #[test]
    fn fail_invalid_script_sig() {
        let mut contract = Contract::from_bytes(hex::decode(CONTRACT).unwrap()).unwrap();
        all_sign(&mut contract);

        let address = Address::p2wpkh(&contract.p1_pubkey, NETWORK).unwrap();
        let mut payout = create_payout(&contract, &address);
        let wallet = Trezor::new(Mnemonic::parse(PLAYER_1_MNEMONIC).unwrap());
        payout.psbt = sign_payout_psbt(&wallet, &mut payout).unwrap();
        let arbiter_wallet = Trezor::new(Mnemonic::parse(ARBITER_MNEMONIC).unwrap());
// signing with the player's wallet incorrectly
        payout.script_sig = Some(wallet.sign_message(Message::from_slice(&payout.psbt.clone().extract_tx().txid()).unwrap(), 
                DerivationPath::from_str(&format!("m/{}/{}", ESCROW_SUBACCOUNT, ESCROW_KIX)).unwrap()).unwrap());

        assert!(arbiter_wallet.validate_payout(&payout).is_err())
    }

    #[test]
    fn fail_unsigned_payout_tx() {
        let mut contract = Contract::from_bytes(hex::decode(CONTRACT).unwrap()).unwrap();
        all_sign(&mut contract);

        let address = Address::p2wpkh(&contract.p1_pubkey, NETWORK).unwrap();
        let mut payout = create_payout(&contract, &address);

        let arbiter_wallet = Trezor::new(Mnemonic::parse(ARBITER_MNEMONIC).unwrap());
        let msg = Message::from_slice(&payout.psbt.clone().extract_tx().txid()).unwrap();
        payout.script_sig = Some(get_referee_signature(msg));
        assert!(arbiter_wallet.validate_payout(&payout).is_err())

    }

    #[test]
    fn fail_invalid_payout_tx() {
        let mut contract = Contract::from_bytes(hex::decode(CONTRACT).unwrap()).unwrap();
        all_sign(&mut contract);

        let wallet = Trezor::new(Mnemonic::parse(PLAYER_1_MNEMONIC).unwrap());
// giving a new address for the payout tx instead of the ones baked into the payout script
        let mut payout = create_payout(&contract, &wallet.wallet.get_new_address().unwrap());
        payout.psbt = sign_payout_psbt(&wallet, &mut payout).unwrap();
        let arbiter_wallet = Trezor::new(Mnemonic::parse(ARBITER_MNEMONIC).unwrap());
        let msg = Message::from_slice(&payout.psbt.clone().extract_tx().txid()).unwrap();
        payout.script_sig = Some(get_referee_signature(msg));
        assert!(arbiter_wallet.validate_payout(&payout).is_err())
    }
}
