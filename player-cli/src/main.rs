use std::{
    env::current_dir,
    path::PathBuf,
};
use clap::{App, Arg, ArgMatches, SubCommand, AppSettings};
use rustyline::Editor;
use rustyline::error::ReadlineError;
use shell_words;
use tglib::{
    bdk::{
        Error,
        electrum_client::Client,
        bitcoin::{
            Amount,
            secp256k1::Signature,
        },
    },
    bip39::Mnemonic,
    hex,
    player::PlayerName,
    wallet::SigningWallet,
    mock::{
        Trezor,
        NETWORK,
        PLAYER_1_MNEMONIC,
    },
};
use player_wallet::{
    db::{
        ContractRecord,
        PayoutRecord,
    },
    ui::{
        DocumentUI,
        NewDocumentParams,
        PlayerUI,
        WalletUI,
        SignDocumentParams,
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
        .subcommand(SubCommand::with_name("balance").about("display balance (sats)"))
        .subcommand(SubCommand::with_name("deposit").about("display a deposit address"))
        .subcommand(SubCommand::with_name("withdraw").about("withdraw amount to address")
            .arg(Arg::with_name("amount")
                .index(1)
                .help("withdrawal amount (sats)")
                .required(true))
            .arg(Arg::with_name("address")
                .index(2)
                .help("withdrawal address")
                .required(true)))
        .subcommand(player_ui())
        .subcommand(contract_ui())
        .subcommand(payout_ui())
}

pub fn player_ui<'a, 'b>() -> App<'a, 'b> {
    App::new("player")
        .version(option_env!("CARGO_PKG_VERSION").unwrap_or("unknown"))
        .author(option_env!("CARGO_PKG_AUTHORS").unwrap_or(""))
        .about("player commands")
        .settings(&[AppSettings::NoBinaryName, AppSettings::SubcommandRequiredElseHelp,
            AppSettings::VersionlessSubcommands])
        .subcommands(vec![
            SubCommand::with_name("register").about("register new player")
                .arg(Arg::with_name("name")
                    .index(1)
                    .help("new player name")
                    .required(true)),
            SubCommand::with_name("add").about("add to known players")
                .arg(Arg::with_name("name")
                    .index(1)
                    .help("player name")
                    .required(true)),
            SubCommand::with_name("remove").about("remove from known players")
                .arg(Arg::with_name("name")
                    .index(1)
                    .help("name of player to remove")
                    .required(true)),
            SubCommand::with_name("list").about("list known players"),
            SubCommand::with_name("mine").about("show local player names"),
        ])
}

pub fn player_subcommand(subcommand: (&str, Option<&ArgMatches>), wallet: &PlayerWallet) {
    if let (c, Some(a)) = subcommand {
        match c {
            "register" => match wallet.register(PlayerName(a.value_of("name").unwrap().to_string())) {
                Ok(()) => println!("registered player"),
                Err(e) => println!("{}", e),
            }
            "add" => match wallet.add(PlayerName(a.value_of("name").unwrap().to_string())) {
                Ok(()) => println!("added player"),
                Err(e) => println!("{}", e),
            }
            "remove" => match wallet.remove(PlayerName(a.value_of("name").unwrap().to_string())) {
                Ok(()) => println!("removed player"),
                Err(e) => println!("{}", e),
            }
            "list" => PlayerUI::list(wallet).iter().for_each(|p| println!("{}", p.name.0) ),
//            "mine" => for p in wallet.mine() { println!("{}", p.0) },
            "mine" => wallet.mine().iter().for_each(|p| println!("{}", p.0) ),
            _ => { 
                println!("command '{}' is not implemented", c);
            }
        }
    }
}

pub fn contract_ui<'a, 'b>() -> App<'a, 'b> {
    App::new("contract")
        .version(option_env!("CARGO_PKG_VERSION").unwrap_or("unknown"))
        .author(option_env!("CARGO_PKG_AUTHORS").unwrap_or(""))
        .about("contract commands")
        .settings(&[AppSettings::NoBinaryName, AppSettings::SubcommandRequiredElseHelp,
            AppSettings::VersionlessSubcommands])
        .subcommands(vec![
            SubCommand::with_name("new").about("create a new contract")
                .arg(Arg::with_name("player-1")
                    .index(1)
                    .help("player 1's name")
                    .required(true)
                    .takes_value(true))
                .arg(Arg::with_name("player-2")
                    .index(2)
                    .help("player 2's name")
                    .required(true)
                    .takes_value(true))
                .arg(Arg::with_name("amount")
                    .index(3)
                    .help("amount")
                    .required(true)
                    .takes_value(true))
                .arg(Arg::with_name("desc")
                    .short("d")
                    .long("desc")
                    .value_name("DESC")
                    .help("description")
                    .takes_value(true)),
            SubCommand::with_name("import").about("import contract")
                .arg(Arg::with_name("contract-value")
                    .index(1)
                    .help("contract record json or hex-encoded contract")
                    .required(true)
                    .takes_value(true)),
            SubCommand::with_name("export").about("export contract as hex")
                .arg(Arg::with_name("cxid")
                    .index(1)
                    .help("contract id")
                    .required(true)
                    .takes_value(true)),
            SubCommand::with_name("details").about("show contract details")
                .arg(Arg::with_name("cxid")
                    .index(1)
                    .value_name("CXID")
                    .help("contract id")
                    .required(true)
                    .takes_value(true)),
            SubCommand::with_name("sign").about("sign contract")
                .arg(Arg::with_name("cxid")
                    .index(1)
                    .value_name("CXID")
                    .help("contract id")
                    .required(true)
                    .takes_value(true))
                .arg(Arg::with_name("sign-funding-tx")
                    .long("sign-funding-tx")
                    .help("sign the funding tx as well as the contract")),
            SubCommand::with_name("submit").about("submit contract to arbiter")
                .arg(Arg::with_name("cxid")
                    .index(1)
                    .value_name("CXID")
                    .help("contract id")
                    .required(true)
                    .takes_value(true)),
            SubCommand::with_name("broadcast").about("broadcast funding tx")
                .arg(Arg::with_name("cxid")
                    .index(1)
                    .value_name("CXID")
                    .help("contract id")
                    .required(true)
                    .takes_value(true)),
            SubCommand::with_name("delete").about("delete contract")
                .arg(Arg::with_name("cxid")
                    .index(1)
                    .value_name("CXID")
                    .help("contract id")
                    .required(true)
                    .takes_value(true)),
            SubCommand::with_name("list").about("list all contracts"),
        ])
}

pub fn contract_subcommand(subcommand: (&str, Option<&ArgMatches>), wallet: &PlayerWallet) {
    if let (c, Some(a)) = subcommand {
        match c {
            "new" => match DocumentUI::<ContractRecord>::new(
                wallet,
                NewDocumentParams::NewContractParams {
                    p1_name: PlayerName(a.value_of("player-1").unwrap().to_string()),
                    p2_name: PlayerName(a.value_of("player-2").unwrap().to_string()),
                    amount: Amount::from_sat(a.value_of("amount").unwrap().parse::<u64>().unwrap()),
                    desc: match a.value_of("desc") {
                        Some(d) => Some(String::from(d)),
                        None => None,
                    } 
                }) {
                Ok(()) => println!("contract created"),
                Err(e) => println!("{}", e),
            }
            "import" => match DocumentUI::<ContractRecord>::import(wallet, &a.value_of("contract-value").unwrap()) {
// TODO: print cxid of imported contract
                Ok(()) => println!("contract imported"),
                Err(e) => println!("{}", e),
            }
            "export" => match DocumentUI::<ContractRecord>::export(wallet, &a.value_of("cxid").unwrap()) {
                Some(hex) => println!("{}", hex),
                None => println!("no such contract"),
            }
            "details" => match DocumentUI::<ContractRecord>::get(wallet, a.value_of("cxid").unwrap()) {
                Some(cr) => println!("{:?}", cr),
                None => println!("no such contract"),
            }
            "sign" => match DocumentUI::<ContractRecord>::sign(
                wallet,
                SignDocumentParams::SignContractParams {
                    cxid: a.value_of("cxid").unwrap().to_string(),
                    sign_funding_tx: a.value_of("sign-funding-tx").is_some(),
                }) {
                Ok(()) => println!("contract created"),
                Err(e) => println!("{}", e),
            }
            "submit" => match DocumentUI::<ContractRecord>::submit(wallet, a.value_of("cxid").unwrap()) {
                Ok(()) => println!("submission accepted"),
                Err(e) => println!("{}", e),
            }
            "broadcast" => match DocumentUI::<ContractRecord>::broadcast(wallet, a.value_of("cxid").unwrap()) {
                Ok(()) => println!("funding tx broadcasted to network"),
                Err(e) => println!("{}", e),
            }
            "delete" => match DocumentUI::<ContractRecord>::delete(wallet, a.value_of("cxid").unwrap()) {
                Ok(()) => println!("contract deleted"),
                Err(e) => println!("{}", e),
            }
            "list" => for c in DocumentUI::<ContractRecord>::list(wallet) { println!("{:?}", c) },
            _ => {
                println!("command '{}' is not implemented", c);
            }
        }            
    }
}

pub fn payout_ui<'a, 'b>() -> App<'a, 'b> {
    App::new("payout")
        .version(option_env!("CARGO_PKG_VERSION").unwrap_or("unknown"))
        .author(option_env!("CARGO_PKG_AUTHORS").unwrap_or(""))
        .about("payout commands")
        .settings(&[AppSettings::NoBinaryName, AppSettings::SubcommandRequiredElseHelp,
            AppSettings::VersionlessSubcommands])
        .subcommands(vec![
            SubCommand::with_name("new").about("create a new payout")
                .arg(Arg::with_name("cxid")
                    .index(1)
                    .help("which contract to pay out")
                    .required(true)
                    .takes_value(true))
                .arg(Arg::with_name("player")
                    .index(2)
                    .help("which player to pay")
                    .required(true)
                    .takes_value(true))
                .arg(Arg::with_name("amount")
                    .index(3)
                    .help("how much to pay the specified player. remainder goes to other player")
                    .required(true)
                    .takes_value(true)),
            SubCommand::with_name("import").about("import payout")
                .arg(Arg::with_name("payout-value")
                    .index(1)
                    .help("payout record or hex-encoded payout")
                    .required(true)
                    .takes_value(true)),
            SubCommand::with_name("export").about("export payout as hex")
                .arg(Arg::with_name("cxid")
                    .index(1)
                    .help("payout contract id")
                    .required(true)
                    .takes_value(true)),
            SubCommand::with_name("details").about("show payout details")
                .arg(Arg::with_name("cxid")
                    .index(1)
                    .value_name("CXID")
                    .help("contract id of payout")
                    .required(true)
                    .takes_value(true)),
            SubCommand::with_name("sign").about("sign payout tx and optionally set script sig")
                .arg(Arg::with_name("cxid")
                    .index(1)
                    .help("contract id of payout to sign")
                    .required(true)
                    .takes_value(true))
                .arg(Arg::with_name("script-sig")
                    .index(2)
                    .help("payout script sig")
                    .takes_value(true)),
            SubCommand::with_name("submit").about("submit payout to arbiter")
                .arg(Arg::with_name("cxid")
                    .index(1)
                    .value_name("CXID")
                    .help("contract id for the payout")
                    .required(true)
                    .takes_value(true)),
            SubCommand::with_name("broadcast").about("broadcast payout tx")
                .arg(Arg::with_name("cxid")
                    .index(1)
                    .value_name("CXID")
                    .help("contract id for the payout tx")
                    .required(true)
                    .takes_value(true)),
            SubCommand::with_name("delete").about("delete payout")
                .arg(Arg::with_name("cxid")
                    .index(1)
                    .value_name("CXID")
                    .help("contract id of payout")
                    .required(true)
                    .takes_value(true)),
            SubCommand::with_name("list").about("list all payouts"),
        ])
}

pub fn payout_subcommand(subcommand: (&str, Option<&ArgMatches>), wallet: &PlayerWallet) {
    if let (c, Some(a)) = subcommand {
        match c {
            "new" => match DocumentUI::<PayoutRecord>::new(
                wallet,
                NewDocumentParams::NewPayoutParams {
                    cxid: a.value_of("cxid").unwrap().to_string(),
                    name: PlayerName(a.value_of("player").unwrap().to_string()),
                    amount: Amount::from_sat(a.value_of("amount").unwrap().parse::<u64>().unwrap()),
                }) {
                Ok(()) => println!("payout created"),
                Err(e) => println!("{}", e),
            }
            "import" => match DocumentUI::<PayoutRecord>::import(wallet, a.value_of("payout-value").unwrap()) {
// TODO: print cxid of imported payout
                Ok(()) => println!("payout imported"),
                Err(e) => println!("{}", e),
            }
            "export" => match DocumentUI::<PayoutRecord>::export(wallet, a.value_of("cxid").unwrap()) {
                Some(hex) => println!("{}", hex),
                None => println!("no such payout"),
            }
            "details" => match DocumentUI::<PayoutRecord>::get(wallet, a.value_of("cxid").unwrap()) {
                Some(pr) => println!("{:?}", pr),
                None => println!("no such payout"),
            }
            "sign" => match DocumentUI::<PayoutRecord>::sign(
                wallet,
                SignDocumentParams::SignPayoutParams {
                    cxid: a.value_of("cxid").unwrap().to_string(),
                    script_sig: match a.value_of("script_sig") {
                        Some(sig_hex) => Some(Signature::from_compact(&hex::decode(sig_hex).unwrap()).unwrap()),
                        None => None,
                    },
                }) {
                Ok(()) => println!("payout created"),
                Err(e) => println!("{}", e),
            }
            "submit" => match DocumentUI::<PayoutRecord>::submit(wallet, a.value_of("cxid").unwrap()) {
                Ok(()) => println!("submission accepted"),
                Err(e) => println!("{}", e),
            }
            "broadcast" => match DocumentUI::<PayoutRecord>::broadcast(wallet, a.value_of("cxid").unwrap()) {
                Ok(()) => println!("funding tx broadcasted to network"),
                Err(e) => println!("{}", e),
            }
            "delete" => match DocumentUI::<PayoutRecord>::delete(wallet, a.value_of("cxid").unwrap()) {
                Ok(()) => println!("payout deleted"),
                Err(e) => println!("{}", e),
            }
            "list" => for p in DocumentUI::<PayoutRecord>::list(wallet) { println!("{:?}", p) },
            _ => {
                println!("command '{}' is not implemented", c);
            }
        }            
    }
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
    let client = Client::new("tcp://localhost:60401").unwrap();
    let wallet = PlayerWallet::new(signing_wallet.fingerprint(), signing_wallet.xpubkey(), NETWORK, client);

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
                            "balance" => {
                                println!("{}", wallet.balance());
                            }
                            "deposit" => {
                                println!("{}", wallet.deposit());
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
