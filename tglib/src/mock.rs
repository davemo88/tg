use bdk::bitcoin::{
    PrivateKey,
    PublicKey,
    secp256k1::{
        Secp256k1,
        Message,
        Signature,
    },
    Network,
};
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
pub const SEED_NAME: &'static str = "dev-seed.json";
pub const NETWORK: Network = Network::Regtest;
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
