use std::convert::From;
use bdk::bitcoin::{
    PublicKey,
    util::bip32::ExtendedPubKey,
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
use crate::{
    Result,
    contract::PlayerContractInfo,
};

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

pub trait PlayerIdentityService {
    fn get_player_id(&self, pubkey: &PublicKey) -> Option<PlayerId>;
    fn get_player_info(&self, player_id: PlayerId) -> Option<PlayerContractInfo>;
}
