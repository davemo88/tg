use std::{
    cmp::max,
    collections::HashMap,
    convert::{
        Into,
        TryFrom,
    },
    env::current_dir,
    net::{
        AddrParseError, 
        SocketAddr
    },
    path::{
        PathBuf, 
        Path,
    },
    process::ChildStderr,
    str::FromStr,
    thread,
    time::Duration,
};
use hex::{decode, encode};
use log::debug;
use serde::{
    Serialize,
};
use bdk::{
    bitcoin::{
        Address,
        Amount,
        secp256k1::{
            Secp256k1,
            Message,
        },
        util::{
            psbt::PartiallySignedTransaction,
            bip32::{
                DerivationPath,
            }
        },
        hashes::sha256
    },
    blockchain::{
        noop_progress,
    },
    Error,
};
use bip39::{
    Mnemonic, 
};
use clap::{App, Arg, ArgMatches, SubCommand, AppSettings};
use rustyline::Editor;
use rustyline::error::ReadlineError;
use shell_words;
use tglib::{
    Result as TgResult,
    TgError,
    arbiter::{
        ArbiterId,
    },
    contract::{
        Contract,
    },
    payout::{
        Payout,
    },
    player::{
        PlayerId,
    },
    script::TgScript,
    wallet::{
        SigningWallet,
    }
};

mod db;
mod mock;
mod ui;
mod wallet;
use mock::{
    ArbiterService,
    PlayerInfoService,
    Trezor,
    ARBITER_MNEMONIC,
    DB_NAME,
    NETWORK,
    PLAYER_1_MNEMONIC,
    PLAYER_2_MNEMONIC,
    BITCOIN_DERIVATION_PATH,
    ESCROW_SUBACCOUNT,
    ESCROW_KIX,
};
use wallet::{
    PlayerWallet,
};
use ui::wallet_subcommand;

fn repl<'a, 'b>() -> App<'a, 'b> {
    App::new("repl")
        .version(option_env!("CARGO_PKG_VERSION").unwrap_or("unknown"))
        .author(option_env!("CARGO_PKG_AUTHORS").unwrap_or(""))
        .about("wallet repl")
        .settings(&[AppSettings::NoBinaryName, AppSettings::SubcommandRequiredElseHelp,
            AppSettings::VersionlessSubcommands])
        .subcommand(SubCommand::with_name("quit").about("quit the repl"))
        .subcommand(ui::wallet_ui())
}

fn main() -> Result<(), Error> {

    let work_dir: PathBuf = current_dir().unwrap();
    let mut history_file = work_dir.clone();
    history_file.push(&NETWORK.to_string());
    history_file.push("history.txt");
    let history_file = history_file.as_path();

//    let signing_wallet = Trezor::new(Mnemonic::parse(PLAYER_1_MNEMONIC).unwrap());
//
//    let wallet = PlayerWallet::new(signing_wallet.fingerprint(), signing_wallet.xpubkey(), NETWORK);

    let mut rl = Editor::<()>::new();

    if rl.load_history(history_file).is_err() {
        println!("No previous history.");
    }

    loop {
        let readline = rl.readline(">> ");
        match readline {
            Ok(line) => {
                let split_line = shell_words::split(&line).unwrap();
                let matches = repl().get_matches_from_safe(split_line);
                if matches.is_ok() {
                    if let (c, Some(a)) = matches.unwrap().subcommand() {
                        debug!("command: {}, args: {:?}", c, a);
                        rl.add_history_entry(line.as_str());
                        match c {
                            "wallet" => {
                                wallet_subcommand(a.subcommand());
                            }
                            "quit" => {
                                break;
                            }
                            _ => {
                                println!("command '{}' is not implemented", c);
                            }
                        }
                    }
                } else {
                    let err = matches.err().unwrap();
                    println!("{}", err);
                }
            }
            Err(ReadlineError::Interrupted) => {
                println!("CTRL-C");
                break;
            }
            Err(ReadlineError::Eof) => {
                println!("CTRL-D");
                break;
            }
            Err(err) => {
                println!("Error: {:?}", err);
                break;
            }
        }
    }
    rl.save_history(history_file).unwrap();
    println!("stopping");
    println!("stopped");

    Ok(())

}

#[cfg(test)]
mod tests {

    use super::*;
    use mock::{
        BITCOIN_RPC_URL,
    };
    use bitcoincore_rpc::{Auth, Client as RpcClient, RpcApi, json::EstimateMode};
    use tglib::wallet::{
        create_payout_script,
        create_escrow_address,
    };

    const SATS: u64 = 1000000;

    #[test]
    fn fund_players() {
        let p1_signing_wallet = Trezor::new(Mnemonic::parse(PLAYER_1_MNEMONIC).unwrap()); 
        let p1_wallet = PlayerWallet::new(p1_signing_wallet.fingerprint(), p1_signing_wallet.xpubkey(), NETWORK);
        let p1_addr = p1_wallet.new_address();

        let p2_signing_wallet = Trezor::new(Mnemonic::parse(PLAYER_2_MNEMONIC).unwrap()); 
        let p2_wallet = PlayerWallet::new(p2_signing_wallet.fingerprint(), p2_signing_wallet.xpubkey(), NETWORK);
        let p2_addr = p2_wallet.new_address();

        let rpc = RpcClient::new(BITCOIN_RPC_URL.to_string(), Auth::UserPass("admin".to_string(), "passw".to_string())).unwrap();
        let coinbase_addr = rpc.get_new_address(None, None).unwrap();
        let txid1 = rpc.send_to_address(&p1_addr, Amount::ONE_BTC, None, None, None, None, None, Some(EstimateMode::Conservative)).unwrap();
        let txid2 = rpc.send_to_address(&p2_addr, Amount::ONE_BTC, None, None, None, None, None,Some(EstimateMode::Conservative)).unwrap();
        let blockhashes = rpc.generate_to_address(150, &coinbase_addr).unwrap();
// electrs needs some time to catch up to bitcoind i think?
        thread::sleep(Duration::new(5,0));
    }

    fn create_contract() -> Contract {
        let p1_signing_wallet = Trezor::new(Mnemonic::parse(PLAYER_1_MNEMONIC).unwrap()); 
        let p1_wallet = PlayerWallet::new(p1_signing_wallet.fingerprint(), p1_signing_wallet.xpubkey(), NETWORK);
        let p2_contract_info = PlayerInfoService::get_contract_info(&PlayerId(String::from("player 2")));
        let arbiter_pubkey = ArbiterService::get_escrow_pubkey();
        p1_wallet.create_contract(p2_contract_info, Amount::from_sat(SATS), arbiter_pubkey)
    }

    fn create_signed_contract() -> Contract {
        let p1_signing_wallet = Trezor::new(Mnemonic::parse(PLAYER_1_MNEMONIC).unwrap()); 
        let p1_wallet = PlayerWallet::new(p1_signing_wallet.fingerprint(), p1_signing_wallet.xpubkey(), NETWORK);
        let p2_contract_info = PlayerInfoService::get_contract_info(&PlayerId(String::from("player 2")));
        let arbiter_pubkey = ArbiterService::get_escrow_pubkey();
        let mut contract = p1_wallet.create_contract(p2_contract_info, Amount::from_sat(SATS), arbiter_pubkey);
        let cxid = contract.cxid();
        
        let p1_sig = p1_signing_wallet.sign_message(
            Message::from_slice(&cxid).unwrap(),
            DerivationPath::from_str(&format!("m/{}/{}/{}", BITCOIN_DERIVATION_PATH, ESCROW_SUBACCOUNT, ESCROW_KIX)).unwrap(),
        ).unwrap();

        
        let p2_signing_wallet = Trezor::new(Mnemonic::parse(PLAYER_2_MNEMONIC).unwrap()); 
        let p2_wallet = PlayerWallet::new(p2_signing_wallet.fingerprint(), p2_signing_wallet.xpubkey(), NETWORK);
        let p2_sig = p2_signing_wallet.sign_message(
            Message::from_slice(&cxid).unwrap(),
            DerivationPath::from_str(&format!("m/{}/{}/{}", BITCOIN_DERIVATION_PATH, ESCROW_SUBACCOUNT, ESCROW_KIX)).unwrap(),
        ).unwrap();

        let arbiter_wallet = Trezor::new(Mnemonic::parse(ARBITER_MNEMONIC).unwrap()); 
        let arbiter_sig = arbiter_wallet.sign_message(
            Message::from_slice(&cxid).unwrap(),
            DerivationPath::from_str(&format!("m/{}/{}/{}", BITCOIN_DERIVATION_PATH, ESCROW_SUBACCOUNT, ESCROW_KIX)).unwrap(),
        ).unwrap();

        contract.sigs.push(p1_sig);
        contract.sigs.push(p2_sig);
        contract.sigs.push(arbiter_sig);

        contract
    }

    #[test]
    fn validate_contract() {
// need to check:
// funding_tx is valid
//  spends contract amount to escrow address
//  escrow address is standard multisig with keys
//  p1_pubkey, p2_pubkey, arbiter_pubkey in that order
//  spends fee to arbiter's fee address
// payout_script follows standard template
// signed contract only:
// verify signatures
        let contract = create_signed_contract();

        let arbiter_pubkey = ArbiterService::get_escrow_pubkey();

        let escrow_address = create_escrow_address(
            &contract.p1_pubkey,
            &contract.p2_pubkey,
            &arbiter_pubkey,
            NETWORK,
        ).unwrap();
        let escrow_script_pubkey = escrow_address.script_pubkey();
        let amount = SATS;
        let fee_address = ArbiterService::get_fee_address();
        let fee_script_pubkey = fee_address.script_pubkey();
        let fee_amount = amount/100;

        let mut matching_escrow_address = false;
        let mut correct_fee = false;

        for txout in contract.funding_tx.clone().output {
            matching_escrow_address = (txout.script_pubkey == escrow_script_pubkey && txout.value == amount) || matching_escrow_address;
            correct_fee = (txout.script_pubkey == fee_script_pubkey && txout.value == fee_amount) || correct_fee; 
        }

        assert!(matching_escrow_address);
        assert!(correct_fee);

        let payout_script = create_payout_script(
            &contract.p1_pubkey,
            &contract.p2_pubkey,
            &arbiter_pubkey,
            &contract.funding_tx,
            NETWORK,
        );

        assert_eq!(contract.payout_script, payout_script);

        assert_eq!(contract.sigs.len(), 3);
        let secp = Secp256k1::new();
        let msg = Message::from_slice(&contract.cxid()).unwrap();
        assert!(secp.verify(&msg, &contract.sigs[0], &contract.p1_pubkey.key).is_ok());
        assert!(secp.verify(&msg, &contract.sigs[1], &contract.p2_pubkey.key).is_ok());
        assert!(secp.verify(&msg, &contract.sigs[2], &arbiter_pubkey.key).is_ok());
    }

    #[test] 
    fn create_payout() {

    }

    #[test] 
    fn sign_payout() {

    }

    #[test] 
    fn validate_payout() {

    }

    #[test] 
    fn reject_invalid_payout() {

    }
}
