use tglib::{
    bdk::{
        bitcoin::PublicKey,
        blockchain::noop_progress,
    },
    bip39::Mnemonic,
    contract::PlayerContractInfo,
    player::{
        PlayerId,
        PlayerIdentityService,
    },
    wallet::{
        EscrowWallet,
        SigningWallet,
    },
    mock::{
        Trezor,
        PLAYER_2_MNEMONIC,
        NETWORK,
    }
};
use player_wallet::wallet::PlayerWallet;

struct NmcId;

impl PlayerIdentityService for NmcId {
    fn get_player_id(&self, _pubkey: &PublicKey) -> Option<PlayerId> {
        None
    }

    fn get_player_info(&self, _player_id: PlayerId) -> Option<PlayerContractInfo> {
        let signing_wallet = Trezor::new(Mnemonic::parse(PLAYER_2_MNEMONIC).unwrap());
        let player_wallet = PlayerWallet::new(signing_wallet.fingerprint(), signing_wallet.xpubkey(), NETWORK);
        let escrow_pubkey = EscrowWallet::get_escrow_pubkey(&player_wallet);
        player_wallet.wallet.sync(noop_progress(), None).unwrap();
        Some(PlayerContractInfo {
            escrow_pubkey,
// TODO: send to internal descriptor, no immediate way to do so atm
            change_address: player_wallet.wallet.get_new_address().unwrap(),
            utxos: player_wallet.wallet.list_unspent().unwrap(),
        })
    }
}

fn main() {
    println!("Hello, world!");
}
