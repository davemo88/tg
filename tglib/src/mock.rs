use std::str::FromStr;
use crate::{
    contract::{
        PlayerContractInfo,
    },
    player::PlayerId,
    wallet::{
        SigningWallet,
    },
    Result as TgResult,
    TgError,
};
use bdk::{
    Wallet,
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
    blockchain::{
        OfflineBlockchain,
        ElectrumBlockchain,
        noop_progress,
    },
    database::MemoryDatabase,
    electrum_client::Client,
};
use bip39::Mnemonic;
pub use crate::{
    wallet::{ 
        derive_account_xpubkey,
        derive_account_xprivkey,
        EscrowWallet,
        BITCOIN_ACCOUNT_PATH,
        ESCROW_SUBACCOUNT,
    },
};



pub const DB_NAME: &'static str = "dev-app.db";
pub const NETWORK: Network = Network::Regtest;
pub const BITCOIN_RPC_URL: &'static str = "http://127.0.0.1:18443";
pub const ELECTRS_SERVER: &'static str = "tcp://127.0.0.1:60401";

pub const ESCROW_KIX: &'static str = "0";

pub const PLAYER_1_MNEMONIC: &'static str = "deny income tiger glove special recycle cup surface unusual sleep speed scene enroll finger protect dice powder unit";
pub const PLAYER_2_MNEMONIC: &'static str = "carry tooth vague volcano refuse purity bike owner diary dignity toe body notable foil hedgehog mesh dream shock";
pub const ARBITER_MNEMONIC: &'static str = "meadow found language where fringe casual print marine segment throw old tackle industry chest screen group huge output";
pub const ARBITER_FINGERPRINT: &'static str = "";
pub const ARBITER_XPUBKEY: &'static str = "";

pub struct MockWallet {
    fingerprint: Fingerprint,
    xpubkey: ExtendedPubKey,
    network: Network,
    pub wallet: Wallet<ElectrumBlockchain, MemoryDatabase>,
}

impl EscrowWallet for MockWallet {
    fn get_escrow_pubkey(&self) -> PublicKey {
        let secp = Secp256k1::new();
        let path = DerivationPath::from_str(&String::from(format!("m/{}/{}", ESCROW_SUBACCOUNT, ESCROW_KIX))).unwrap();
        let escrow_pubkey = self.xpubkey.derive_pub(&secp, &path).unwrap();
        escrow_pubkey.public_key
    }
}

impl MockWallet {
    pub fn new(fingerprint: Fingerprint, xpubkey: ExtendedPubKey, network: Network) -> Self {
        let descriptor_key = format!("[{}/{}]{}", fingerprint, BITCOIN_ACCOUNT_PATH, xpubkey);
        let external_descriptor = format!("wpkh({}/0/*)", descriptor_key);
        let internal_descriptor = format!("wpkh({}/1/*)", descriptor_key);
        let client = Client::new(ELECTRS_SERVER, None).unwrap();

        MockWallet {
            fingerprint,
            xpubkey,
            network,
            wallet: Wallet::new(
                &external_descriptor,
                Some(&internal_descriptor),
                network,
                MemoryDatabase::default(),
                ElectrumBlockchain::from(client)
            ).unwrap(),
        }
    }
}


pub struct PlayerInfoService;

impl PlayerInfoService {
    pub fn get_contract_info(player_id: &PlayerId) -> PlayerContractInfo {
        let signing_wallet = Trezor::new(Mnemonic::parse(PLAYER_2_MNEMONIC).unwrap());
        let player_wallet = MockWallet::new(signing_wallet.fingerprint(), signing_wallet.xpubkey(), NETWORK);
        player_wallet.wallet.sync(noop_progress(), None).unwrap();
        let escrow_pubkey = player_wallet.get_escrow_pubkey();
        PlayerContractInfo {
            escrow_pubkey,
// TODO: send to internal descriptor, no immediate way to do so atm
            change_address: player_wallet.wallet.get_new_address().unwrap(),
            utxos: player_wallet.wallet.list_unspent().unwrap(),
        }
    }
}

pub struct ArbiterService;

impl ArbiterService {
    pub fn get_escrow_pubkey() -> PublicKey {
        let signing_wallet = Trezor::new(Mnemonic::parse(ARBITER_MNEMONIC).unwrap());
        let arbiter_wallet = MockWallet::new(signing_wallet.fingerprint(), signing_wallet.xpubkey(), NETWORK);
        arbiter_wallet.get_escrow_pubkey()
    }

    pub fn get_fee_address() -> Address {
        let root_key = ExtendedPrivKey::new_master(NETWORK, &Mnemonic::parse(ARBITER_MNEMONIC).unwrap().to_seed("")).unwrap();
        let secp = Secp256k1::new();
        let path = DerivationPath::from_str("m/44'/0'/0'/0/0").unwrap();
        let fee_address_key = root_key.derive_priv(&secp, &path).unwrap();
        Address::p2wpkh(&PublicKey::from_private_key(&secp, &fee_address_key.private_key), NETWORK).unwrap()
    }
}

pub struct Trezor {
    mnemonic: Mnemonic,
    wallet: Wallet<OfflineBlockchain, MemoryDatabase>,
}

impl Trezor {
    pub fn new(mnemonic: Mnemonic) -> Self {
        let root_key = ExtendedPrivKey::new_master(NETWORK, &mnemonic.to_seed("")).unwrap();
        let secp = Secp256k1::new();
        let fingerprint = root_key.fingerprint(&secp);
        let account_key = derive_account_xprivkey(&mnemonic, NETWORK);
        let descriptor_key = format!("[{}/{}]{}", fingerprint, BITCOIN_ACCOUNT_PATH, account_key);
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
        let xprivkey = derive_account_xprivkey(&self.mnemonic, NETWORK);
        let secp = Secp256k1::new();
        xprivkey.fingerprint(&secp)
    }

    fn xpubkey(&self) -> ExtendedPubKey {
        derive_account_xpubkey(&self.mnemonic, NETWORK)
    }

    fn sign_tx(&self, pstx: PartiallySignedTransaction, kdp: String) -> TgResult<Transaction> {
        match self.wallet.sign(pstx, None) {
            Ok((signed_tx, _)) => {
                Ok(signed_tx.extract_tx())
            }
            _ => Err(TgError("cannot sign transaction"))
        }
    }

    fn sign_message(&self, msg: Message, path: DerivationPath) -> TgResult<Signature> {
        let root_key = ExtendedPrivKey::new_master(NETWORK, &self.mnemonic.to_seed("")).unwrap();
        let secp = Secp256k1::new();
        let signing_key = root_key.derive_priv(&secp, &path).unwrap();
        Ok(secp.sign(&msg, &signing_key.private_key.key))
    }
}

