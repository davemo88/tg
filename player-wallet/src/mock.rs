use std::str::FromStr;
use tglib::{
    player::PlayerId,
    wallet::{
        Signing,
    },
    Result as TgResult,
    TgError,
};
use bdk::{
    bitcoin::{
        util::{
            bip32::{
                DerivationPath,
                ExtendedPubKey,
                ExtendedPrivKey,
                Fingerprint,
            },
            psbt::PartiallySignedTransaction,
        },
        secp256k1::{
            Secp256k1,
            Message,
            Signature,
            All,
        },
        Network,
        PublicKey,
        Transaction,
    },
    blockchain::OfflineBlockchain,
    database::MemoryDatabase,
    Wallet,
};
use bip39::Mnemonic;
use crate::{
    wallet::SigningWallet,
};

pub const NETWORK: Network = Network::Regtest;
pub const ELECTRS_SERVER: &'static str = "tcp://127.0.0.1:60401";
pub const ARBITER_ID: &'static str = "arbiter1somebogusid";
const ARBITER_PUBKEY: &'static str = "bogusarbiterpubkey";
const PLAYER_PUBKEY: &'static str = "bogusotherplayerpubkey";

const PURPOSE: u32 = 44;
const COIN: u32 = 0;
const ACCOUNT: u32 = 0;

const PLAYER_MNEMONIC: &'static str = "snake predict impose woman wire tattoo hungry survey uphold breeze system learn clerk media faint brisk betray retreat";

pub const PASSPHRASE: &'static str = "testpass";

pub struct ArbiterPubkeyService;

impl ArbiterPubkeyService {
    pub fn get_pubkey() -> PublicKey {
        PublicKey::from_str(ARBITER_PUBKEY).unwrap()
    }
}

pub struct PlayerPubkeyService;

impl PlayerPubkeyService {
    pub fn get_pubkey(player_id: &PlayerId) -> PublicKey {
        PublicKey::from_str(PLAYER_PUBKEY).unwrap()
    }
}

pub struct Trezor;

impl Trezor {
    fn wallet() -> Wallet<OfflineBlockchain, MemoryDatabase> {
        let m = Mnemonic::parse(PLAYER_MNEMONIC).unwrap();
        let xprivkey = ExtendedPrivKey::new_master(Network::Regtest, &m.to_seed("")).unwrap();
        let secp = Secp256k1::new();
        let fingerprint = xprivkey.fingerprint(&secp);
        let descriptor_key = format!("[{}/44'/0'/0']{}", fingerprint, xprivkey);
        let external_descriptor = format!("wpkh({}/0/*)", descriptor_key);
        let internal_descriptor = format!("wpkh({}/1/*)", descriptor_key);
        Wallet::new_offline(
            &external_descriptor,
            Some(&internal_descriptor),
            NETWORK,
            MemoryDatabase::default(),
        ).unwrap()
    }
}

impl SigningWallet for Trezor {

    fn fingerprint() -> Fingerprint {
        let m = Mnemonic::parse(PLAYER_MNEMONIC).unwrap();
        let xprivkey = ExtendedPrivKey::new_master(NETWORK, &m.to_seed("")).unwrap();
        let secp = Secp256k1::new();
        xprivkey.fingerprint(&secp)
    }

    fn xpubkey() -> ExtendedPubKey {
        let m = Mnemonic::parse(PLAYER_MNEMONIC).unwrap();
        let xprivkey = ExtendedPrivKey::new_master(NETWORK, &m.to_seed("")).unwrap();
        let secp = Secp256k1::new();
        let fingerprint = xprivkey.fingerprint(&secp);
        ExtendedPubKey::from_private(&secp, &xprivkey)
    }

    fn descriptor_xpubkey() -> String {
        let m = Mnemonic::parse(PLAYER_MNEMONIC).unwrap();
        let xprivkey = ExtendedPrivKey::new_master(NETWORK, &m.to_seed("")).unwrap();
        let secp = Secp256k1::new();
        let fingerprint = xprivkey.fingerprint(&secp);
        let xpubkey = ExtendedPubKey::from_private(&secp, &xprivkey);
        String::from(format!("[{}/44'/0'/0']{}", fingerprint, xpubkey))
    }

    fn sign_tx(pstx: PartiallySignedTransaction, kdp: String) -> TgResult<Transaction> {
        let wallet = Trezor::wallet();

        match wallet.sign(pstx, None) {
            Ok((signed_tx, _)) => {
                Ok(signed_tx.extract_tx())
            }
            _ => Err(TgError("cannot sign transaction"))
        }
    }

    fn sign_message(msg: Message, kdp: String) -> TgResult<Signature> {
        Err(TgError("cannot sign message"))

    }
}

//pub struct BitcoinCore;
//
//impl SigningWallet for BitcoinCore {
//
//    fn descriptor_xpubkey() -> ExtendedPubKey {
//        let m = Mnemonic::parse(PLAYER_MNEMONIC).unwrap();
//        let xprivkey = ExtendedPrivKey::new_master(Network::Testnet, &m.to_seed("")).unwrap();
//        ExtendedPubKey::from_private(&Secp256k1::new(), &xprivkey)
//    }
//
//    fn sign_tx(pstx: PartiallySignedTransaction, kdp: String) -> TgResult<Transaction> {
//        Err(TgError("cannot sign transaction"))
//    }
//
//    fn sign_message(msg: Message, kdp: String) -> TgResult<Signature> {
//        Err(TgError("cannot sign message"))
//
//    }
//}
