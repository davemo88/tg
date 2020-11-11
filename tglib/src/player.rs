use std::convert::{
    From,
    Into,
    TryInto,
};
use bdk::{
    bitcoin::{
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
        secp256k1::{
            PublicKey,
        },
    },
    descriptor::{
        Descriptor,
        MiniscriptKey,
    }
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
