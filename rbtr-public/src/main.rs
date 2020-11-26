use std::{
    sync::{ 
        Arc, 
        atomic::{AtomicUsize, Ordering},
    },
};
use futures::{FutureExt, StreamExt};
use tokio::sync::{
    mpsc,
    RwLock,
};
use warp::{
    Filter,
    path,
};
use bip39::Mnemonic;
use tglib::{
    wallet::{
        EscrowWallet,
        SigningWallet,
    },
    mock::{
        Trezor,
        ARBITER_MNEMONIC,
        NETWORK,
    },
};

mod wallet;
use wallet::{
    Wallet,
};

type WebWallet = Arc<RwLock<Wallet>>;

#[tokio::main]
async fn main() {

    let signing_wallet = Trezor::new(Mnemonic::parse(ARBITER_MNEMONIC).unwrap());
    let wallet: WebWallet = Arc::from(RwLock::from(Wallet::new(signing_wallet.fingerprint(), signing_wallet.xpubkey(), NETWORK)));
    let w = warp::any().map(move || wallet.clone());

    let get_escrow_pubkey = warp::path("get-escrow-pubkey")
        .and(w)
        .map(|w| format!("escrow_pubkey: {:?}","lol")); 

    warp::serve(get_escrow_pubkey)
        .run(([127, 0, 0, 1], 3030))
        .await;
}
