use std::str::FromStr;
use tglib::{
    player::PlayerId,
};
use bdk::bitcoin::PublicKey;

const ARBITER_PUBKEY: &'static str = "bogusarbiterpubkey";
const PLAYER_PUBKEY: &'static str = "bogusplayerpubkey";

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
