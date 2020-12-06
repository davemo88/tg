use std::{
    env::current_dir,
    path::PathBuf,
};
//use hex::{decode, encode};
use clap::{App, SubCommand, AppSettings};
use rustyline::Editor;
use rustyline::error::ReadlineError;
use shell_words;
use tglib::{
    bdk::Error,
    bip39::Mnemonic,
    wallet::SigningWallet,
    mock::{
        Trezor,
        NETWORK,
        PLAYER_1_MNEMONIC,
    },
};
use player_wallet::{
//    arbiter,
//    db,
    ui::{
        self,
        contract_subcommand,
        payout_subcommand,
        player_subcommand,
        wallet_subcommand,
    },
    wallet::PlayerWallet,
};

fn repl<'a, 'b>() -> App<'a, 'b> {
    App::new("repl")
        .version(option_env!("CARGO_PKG_VERSION").unwrap_or("unknown"))
        .author(option_env!("CARGO_PKG_AUTHORS").unwrap_or(""))
        .about("wallet repl")
        .settings(&[AppSettings::NoBinaryName, AppSettings::SubcommandRequiredElseHelp,
            AppSettings::VersionlessSubcommands])
        .subcommand(SubCommand::with_name("quit").about("quit the repl"))
        .subcommand(ui::wallet_ui())
        .subcommand(ui::player_ui())
        .subcommand(ui::contract_ui())
        .subcommand(ui::payout_ui())
}

fn main() -> Result<(), Error> {

    let work_dir: PathBuf = current_dir().unwrap();
    let mut history_file = work_dir.clone();
    history_file.push(&NETWORK.to_string());
    history_file.push("history.txt");
    let history_file = history_file.as_path();

    let mut rl = Editor::<()>::new();

    if rl.load_history(history_file).is_err() {
        println!("No previous history.");
    }

    let signing_wallet = Trezor::new(Mnemonic::parse(PLAYER_1_MNEMONIC).unwrap());
    let wallet = PlayerWallet::new(signing_wallet.fingerprint(), signing_wallet.xpubkey(), NETWORK);

    loop {
        let readline = rl.readline(">> ");
        match readline {
            Ok(line) => {
                let split_line = shell_words::split(&line).unwrap();
                let matches = repl().get_matches_from_safe(split_line);
                if matches.is_ok() {
                    if let (c, Some(a)) = matches.unwrap().subcommand() {
//                        println!("command: {}, args: {:?}", c, a);
                        rl.add_history_entry(line.as_str());
                        match c {
                            "wallet" => {
                                let _ = wallet_subcommand(a.subcommand(), &wallet);
                            }
                            "player" => {
                                let _ = player_subcommand(a.subcommand(), &wallet);
                            }
                            "contract" => {
                                let _ = contract_subcommand(a.subcommand(), &wallet);
                            }
                            "payout" => {
                                let _ = payout_subcommand(a.subcommand(), &wallet);
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
