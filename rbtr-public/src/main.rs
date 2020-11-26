use std::{
    str::FromStr,
    sync::{
        Arc,
    },
};
use warp::{
    Filter,
};
use hex::decode;
use bdk::{
    bitcoin::{
        Address,
        util::{
            bip32::{
                ExtendedPubKey,
                Fingerprint,
            }
        }
    },
};
use bip39::Mnemonic;
use tglib::{
    contract::Contract,
    payout::Payout,
    wallet::{
        EscrowWallet,
        SigningWallet,
    },
    mock::{
        Trezor,
        ARBITER_FINGERPRINT,
        ARBITER_XPUBKEY,
        NETWORK,
    },
};

mod wallet;
use wallet::Wallet;

fn wallet() -> Wallet {
    Wallet::new(Fingerprint::from_str(ARBITER_FINGERPRINT).unwrap(), ExtendedPubKey::from_str(ARBITER_XPUBKEY).unwrap(), NETWORK)
}

#[tokio::main]
async fn main() {

    let escrow_pubkey = wallet().get_escrow_pubkey();
    let fee_address = warp::any().map(move || Address::p2wpkh(&escrow_pubkey, NETWORK).unwrap());
    let escrow_pubkey = warp::any().map(move || escrow_pubkey.clone());

    let get_escrow_pubkey = warp::path("escrow-pubkey")
        .and(escrow_pubkey)
        .map(|e| format!("escrow_pubkey: {:?}", e)); 

    let get_fee_address = warp::path("fee-address")
        .and(fee_address)
        .map(|f| format!("fee address:   {:?}", f)); 

    let submit_contract = warp::path("submit-contract")
        .and(warp::path::param::<String>())
        .map(|contract_hex| {
            match Contract::from_bytes(hex::decode(contract_hex).unwrap()) {
                Ok(c) => {
                    match wallet().validate_contract(&c) {
                        Ok(_) => format!("contract: {:?}", c),
                        Err(e) => format!("err: {:?}", e)
                    }
                },
                Err(e) => format!("err:      {:?}", e),
            }
        });

    let submit_payout = warp::path("submit-payout")
        .and(warp::path::param::<String>())
        .map(|payout_hex| {
            match Payout::from_bytes(hex::decode(payout_hex).unwrap()) {
                Ok(p) => {
                    match wallet().validate_payout(&p) {
                        Ok(_) => format!("payout: {:?}", p),
                        Err(e) => format!("err: {:?}", e)
                    }
                },
                Err(e) => format!("err:    {:?}", e),
            }
        });

    let routes = get_escrow_pubkey
        .or(get_fee_address)
        .or(submit_contract)
        .or(submit_payout);

    warp::serve(routes).run(([127, 0, 0, 1], 3030)).await;
}
