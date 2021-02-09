use std::{
    env::current_dir,
    fs::File,
    io::Write,
    path::PathBuf,
};
use clap::{App, Arg, ArgMatches, SubCommand, AppSettings};
use serde_json;
use shell_words;
use tglib::{
    bdk::{
        electrum_client::Client,
        bitcoin::{
            Amount,
            secp256k1::Signature,
        },
    },
    hex,
    secrecy::Secret,
    player::PlayerName,
    wallet::SavedSeed,
    mock::{
        DB_NAME,
        NETWORK,
        SEED_NAME,
    },
};
use player_wallet::{
    arbiter::ArbiterClient,
    db::{
        DB,
        ContractRecord,
        PayoutRecord,
    },
    player::PlayerNameClient,
    ui::{
        DocumentUI,
        NewDocumentParams,
        PlayerUI,
        WalletUI,
        SignDocumentParams,
    },
    wallet::PlayerWallet,
};

fn player_cli<'a, 'b>() -> App<'a, 'b> {
    App::new("player-cli")
        .version(option_env!("CARGO_PKG_VERSION").unwrap_or("unknown"))
        .author(option_env!("CARGO_PKG_AUTHORS").unwrap_or(""))
        .about("player cli")
        .settings(&[AppSettings::NoBinaryName, AppSettings::SubcommandRequiredElseHelp,
            AppSettings::VersionlessSubcommands])
        .subcommand(SubCommand::with_name("init").about("initialize new wallet")
            .arg(Arg::with_name("passphrase")
                .long("passphrase")
                .index(1)
                .required(true)
                .takes_value(true)
                .help("wallet passphrase"))
            .arg(Arg::with_name("seed-phrase")
                .long("seed-phrase")
                .required(false)
                .takes_value(true)
                .help("BIP39 seed phrase")))
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
        .arg(Arg::with_name("json-output")
            .help("output json instead of user-friendly messages")
            .global(true)
            .required(false)
            .takes_value(false)
            .long("json-output"))
        .arg(Arg::with_name("wallet-path")
            .help("specify path to wallet")
            .required(false)
            .global(true)
            .takes_value(true)
            .long("wallet-path"))
}

pub struct Conf {
    pub electrum_url: String,
    pub name_url: String,
    pub arbiter_url: String,
}

pub fn cli(line: String, conf: Conf) -> String {
    let split_line = shell_words::split(&line).unwrap();
    let matches = player_cli().get_matches_from_safe(split_line);
    if matches.is_ok() {
        if let (c, Some(a)) = matches.unwrap().subcommand() {

            let wallet_path = match a.value_of("wallet-path") {
                Some(p) => PathBuf::from(p),
                None => current_dir().unwrap(),
            };

            let mut seed_path = wallet_path.clone();
            seed_path.push(SEED_NAME);
            let seed = match File::open(&seed_path) {
                Ok(reader) => match c {
                    "init" => return format!("cannot init, seed already exists"),
                    _ => serde_json::from_reader(reader).unwrap(),
                }
                Err(_) => match c {
                    "init" => {
                        let mnemonic = match a.value_of("seed-phrase") {
                            Some(phrase) => Some(Secret::new(phrase.to_owned())),
                            None => None,
                        };
                        let new_seed = SavedSeed::new(Secret::new(a.value_of("passphrase").unwrap().to_owned()), mnemonic);
                        match File::create(&seed_path) {
                            Ok(mut writer) => writer.write_all(serde_json::to_string(&new_seed).unwrap().as_bytes()).unwrap(),
                            Err(e) => return format!("{:?}", e),
                        };
                        return format!("wallet initialized")
                    }
                    _ => return format!("no seed. initialize wallet first"),
                }
            };

            let mut db_path = wallet_path.clone();
            db_path.push(DB_NAME);
            let db = DB::new(&db_path).unwrap();
            let _r = db.create_tables().unwrap();

            let electrum_client = match Client::new(&conf.electrum_url) {
                Ok(c) => c,
                Err(e) => return format!("{:?}", e)
            };
            let name_client = PlayerNameClient::new(&conf.name_url);
            let arbiter_client = ArbiterClient::new(&conf.arbiter_url);

            let wallet = PlayerWallet::new(seed, db, NETWORK, electrum_client, name_client, arbiter_client);
            match c {
                "balance" => format!("{}", wallet.balance()),
                "deposit" => format!("{}", wallet.deposit()),
                "player" => player_subcommand(a.subcommand(), &wallet),
                "contract" => contract_subcommand(a.subcommand(), &wallet),
                "payout" => payout_subcommand(a.subcommand(), &wallet),
                _ => format!("command '{}' is not implemented", c),
            }
        } else { 
            format!("invalid command") 
        }
    } else {
        let err = matches.err().unwrap();
        format!("{}", err)
    }
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
                    .required(true))
                .arg(Arg::with_name("passphrase")
                    .long("passphrase")
                    .index(2)
                    .required(true)
                    .takes_value(true)
                    .help("wallet passphrase")),
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
            SubCommand::with_name("post").about("post contract info to public")
                .arg(Arg::with_name("name")
                    .index(1)
                    .help("player to post info for")
                    .required(true))
                .arg(Arg::with_name("amount")
                    .index(2)
                    .help("total utxo amount to post")
                    .required(true))
                .arg(Arg::with_name("passphrase")
                    .long("passphrase")
                    .index(3)
                    .required(true)
                    .takes_value(true)
                    .help("wallet passphrase")),
        ])
}

pub fn player_subcommand(subcommand: (&str, Option<&ArgMatches>), wallet: &PlayerWallet) -> String {
    if let (c, Some(a)) = subcommand {
        match c {
            "register" => match wallet.register(PlayerName(a.value_of("name").unwrap().to_string()), Secret::new(a.value_of("passphrase").unwrap().to_owned())) {
                Ok(()) => format!("registered player"),
                Err(e) => format!("{}", e),
            }
            "add" => match wallet.add(PlayerName(a.value_of("name").unwrap().to_string())) {
                Ok(()) => format!("added player"),
                Err(e) => format!("{}", e),
            }
            "remove" => match wallet.remove(PlayerName(a.value_of("name").unwrap().to_string())) {
                Ok(()) => format!("removed player"),
                Err(e) => format!("{}", e),
            }
            "list" => if a.is_present("json-output") {
                    serde_json::to_string(&PlayerUI::list(wallet)).unwrap()
                } else {
                    PlayerUI::list(wallet).iter().fold(String::default(), |acc, p| acc + &format!("{}\n", p.name.0) )
                }
            "mine" => if a.is_present("json-output") {
                    serde_json::to_string(&wallet.mine()).unwrap()
                } else {
                    wallet.mine().iter().fold(String::default(), |acc, p| acc + &format!("{}", p.0) )
                },
            "post" => match wallet.post(PlayerName(a.value_of("name").unwrap().to_string()), Amount::from_sat(a.value_of("amount").unwrap().parse::<u64>().unwrap()), Secret::new(a.value_of("passphrase").unwrap().to_owned())) {
                Ok(_) => format!("posted contract info"),
                Err(e) => format!("{}", e),
            }
            _ => format!("command '{}' is not implemented", c),
        }
    }
    else { format!("invalid command") }
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
                .arg(Arg::with_name("passphrase")
                    .long("passphrase")
                    .index(2)
                    .required(true)
                    .takes_value(true)
                    .help("wallet passphrase"))
                .arg(Arg::with_name("sign-funding-tx")
                    .long("sign-funding-tx")
                    .required(false)
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

pub fn contract_subcommand(subcommand: (&str, Option<&ArgMatches>), wallet: &PlayerWallet) -> String{
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
                Ok(()) => format!("contract created"),
                Err(e) => format!("{}", e),
            }
            "import" => match DocumentUI::<ContractRecord>::import(wallet, &a.value_of("contract-value").unwrap()) {
// TODO: print cxid of imported contract
                Ok(()) => format!("contract imported"),
                Err(e) => format!("{}", e),
            }
            "export" => match DocumentUI::<ContractRecord>::export(wallet, &a.value_of("cxid").unwrap()) {
                Some(hex) => format!("{}", hex),
                None => format!("no such contract"),
            }
            "details" => match DocumentUI::<ContractRecord>::get(wallet, a.value_of("cxid").unwrap()) {
                Some(cr) => {
                    format!("{:?}", cr)
                }
                None => format!("no such contract"),
            }
            "sign" => match DocumentUI::<ContractRecord>::sign(
                    wallet,
                    SignDocumentParams::SignContractParams {
                        cxid: a.value_of("cxid").unwrap().to_string(),
                        sign_funding_tx: a.value_of("sign-funding-tx").is_some(),
                    },
                    Secret::new(a.value_of("passphrase").unwrap().to_owned())) {
                Ok(()) => format!("contract created"),
                Err(e) => format!("{}", e),
            }
            "submit" => match DocumentUI::<ContractRecord>::submit(wallet, a.value_of("cxid").unwrap()) {
                Ok(()) => format!("submission accepted"),
                Err(e) => format!("{}", e),
            }
            "broadcast" => match DocumentUI::<ContractRecord>::broadcast(wallet, a.value_of("cxid").unwrap()) {
                Ok(()) => format!("funding tx broadcasted to network"),
                Err(e) => format!("{}", e),
            }
            "delete" => match DocumentUI::<ContractRecord>::delete(wallet, a.value_of("cxid").unwrap()) {
                Ok(()) => format!("contract deleted"),
                Err(e) => format!("{}", e),
            }
            "list" => if a.is_present("json-output") {
                    serde_json::to_string(&DocumentUI::<ContractRecord>::list(wallet)).unwrap()
                } else {
                    DocumentUI::<ContractRecord>::list(wallet).iter().fold(String::default(),|acc, c| acc + &format!("{:?}", c))
                }
            _ => {
                format!("command '{}' is not implemented", c)
            }
        }            
    }
    else { format!("invalid command") }
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
                .arg(Arg::with_name("passphrase")
                    .long("passphrase")
                    .index(2)
                    .required(true)
                    .takes_value(true)
                    .help("wallet passphrase"))
                .arg(Arg::with_name("script-sig")
                    .index(3)
                    .required(false)
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

pub fn payout_subcommand(subcommand: (&str, Option<&ArgMatches>), wallet: &PlayerWallet) -> String {
    if let (c, Some(a)) = subcommand {
        match c {
            "new" => match DocumentUI::<PayoutRecord>::new(
                wallet,
                NewDocumentParams::NewPayoutParams {
                    cxid: a.value_of("cxid").unwrap().to_string(),
                    name: PlayerName(a.value_of("player").unwrap().to_string()),
                    amount: Amount::from_sat(a.value_of("amount").unwrap().parse::<u64>().unwrap()),
                }) {
                Ok(()) => format!("payout created"),
                Err(e) => format!("{}", e),
            }
            "import" => match DocumentUI::<PayoutRecord>::import(wallet, a.value_of("payout-value").unwrap()) {
// TODO: print cxid of imported payout
                Ok(()) => format!("payout imported"),
                Err(e) => format!("{}", e),
            }
            "export" => match DocumentUI::<PayoutRecord>::export(wallet, a.value_of("cxid").unwrap()) {
                Some(hex) => format!("{}", hex),
                None => format!("no such payout"),
            }
            "details" => match DocumentUI::<PayoutRecord>::get(wallet, a.value_of("cxid").unwrap()) {
                Some(pr) => format!("{:?}", pr),
                None => format!("no such payout"),
            }
            "sign" => match DocumentUI::<PayoutRecord>::sign(
                    wallet,
                    SignDocumentParams::SignPayoutParams {
                        cxid: a.value_of("cxid").unwrap().to_string(),
                        script_sig: match a.value_of("script_sig") {
                            Some(sig_hex) => Some(Signature::from_compact(&hex::decode(sig_hex).unwrap()).unwrap()),
                            None => None,
                        },
                    },
                    Secret::new(a.value_of("passphrase").unwrap().to_owned())) {
                Ok(()) => format!("payout created"),
                Err(e) => format!("{}", e),
            }
            "submit" => match DocumentUI::<PayoutRecord>::submit(wallet, a.value_of("cxid").unwrap()) {
                Ok(()) => format!("submission accepted"),
                Err(e) => format!("{}", e),
            }
            "broadcast" => match DocumentUI::<PayoutRecord>::broadcast(wallet, a.value_of("cxid").unwrap()) {
                Ok(()) => format!("payout tx broadcasted to network"),
                Err(e) => format!("{}", e),
            }
            "delete" => match DocumentUI::<PayoutRecord>::delete(wallet, a.value_of("cxid").unwrap()) {
                Ok(()) => format!("payout deleted"),
                Err(e) => format!("{}", e),
            }
            "list" => DocumentUI::<PayoutRecord>::list(wallet).iter().fold(String::default(), |acc, p| acc + &format!("{:?}", p)),
            _ => format!("command '{}' is not implemented", c)
        }            
    }
    else { format!("invalid command") }
}

