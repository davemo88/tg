use std::{
    convert::From,
    fmt,
};
use serde::{
    Serialize,
    Deserialize,
};
use bdk::bitcoin::{
    PublicKey,
    util::bip32::ExtendedPubKey,
    secp256k1::Signature,
    hashes::{
        Hash,
        HashEngine,
        sha256::HashEngine as Sha2Engine,
        sha256::Hash as Sha2Hash,
    },
    bech32::{
        self,
        ToBase32,
    },
};

#[derive(Clone, Debug, Hash, PartialEq, Eq, Default, Serialize, Deserialize)]
pub struct PlayerName(pub String);

impl fmt::Display for PlayerName {
    // This trait requires `fmt` with this exact signature.
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

pub trait PlayerNameService {
    fn register_name(&self, name: PlayerName, pubkey: PublicKey, sig: Signature) -> Result<(), String>;
//    fn set_contract_info(&self, info: PlayerContractInfo, pubkey: PublicKey, sig: Signature) -> Result<(), String>;
//    fn get_contract_info(&self, name: PlayerName) -> Option<PlayerContractInfo>;
    fn get_player_names(&self, pubkey: &PublicKey) -> Vec<PlayerName>;
}

#[derive(Debug, Default, Clone)]
pub struct PlayerId(pub String); 

impl From<ExtendedPubKey> for PlayerId {
    fn from(xpubkey: ExtendedPubKey) -> Self {
        let mut hash_engine = Sha2Engine::default();
// extended pubkey -> bitcoin pubkey wrapper -> actual pubkey
        hash_engine.input(&xpubkey.public_key.key.serialize_uncompressed());
        let pubkey_hash: &[u8] = &Sha2Hash::from_engine(hash_engine);
        let encoded = bech32::encode("player", pubkey_hash.to_base32()).unwrap();
        PlayerId(encoded)
    }
}

impl From<PublicKey> for PlayerId {
    fn from(pubkey: PublicKey) -> Self {
        let mut hash_engine = Sha2Engine::default();
// bitcoin pubkey wrapper -> actual pubkey
        hash_engine.input(&pubkey.key.serialize_uncompressed());
        let pubkey_hash: &[u8] = &Sha2Hash::from_engine(hash_engine);
        let encoded = bech32::encode("player", pubkey_hash.to_base32()).unwrap();
        PlayerId(encoded)
    }
}
