use std::convert::{
    From,
};
use bdk::bitcoin::{
    util::bip32::{
        ExtendedPubKey,
    },
    hashes::{
        Hash,
        HashEngine,
        sha256::HashEngine as Sha2Engine,
        sha256::Hash as Sha2Hash,
    },
    bech32::{
        self,
        FromBase32,
        ToBase32,
    },
};

#[derive(Debug, Default, Clone)]
pub struct ArbiterId(pub String); 

impl From<ExtendedPubKey> for ArbiterId {
    fn from(xpubkey: ExtendedPubKey) -> Self {
// extended pubkey -> bitcoin pubkey wrapper -> actual pubkey
        let mut hash_engine = Sha2Engine::default();
        hash_engine.input(&xpubkey.public_key.key.serialize_uncompressed());
        let pubkey_hash: &[u8] = &Sha2Hash::from_engine(hash_engine);
        let encoded = bech32::encode("arbiter", pubkey_hash.to_base32()).unwrap();
        ArbiterId(encoded)
    }
}

pub struct Arbiter {
    id:     ArbiterId,
}
