pub use sled;
pub use rusqlite;

pub mod arbiter;
pub mod db;
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
    use std::{
        str::FromStr,
        thread,
        time::Duration,
    };
    use bitcoincore_rpc::{Auth, Client as RpcClient, RpcApi, json::EstimateMode};
    use tglib::{
        bip39::Mnemonic,
        bdk::{
            bitcoin::{
                Amount,
                secp256k1::{
                    Message,
                    Secp256k1,
                },
                util::bip32::DerivationPath,
            },
            electrum_client::Client,
        },
        arbiter::ArbiterService,
        contract::Contract,
        player::PlayerName,
        wallet::{
            SigningWallet,
            BITCOIN_ACCOUNT_PATH,
            ESCROW_SUBACCOUNT,
            create_payout_script,
            create_escrow_address,
        },
        mock::{
            Trezor,
            ARBITER_MNEMONIC,
            ARBITER_PUBLIC_URL,
            ESCROW_KIX,
            PLAYER_1_MNEMONIC,
            PLAYER_2_MNEMONIC,
            NETWORK,
        }
    };
    use crate::{
        arbiter::ArbiterClient,
        wallet::PlayerWallet,
        player::PlayerNameClient,
    };

    const SATS: u64 = 1000000;

    fn local_electrum_client() -> Client {
        Client::new("tcp://localhost:60401").unwrap()
    }

    fn local_player_id_client() -> PlayerNameClient {
        PlayerNameClient::new("http://localhost:18420")
    }

    #[test]
    fn fund_players() {
        let p1_signing_wallet = Trezor::new(Mnemonic::parse(PLAYER_1_MNEMONIC).unwrap()); 
        let p1_wallet = PlayerWallet::new(p1_signing_wallet.fingerprint(), p1_signing_wallet.xpubkey(), NETWORK, local_electrum_client()).unwrap();
        let p1_addr = p1_wallet.new_address();

        let p2_signing_wallet = Trezor::new(Mnemonic::parse(PLAYER_2_MNEMONIC).unwrap()); 
        let p2_wallet = PlayerWallet::new(p2_signing_wallet.fingerprint(), p2_signing_wallet.xpubkey(), NETWORK, local_electrum_client()).unwrap();
        let p2_addr = p2_wallet.new_address();

        let rpc = RpcClient::new("http://127.0.0.1:18443".to_string(), Auth::UserPass("admin".to_string(), "passw".to_string())).unwrap();
        let coinbase_addr = rpc.get_new_address(None, None).unwrap();
        let txid1 = rpc.send_to_address(&p1_addr, Amount::ONE_BTC, None, None, None, None, None, Some(EstimateMode::Conservative)).unwrap();
        let txid2 = rpc.send_to_address(&p2_addr, Amount::ONE_BTC, None, None, None, None, None,Some(EstimateMode::Conservative)).unwrap();
        let blockhashes = rpc.generate_to_address(150, &coinbase_addr).unwrap();
// electrs needs some time to catch up to bitcoind i think?
        thread::sleep(Duration::new(5,0));
    }

    fn create_contract() -> Contract {
        let p1_signing_wallet = Trezor::new(Mnemonic::parse(PLAYER_1_MNEMONIC).unwrap()); 
        let p1_wallet = PlayerWallet::new(p1_signing_wallet.fingerprint(), p1_signing_wallet.xpubkey(), NETWORK, local_electrum_client()).unwrap();
        let arbiter_client = ArbiterClient::new(ARBITER_PUBLIC_URL);
        let arbiter_pubkey = arbiter_client.get_escrow_pubkey().unwrap();
//        let p2_contract_info = local_player_id_client().get_player_info(PlayerId(String::from("player 2"))).unwrap();
        let p2_contract_info = arbiter_client.get_contract_info(PlayerName(String::from("player 2"))).unwrap();
        p1_wallet.create_contract(p2_contract_info, Amount::from_sat(SATS), arbiter_pubkey)
    }

    fn create_signed_contract() -> Contract {
        let p1_signing_wallet = Trezor::new(Mnemonic::parse(PLAYER_1_MNEMONIC).unwrap()); 
        let p1_wallet = PlayerWallet::new(p1_signing_wallet.fingerprint(), p1_signing_wallet.xpubkey(), NETWORK, local_electrum_client()).unwrap();
        let arbiter_client = ArbiterClient::new(ARBITER_PUBLIC_URL);
        let arbiter_pubkey = arbiter_client.get_escrow_pubkey().unwrap();
        let p2_contract_info = arbiter_client.get_contract_info(PlayerName(String::from("player 2"))).unwrap();
        let mut contract = p1_wallet.create_contract(p2_contract_info, Amount::from_sat(SATS), arbiter_pubkey);
        let cxid = contract.cxid();
        
        let p1_sig = p1_signing_wallet.sign_message(
            Message::from_slice(&cxid).unwrap(),
            DerivationPath::from_str(&format!("m/{}/{}/{}", BITCOIN_ACCOUNT_PATH, ESCROW_SUBACCOUNT, ESCROW_KIX)).unwrap(),
        ).unwrap();
        
        let p2_signing_wallet = Trezor::new(Mnemonic::parse(PLAYER_2_MNEMONIC).unwrap()); 
        let p2_wallet = PlayerWallet::new(p2_signing_wallet.fingerprint(), p2_signing_wallet.xpubkey(), NETWORK, local_electrum_client());
        let p2_sig = p2_signing_wallet.sign_message(
            Message::from_slice(&cxid).unwrap(),
            DerivationPath::from_str(&format!("m/{}/{}/{}", BITCOIN_ACCOUNT_PATH, ESCROW_SUBACCOUNT, ESCROW_KIX)).unwrap(),
        ).unwrap();

        let arbiter_wallet = Trezor::new(Mnemonic::parse(ARBITER_MNEMONIC).unwrap()); 
        let arbiter_sig = arbiter_wallet.sign_message(
            Message::from_slice(&cxid).unwrap(),
            DerivationPath::from_str(&format!("m/{}/{}/{}", BITCOIN_ACCOUNT_PATH, ESCROW_SUBACCOUNT, ESCROW_KIX)).unwrap(),
        ).unwrap();
        contract.sigs.push(p1_sig);
        contract.sigs.push(p2_sig);
        contract.sigs.push(arbiter_sig);

        contract
    }

    #[test]
    fn validate_contract() {
// need to check:
// funding_tx is valid
//  spends contract amount to escrow address
//  escrow address is standard multisig with keys
//  p1_pubkey, p2_pubkey, arbiter_pubkey in that order
//  spends fee to arbiter's fee address
// payout_script follows standard template
// signed contract only:
// verify signatures
        let contract = create_signed_contract();

        let arbiter_client = ArbiterClient::new(ARBITER_PUBLIC_URL);
        let arbiter_pubkey = arbiter_client.get_escrow_pubkey().unwrap();

        let escrow_address = create_escrow_address(
            &contract.p1_pubkey,
            &contract.p2_pubkey,
            &arbiter_pubkey,
            NETWORK,
        ).unwrap();
        let escrow_script_pubkey = escrow_address.script_pubkey();
        let amount = SATS;
        let fee_address = arbiter_client.get_fee_address().unwrap();
        let fee_script_pubkey = fee_address.script_pubkey();
        let fee_amount = amount/100;

        let mut matching_escrow_address = false;
        let mut correct_fee = false;

        let funding_tx = contract.clone().funding_tx.extract_tx();
        for txout in funding_tx.output {
            matching_escrow_address = (txout.script_pubkey == escrow_script_pubkey && txout.value == amount) || matching_escrow_address;
            correct_fee = (txout.script_pubkey == fee_script_pubkey && txout.value == fee_amount) || correct_fee; 
        }

        assert!(matching_escrow_address);
        assert!(correct_fee);

        let payout_script = create_payout_script(
            &contract.p1_pubkey,
            &contract.p2_pubkey,
            &arbiter_pubkey,
            &contract.funding_tx.clone().extract_tx(),
            NETWORK,
        );

        assert_eq!(contract.payout_script, payout_script);

        assert_eq!(contract.sigs.len(), 3);
        let secp = Secp256k1::new();
        let msg = Message::from_slice(&contract.cxid()).unwrap();
        assert!(secp.verify(&msg, &contract.sigs[0], &contract.p1_pubkey.key).is_ok());
        assert!(secp.verify(&msg, &contract.sigs[1], &contract.p2_pubkey.key).is_ok());
        assert!(secp.verify(&msg, &contract.sigs[2], &arbiter_pubkey.key).is_ok());
    }

    #[test] 
    fn create_payout() {

    }

    #[test] 
    fn sign_payout() {

    }

    #[test] 
    fn validate_payout() {

    }

    #[test] 
    fn reject_invalid_payout() {

    }
}
