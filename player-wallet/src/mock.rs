use std::str::FromStr;
use tglib::{
    contract::{
        PlayerContractInfo,
    },
    player::PlayerId,
    wallet::{
//        Signing,
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
        Address,
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
    wallet::{
        PlayerWallet,
        SigningWallet,
    }
};

pub const NETWORK: Network = Network::Regtest;
pub const BITCOIN_RPC_URL: &'static str = "http://127.0.0.1:18443";
pub const ELECTRS_SERVER: &'static str = "tcp://127.0.0.1:60401";
pub const ARBITER_ID: &'static str = "arbiter1somebogusid";

pub const BITCOIN_DERIVATION_PATH: &'static str = "44'/0'/0'";
pub const ESCROW_SUBACCOUNT: &'static str = "7";
pub const ESCROW_KIX: &'static str = "0";

pub const PLAYER_1_MNEMONIC: &'static str = "deny income tiger glove special recycle cup surface unusual sleep speed scene enroll finger protect dice powder unit";
pub const PLAYER_2_MNEMONIC: &'static str = "carry tooth vague volcano refuse purity bike owner diary dignity toe body notable foil hedgehog mesh dream shock";
pub const ARBITER_MNEMONIC: &'static str = "meadow found language where fringe casual print marine segment throw old tackle industry chest screen group huge output";

pub const PASSPHRASE: &'static str = "testpass";

pub struct PlayerInfoService;

impl PlayerInfoService {
    pub fn get_contract_info(player_id: &PlayerId) -> PlayerContractInfo {
        let signing_wallet = Trezor::new(Mnemonic::parse(PLAYER_2_MNEMONIC).unwrap());
        let player_wallet = PlayerWallet::new(signing_wallet.fingerprint(), signing_wallet.xpubkey(), NETWORK);
        PlayerContractInfo {
            escrow_pubkey: player_wallet.get_new_escrow_pubkey(),
            payout_address: player_wallet.new_address(),
// TODO: send to internal descriptor, no immediate way to do so atm
            change_address: player_wallet.new_address(),
            utxos: player_wallet.wallet.list_unspent().unwrap(),
        }
    }
}

pub struct ArbiterService;

impl ArbiterService {
    pub fn get_escrow_pubkey() -> PublicKey {
        let signing_wallet = Trezor::new(Mnemonic::parse(ARBITER_MNEMONIC).unwrap());
        let arbiter_wallet = PlayerWallet::new(signing_wallet.fingerprint(), signing_wallet.xpubkey(), NETWORK);
        arbiter_wallet.get_new_escrow_pubkey()
    }

    pub fn get_fee_address() -> Address {
        let signing_wallet = Trezor::new(Mnemonic::parse(ARBITER_MNEMONIC).unwrap());
        let arbiter_wallet = PlayerWallet::new(signing_wallet.fingerprint(), signing_wallet.xpubkey(), NETWORK);
        arbiter_wallet.new_address()
    }
}

pub struct Trezor {
    mnemonic: Mnemonic,
    wallet: Wallet<OfflineBlockchain, MemoryDatabase>,
}

impl Trezor {
    pub fn new(mnemonic: Mnemonic) -> Self {
        let xprivkey = ExtendedPrivKey::new_master(NETWORK, &mnemonic.to_seed("")).unwrap();
        let secp = Secp256k1::new();
        let fingerprint = xprivkey.fingerprint(&secp);
// this is wrong. i didn't actually derive the key below, just pasted in the master lol
// the wallet doesn't derive the full chain of child keys
// what should be passed here is either the extended key just one above the wildcard in the tree
        let descriptor_key = format!("[{}/{}]{}", fingerprint, BITCOIN_DERIVATION_PATH, xprivkey);
        let external_descriptor = format!("wpkh({}/0/*)", descriptor_key);
        let internal_descriptor = format!("wpkh({}/1/*)", descriptor_key);
        let wallet = Wallet::new_offline(
            &external_descriptor,
            Some(&internal_descriptor),
            NETWORK,
            MemoryDatabase::default(),
        ).unwrap();

        Trezor {
            mnemonic,
            wallet,
        }
    }
}

impl SigningWallet for Trezor {

    fn fingerprint(&self) -> Fingerprint {
        let xprivkey = ExtendedPrivKey::new_master(NETWORK, &self.mnemonic.to_seed("")).unwrap();
        let secp = Secp256k1::new();
        xprivkey.fingerprint(&secp)
    }

    fn xpubkey(&self) -> ExtendedPubKey {
        let xprivkey = ExtendedPrivKey::new_master(NETWORK, &self.mnemonic.to_seed("")).unwrap();
        let secp = Secp256k1::new();
        let fingerprint = xprivkey.fingerprint(&secp);
        ExtendedPubKey::from_private(&secp, &xprivkey)
    }

    fn descriptor_xpubkey(&self) -> String {
        let xprivkey = ExtendedPrivKey::new_master(NETWORK, &self.mnemonic.to_seed("")).unwrap();
        let secp = Secp256k1::new();
        let fingerprint = xprivkey.fingerprint(&secp);
        let xpubkey = ExtendedPubKey::from_private(&secp, &xprivkey);
        String::from(format!("[{}/44'/0'/0']{}", fingerprint, xpubkey))
    }

    fn sign_tx(&self, pstx: PartiallySignedTransaction, kdp: String) -> TgResult<Transaction> {
        match self.wallet.sign(pstx, None) {
            Ok((signed_tx, _)) => {
                Ok(signed_tx.extract_tx())
            }
            _ => Err(TgError("cannot sign transaction"))
        }
    }

    fn sign_message(&self, msg: Message, kdp: String) -> TgResult<Signature> {
        Err(TgError("cannot sign message"))

    }
}
