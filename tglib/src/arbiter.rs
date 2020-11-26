use std::convert::{
    From,
};
use bdk::bitcoin::{
    Address,
    PublicKey,
    Transaction,
    bech32::{
        self,
        FromBase32,
        ToBase32,
    },
    hashes::{
        Hash,
        HashEngine,
        sha256::HashEngine as Sha2Engine,
        sha256::Hash as Sha2Hash,
    },
    secp256k1::{
        Signature,
    },
    util::bip32::{
        ExtendedPubKey,
    },
};
use crate::{
    Result,
    TgError,
    contract::Contract,
    payout::Payout,
};

#[derive(Debug, Default, Clone)]
pub struct ArbiterId(pub String); 

impl From<ExtendedPubKey> for ArbiterId {
    fn from(xpubkey: ExtendedPubKey) -> Self {
        let mut hash_engine = Sha2Engine::default();
// extended pubkey -> bitcoin pubkey wrapper -> actual pubkey
        hash_engine.input(&xpubkey.public_key.key.serialize_uncompressed());
        let pubkey_hash: &[u8] = &Sha2Hash::from_engine(hash_engine);
        let encoded = bech32::encode("arbiter", pubkey_hash.to_base32()).unwrap();
        ArbiterId(encoded)
    }
}

pub struct Arbiter {
    id:     ArbiterId,
}

pub trait ArbiterService {
    fn get_escrow_pubkey(&self) -> Result<PublicKey>;
    fn get_fee_address(&self) -> Result<Address>;
    fn submit_contract(&self, contract: &Contract) -> Result<Signature>;
    fn submit_payout(&self, payout: &Payout) -> Result<Transaction>;
}
