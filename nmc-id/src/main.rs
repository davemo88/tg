use tglib::{
    bdk::{
        bitcoin::{
            PublicKey,

        },
        blockchain::{
            noop_progress,
            ElectrumBlockchain,
        },
        database::MemoryDatabase,
        electrum_client::Client,
    },
    bip39::Mnemonic,
    contract::{
        PlayerContractInfo,
    },
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
        ELECTRS_SERVER,
        NETWORK,
    }
};

struct NmcId;

impl PlayerIdentityService for NmcId {
    fn get_player_id(&self, pubkey: &PublicKey) -> Option<PlayerId> {
        None
    }

    fn get_player_info(&self, player_id: PlayerId) -> Option<PlayerContractInfo> {
        let escrow_pubkey = EscrowWallet::get_escrow_pubkey(&player_wallet);
        player_wallet.wallet.sync(noop_progress(), None).unwrap();
        Some(PlayerContractInfo {
            escrow_pubkey,
// TODO: send to internal descriptor, no immediate way to do so atm
            change_address: player_wallet.wallet.get_new_address().unwrap(),
// can't hardcode these effectively so kinda need access to the player wallet
// to do that, need to player player wallet a lib instead of binary i believe
// and need to do that any for android. however then need to make a separate 
// package for cli wallet lol seems fine
            utxos: player_wallet.wallet.list_unspent().unwrap(),
        })
    }
}

fn main() {
    println!("Hello, world!");
}
