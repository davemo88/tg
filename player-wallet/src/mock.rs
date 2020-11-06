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
    SigningWallet,
};

const ARBITER_PUBKEY: &'static str = "bogusarbiterpubkey";
const PLAYER_PUBKEY: &'static str = "bogusplayerpubkey";

const PURPOSE: u32 = 44;
const COIN: u32 = 0;
const ACCOUNT: u32 = 0;

const PLAYER_MNEMONIC: &'static str = "team beauty local basket provide hammer avocado flower virtual soul manual obvious inmate solve almost";

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
        let xprivkey = ExtendedPrivKey::new_master(Network::Testnet, &m.to_seed("")).unwrap();
        let secp = Secp256k1::new();
        let fingerprint = xprivkey.fingerprint(&secp);
        let descriptor_key = format!("[{}/44'/0'/0']{}", fingerprint, xprivkey);
        let external_descriptor = format!("wpkh({}/0/*)", descriptor_key);
        let internal_descriptor = format!("wpkh({}/1/*)", descriptor_key);
        Wallet::new_offline(
            &external_descriptor,
            Some(&internal_descriptor),
            Network::Testnet,
            MemoryDatabase::default(),
        ).unwrap()
    }
}

impl SigningWallet for Trezor {

    fn mxpubkey() -> ExtendedPubKey {
        let m = Mnemonic::parse(PLAYER_MNEMONIC).unwrap();
        let xprivkey = ExtendedPrivKey::new_master(Network::Testnet, &m.to_seed("")).unwrap();
        let secp = Secp256k1::new();
        ExtendedPubKey::from_private(&secp, &xprivkey)
    }

    fn sign_tx(pstx: PartiallySignedTransaction, kdp: String) -> TgResult<Transaction> {
        let wallet = Trezor::wallet();

        let result = wallet.sign(pstx, None);
        if result.is_ok() {
            let (signed_tx, _) = result.unwrap();
            Ok(signed_tx.extract_tx())
        }
        else {
            Err(TgError("cannot sign transaction"))
        }
    }

    fn sign_message(msg: Message, kdp: String) -> TgResult<Signature> {
        Err(TgError("cannot sign message"))

    }
}

pub struct BitcoinCore;

impl SigningWallet for BitcoinCore {

    fn mxpubkey() -> ExtendedPubKey {
        let m = Mnemonic::parse(PLAYER_MNEMONIC).unwrap();
        let xprivkey = ExtendedPrivKey::new_master(Network::Testnet, &m.to_seed("")).unwrap();
        ExtendedPubKey::from_private(&Secp256k1::new(), &xprivkey)
    }

    fn sign_tx(pstx: PartiallySignedTransaction, kdp: String) -> TgResult<Transaction> {
        Err(TgError("cannot sign transaction"))
    }

    fn sign_message(msg: Message, kdp: String) -> TgResult<Signature> {
        Err(TgError("cannot sign message"))

    }
}
