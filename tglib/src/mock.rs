use std::str::FromStr;
use bdk::{
    Wallet,
    bitcoin::{
        PrivateKey,
        PublicKey,
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
        },
        Network,
    },
    database::MemoryDatabase,
    signer::Signer,
};
use bip39::Mnemonic;
pub use crate::{
    Result as TgResult,
    TgError,
    contract::Contract,
    wallet::{ 
        derive_account_xpubkey,
        derive_account_xprivkey,
        SigningWallet,
        EscrowWallet,
        BITCOIN_ACCOUNT_PATH,
        ESCROW_SUBACCOUNT,
    },
};



pub const DB_NAME: &'static str = "dev-app.db";
pub const NETWORK: Network = Network::Regtest;
pub const BITCOIN_RPC_URL: &'static str = "http://electrs:18443";
pub const ELECTRS_SERVER: &'static str = "tcp://electrs:60401";
pub const REDIS_SERVER: &'static str = "redis://redis/";

pub const CONTRACT_VERSION: u8 = 1;
pub const PAYOUT_VERSION: u8 = 1;
pub const ESCROW_KIX: &'static str = "0";

pub const PLAYER_1_MNEMONIC: &'static str = "deny income tiger glove special recycle cup surface unusual sleep speed scene enroll finger protect dice powder unit";
pub const PLAYER_2_MNEMONIC: &'static str = "carry tooth vague volcano refuse purity bike owner diary dignity toe body notable foil hedgehog mesh dream shock";
// m/44'/0'/0'/7/0
pub const PLAYER_2_ESCROW_PUBKEY: &'static str = "03b0e39d8787171dc23888fb8698c6d5872b6a2bb9eadff7ce4e40edfe8feec24b";
pub const ARBITER_MNEMONIC: &'static str = "meadow found language where fringe casual print marine segment throw old tackle industry chest screen group huge output";
pub const ARBITER_FINGERPRINT: &'static str = "1af44eee";
pub const ARBITER_XPUBKEY: &'static str = "tpubDCoCzmZtfuft3oM8Y5RnaT5GFq27NR7iYLbj5r1HZyfbgMAT1AAeAxCoyMnKGQ67GAeZDcekJgsaSMTb7SpmRJ3vGbPXZxDToKHTRa3mBS2";
// TODO: localhost is no good any more, e.g. need 10.0.2.2 for android dev, needs to be
// configurable
pub const ELECTRUM_PORT: u32 = 60401;
pub const ARBITER_PORT: u32 = 5000;
pub const NAME_SERVICE_PORT: u32 = 18420;
pub const ARBITER_PUBLIC_URL: &'static str = "http://localhost:5000";
pub const NAME_SERVICE_URL: &'static str = "http://localhost:18420";
pub const REFEREE_PRIVKEY: &'static str = "L52hw8to1fdBj9eP8HESBNrfcbehxvKU1vsqWjmHJavxNEi9q91i";

pub fn referee_pubkey() -> PublicKey {
    let secp = Secp256k1::new();
    let key = PrivateKey::from_wif(REFEREE_PRIVKEY).unwrap();
    PublicKey::from_private_key(&secp, &key)
}

pub fn get_referee_signature(msg: Message) -> Signature {
    let secp = Secp256k1::new();
    let key = PrivateKey::from_wif(REFEREE_PRIVKEY).unwrap();
    secp.sign(&msg, &key.key)
}

pub struct Trezor {
    mnemonic: Mnemonic,
    pub wallet: Wallet<(), MemoryDatabase>,
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

    fn sign_tx(&self, psbt: PartiallySignedTransaction, _kdp: String) -> TgResult<PartiallySignedTransaction> {
        let secp = Secp256k1::new();
        let path = DerivationPath::from_str(&String::from(format!("m/{}/{}", ESCROW_SUBACCOUNT, ESCROW_KIX))).unwrap();
        let account_key = derive_account_xprivkey(&self.mnemonic, NETWORK);
        let escrow_key = account_key.derive_priv(&secp, &path).unwrap();
        let mut maybe_signed = psbt.clone();
//        println!("psbt to sign: {:?}", psbt);
//        match Signer::sign(&escrow_key.private_key, &mut maybe_signed, Some(0)) {
        match Signer::sign(&escrow_key.private_key, &mut maybe_signed, Some(0), &secp) {
            Ok(()) => {
                Ok(maybe_signed)
            }
            Err(e) => {
                println!("err: {:?}", e);
                Err(TgError("cannot sign transaction".to_string()))
            }
        }
    }

// TODO : make this work
    fn sign_message(&self, msg: Message, path: DerivationPath) -> TgResult<Signature> {
//        let root_key = ExtendedPrivKey::new_master(NETWORK, &self.mnemonic.to_seed("")).unwrap();
        let account_key = derive_account_xprivkey(&self.mnemonic, NETWORK);
        let secp = Secp256k1::new();
        let signing_key = account_key.derive_priv(&secp, &path).unwrap();
        Ok(secp.sign(&msg, &signing_key.private_key.key))
    }
}

impl EscrowWallet for Trezor {
    fn get_escrow_pubkey(&self) -> PublicKey {
        let path = DerivationPath::from_str(&String::from(format!("m/{}/{}", ESCROW_SUBACCOUNT, ESCROW_KIX))).unwrap();
        let escrow_pubkey = self.xpubkey().derive_pub(&Secp256k1::new(), &path).unwrap();
        escrow_pubkey.public_key
    }

    fn validate_contract(&self, contract: &Contract) -> TgResult<()> {
        if contract.arbiter_pubkey != self.get_escrow_pubkey() {
            return Err(TgError("unexpected arbiter pubkey".to_string()));
        }
        contract.validate()
    }
}

