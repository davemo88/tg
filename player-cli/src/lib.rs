use std::{
    env::current_dir,
    fs::File,
    io::Write,
    path::PathBuf,
};
use serde::{Deserialize, Serialize};
use clap::{App, Arg, ArgMatches, SubCommand, AppSettings};
use serde_json;
use shell_words;
use tglib::{
    bdk::bitcoin::{
        consensus,
        Address,
        Amount,
        secp256k1::Signature,
        util::psbt::PartiallySignedTransaction,
    },
    hex,
    secrecy::Secret,
    arbiter::ArbiterService,
    contract::{
        Contract,
        ContractRecord,
    },
    payout::PayoutRecord,
    player::PlayerName,
    wallet::{
        EscrowWallet,
        SavedSeed,
    },
    mock::{
        NETWORK,
        SEED_NAME,
    },
};
use player_wallet::{
    ui::{
        DocumentUI,
        NewDocumentParams,
        PlayerUI,
        WalletUI,
        SignDocumentParams,
    },
    wallet::PlayerWallet,
};

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Status {
    Success,
    Error,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
struct JsonResponse<T: Serialize> {
    status: Status,
    data: Option<T>,
    message: Option<String>,
}

impl<T: Serialize> JsonResponse<T> {
    fn success(data: Option<T>) -> Self {
        JsonResponse {
            status: Status::Success,
            data,
            message: None,
        }
    }
    
    fn error(message: String, data: Option<T>) -> Self {
        JsonResponse {
            status: Status::Error,
            data,
            message: Some(message),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ContractSummary {
    pub cxid:           String,
    pub p1_name:        PlayerName,
    pub p2_name:        PlayerName,
    pub amount:         u64,
    pub desc:           String,
    pub p1_sig:         bool,
    pub p2_sig:         bool,
    pub arbiter_sig:    bool,
}

impl From<&ContractRecord> for ContractSummary {
    fn from(cr: &ContractRecord) -> Self {
        let contract = Contract::from_bytes(hex::decode(&cr.hex).unwrap()).unwrap();
        ContractSummary {
            cxid: cr.cxid.clone(),
            p1_name: cr.p1_name.clone(),
            p2_name: cr.p2_name.clone(),
            amount: contract.amount().unwrap().as_sat(),
            desc: cr.desc.clone(),
            p1_sig: contract.sigs.len() > 0,
            p2_sig: contract.sigs.len() > 1,
            arbiter_sig: contract.sigs.len() > 2,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PayoutSummary {
    pub cxid:           String,
    pub p1_sig:         bool,
    pub p2_sig:         bool,
    pub arbiter_sig:    bool,
    pub p1_amount:      u64,
    pub p2_amount:      u64,
    pub payout_token:   String,
}

impl PayoutSummary {
        fn get(player_wallet: &PlayerWallet, pr: &PayoutRecord) -> Option<PayoutSummary> {
            let psbt: PartiallySignedTransaction = consensus::deserialize(&hex::decode(&pr.psbt).unwrap()).unwrap();
            let cr = DocumentUI::<ContractRecord>::get(player_wallet, &pr.cxid)?; 
            let contract = Contract::from_bytes(hex::decode(cr.hex).unwrap()).unwrap();
            let my_script_pubkey = Address::p2wpkh(&player_wallet.get_escrow_pubkey(), NETWORK).unwrap().script_pubkey();
            let my_amount: u64 = psbt.global.unsigned_tx.output.iter().filter_map(|txout| if txout.script_pubkey == my_script_pubkey { Some(txout.value) } else { None}).sum();
            let contract_amount = contract.amount().ok()?.as_sat();
            let p1_amount = if contract.p1_pubkey == player_wallet.get_escrow_pubkey() {
                my_amount    
            } else {
                contract_amount - my_amount
            };
            let p2_amount = contract_amount - p1_amount;

            Some(PayoutSummary {
                cxid: pr.cxid.clone(),
                p1_sig: psbt.inputs.iter().any(|input| input.partial_sigs.get(&contract.p1_pubkey).is_some()),
                p2_sig: psbt.inputs.iter().any(|input| input.partial_sigs.get(&contract.p2_pubkey).is_some()),
                arbiter_sig: psbt.inputs.iter().any(|input| input.partial_sigs.get(&contract.arbiter_pubkey).is_some()),
                payout_token: pr.sig.clone(),
                p1_amount,
                p2_amount,
            })
        }
    }


fn player_cli<'a, 'b>() -> App<'a, 'b> {
    App::new("player-cli")
        .version(option_env!("CARGO_PKG_VERSION").unwrap_or("unknown"))
        .author(option_env!("CARGO_PKG_AUTHORS").unwrap_or(""))
        .about("player cli")
        .settings(&[AppSettings::NoBinaryName, AppSettings::SubcommandRequiredElseHelp,
            AppSettings::VersionlessSubcommands])
        .subcommand(SubCommand::with_name("init").about("initialize new wallet")
            .arg(Arg::with_name("password")
                .long("password")
                .required(true)
                .takes_value(true)
                .help("wallet password"))
            .arg(Arg::with_name("seed-phrase")
                .long("seed-phrase")
                .required(false)
                .takes_value(true)
                .help("BIP39 seed phrase")))
        .subcommand(SubCommand::with_name("balance").about("display balance (sats)"))
        .subcommand(SubCommand::with_name("deposit").about("display a deposit address"))
        .subcommand(SubCommand::with_name("fund").about("fund the wallet"))
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
        .arg(Arg::with_name("wallet-dir")
            .help("specify path to wallet")
            .required(false)
            .global(true)
            .takes_value(true)
            .long("wallet-dir"))
}

pub struct Conf {
    pub electrum_url: String,
    pub name_url: String,
    pub arbiter_url: String,
}

pub fn cli(line: String, conf: Conf) -> String {
    let split_line = match shell_words::split(&line) {
        Ok(line) => line,
        Err(e) => return format!("invalid command: {:?}", e),
    };
    let matches = player_cli().get_matches_from_safe(split_line);
    if matches.is_ok() {
        if let (c, Some(a)) = matches.unwrap().subcommand() {

            let wallet_dir = match a.value_of("wallet-dir") {
                Some(p) => PathBuf::from(p),
                None => current_dir().unwrap(),
            };

            let mut seed_path = wallet_dir.clone();
            seed_path.push(SEED_NAME);
            let _saved_seed: SavedSeed = match File::open(&seed_path) {
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
                        let new_seed = match SavedSeed::new(Secret::new(a.value_of("password").unwrap().to_owned()), mnemonic) {
                            Ok(seed) => seed,
                            Err(e) => return format!("{:?}", e),
                        };
                        match File::create(&seed_path) {
                            Ok(mut writer) => {
                                writer.write_all(serde_json::to_string(&new_seed).unwrap().as_bytes()).unwrap();
                            },
//                            Ok(mut writer) => writer.write_all(serde_json::to_string(&new_seed).unwrap().as_bytes()).unwrap(),
                            Err(e) => return format!("{:?}", e),
                        };
                        return format!("wallet initialized")
                    }
                    _ => return format!("no seed. initialize wallet first"),
                }
            };

            let wallet = PlayerWallet::new(wallet_dir, NETWORK, conf.electrum_url, conf.name_url, conf.arbiter_url);
            match c {
                "balance" => match wallet.balance() {
                    Ok(balance) => format!("{}", balance.as_sat()),
                    Err(e) => return format!("{:?}", e),
                }
                "deposit" => format!("{}", wallet.deposit()),
                "fund" => format!("{}", wallet.fund().unwrap()),
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
        .about("player subcommand")
        .settings(&[AppSettings::NoBinaryName, AppSettings::SubcommandRequiredElseHelp,
            AppSettings::VersionlessSubcommands])
        .subcommands(vec![
            SubCommand::with_name("register").about("register new player")
                .arg(Arg::with_name("name")
                    .index(1)
                    .help("new player name")
                    .required(true))
                .arg(Arg::with_name("password")
                    .long("password")
                    .required(true)
                    .takes_value(true)
                    .help("wallet password")),
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
                .arg(Arg::with_name("password")
                    .long("password")
                    .required(true)
                    .takes_value(true)
                    .help("wallet password")),
            SubCommand::with_name("posted").about("retrieve posted info for player")
                .arg(Arg::with_name("name")
                    .index(1)
                    .help("player name")
                    .required(true)),
        ])
}

pub fn player_subcommand(subcommand: (&str, Option<&ArgMatches>), wallet: &PlayerWallet) -> String {
    if let (c, Some(a)) = subcommand {
        match c {
            "register" => match wallet.register(PlayerName(a.value_of("name").unwrap().to_string()), Secret::new(a.value_of("password").unwrap().to_owned())) {
                Ok(()) => if a.is_present("json-output") {
// TODO: if not returning data, need to specify a dummy type
                    serde_json::to_string(&JsonResponse::<String>::success(None)).unwrap()
                } else {
                    format!("registered player")
                }
                Err(e) => if a.is_present("json-output") {
                    serde_json::to_string(&JsonResponse::<String>::error(e.to_string(), None)).unwrap()
                } else {
                    format!("{:?}", e)
                }
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
                serde_json::to_string(&JsonResponse::success(Some(PlayerUI::list(wallet)))).unwrap()
            } else {
                PlayerUI::list(wallet).iter().map(|pr| pr.name.clone().0 ).collect::<Vec<String>>().join("\n")
            }
            "mine" => if a.is_present("json-output") {
                serde_json::to_string(&JsonResponse::success(Some(wallet.mine()))).unwrap()
            } else {
                wallet.mine().iter().map(|p| p.clone().0).collect::<Vec<String>>().join("\n")
            },
            "post" => match wallet.post(PlayerName(a.value_of("name").unwrap().to_string()), Amount::from_sat(a.value_of("amount").unwrap().parse::<u64>().unwrap()), Secret::new(a.value_of("password").unwrap().to_owned())) {
                Ok(_) => if a.is_present("json-output") {
// TODO: if not returning data, need to specify a dummy type
                    serde_json::to_string(&JsonResponse::<String>::success(None)).unwrap()
                } else {
                    format!("posted contract info")
                }
                Err(e) => if a.is_present("json-output") {
                    serde_json::to_string(&JsonResponse::<String>::error(e.to_string(), None)).unwrap()
                } else {
                    format!("{:?}", e)
                }
            }
            "posted" => match wallet.arbiter_client().get_contract_info(PlayerName(a.value_of("name").unwrap().to_string())) {
                Ok(Some(info)) => if a.is_present("json-output") {
                    serde_json::to_string(&JsonResponse::success(Some(info.utxos.iter().map(|(_, sats, _)| sats).sum::<u64>()))).unwrap()
                } else {
                    format!("{} has posted {} worth of utxos", info.name.0, info.utxos.iter().map(|(_, sats, _)| sats).sum::<u64>())
                }
                Ok(None) => if a.is_present("json-output") {
                    serde_json::to_string(&JsonResponse::success(Some(0))).unwrap()
                } else {
                    format!("no info posted")
                }
                Err(e) => if a.is_present("json-output") {
                    serde_json::to_string(&JsonResponse::<String>::error(e.to_string(), None)).unwrap()
                } else {
                    format!("{:?}", e)
                }
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
        .about("contract subcommand")
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
            SubCommand::with_name("summary").about("show contract summary")
                .arg(Arg::with_name("cxid")
                    .index(1)
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
                .arg(Arg::with_name("password")
                    .long("password")
                    .required(true)
                    .takes_value(true)
                    .help("wallet password"))
                .arg(Arg::with_name("sign-funding-tx")
                    .long("sign-funding-tx")
                    .required(false)
                    .help("sign the funding tx as well as the contract")),
            SubCommand::with_name("send").about("send contract to other player")
                .arg(Arg::with_name("cxid")
                    .index(1)
                    .help("contract id")
                    .required(true)
                    .takes_value(true)),
            SubCommand::with_name("receive").about("receive a contract for one of your players")
                .arg(Arg::with_name("player-name")
                    .index(1)
                    .help("player to receive payout for")
                    .required(true)
                    .takes_value(true))
                .arg(Arg::with_name("password")
                    .long("password")
                    .required(true)
                    .takes_value(true)
                    .help("wallet password")),
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

pub fn contract_subcommand(subcommand: (&str, Option<&ArgMatches>), wallet: &PlayerWallet) -> String {
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
                Ok(contract_record) => if a.is_present("json-output") {
                    serde_json::to_string(&JsonResponse::success(Some(ContractSummary::from(&contract_record)))).unwrap()
                } else {
                    format!("contract {} created", contract_record.cxid)
                }
                Err(e) => if a.is_present("json-output") {
                    serde_json::to_string(&JsonResponse::<String>::error(e.to_string(), None)).unwrap()
                } else {
                    format!("{:?}", e)
                }
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
                Some(cr) => format!("{:?}\n{:?}", cr, Contract::from_bytes(hex::decode(&cr.hex).unwrap()).unwrap()),
                None => format!("no such contract"),
            }
            "summary" => match DocumentUI::<ContractRecord>::get(wallet, a.value_of("cxid").unwrap()) {
                Some(cr) => if a.is_present("json-output") {
                    serde_json::to_string(&JsonResponse::success(Some(ContractSummary::from(&cr)))).unwrap()
                } else {
                    format!("{:?}", ContractSummary::from(&cr))
                }
                None => if a.is_present("json-output") {
                    serde_json::to_string(&JsonResponse::<String>::success(None)).unwrap()
                } else {
                    format!("no such contract")
                }
            }
            "sign" => match DocumentUI::<ContractRecord>::sign(
                    wallet,
                    SignDocumentParams::SignContractParams {
                        cxid: a.value_of("cxid").unwrap().to_string(),
                        sign_funding_tx: a.is_present("sign-funding-tx"),
                    },
                    Secret::new(a.value_of("password").unwrap().to_owned())) {
                Ok(()) => if a.is_present("json-output") {
                    serde_json::to_string(&JsonResponse::<String>::success(None)).unwrap()
                } else {
                    format!("contract signed")
                }
                Err(e) => if a.is_present("json-output") {
                    serde_json::to_string(&JsonResponse::<String>::error(e.to_string(), None)).unwrap()
                } else {
                    format!("{:?}", e)
                }
            }
            "send" => match DocumentUI::<ContractRecord>::send(wallet, a.value_of("cxid").unwrap()) {
                Ok(()) => if a.is_present("json-output") {
                    serde_json::to_string(&JsonResponse::<String>::success(None)).unwrap()
                } else {
                    format!("contract sent")
                }
                Err(e) => if a.is_present("json-output") {
                    serde_json::to_string(&JsonResponse::<String>::error(e.to_string(), None)).unwrap()
                } else {
                    format!("{:?}", e)
                }
            }
            "receive" => match DocumentUI::<ContractRecord>::receive(
                wallet, 
                PlayerName(a.value_of("player-name").unwrap().to_string()),
                Secret::new(a.value_of("password").unwrap().to_owned())) {
                Ok(cxid) => if a.is_present("json-output") {
                    serde_json::to_string(&JsonResponse::<String>::success(cxid)).unwrap()
                } else {
                    if cxid.is_some() {
                        format!("contract received")
                    } else {
                        format!("no contract to receive")
                    }
                }
                Err(e) => if a.is_present("json-output") {
                    serde_json::to_string(&JsonResponse::<String>::error(e.to_string(), None)).unwrap()
                } else {
                    format!("{:?}", e)
                }
            }
            "submit" => match DocumentUI::<ContractRecord>::submit(wallet, a.value_of("cxid").unwrap()) {
                Ok(()) => if a.is_present("json-output") {
                    serde_json::to_string(&JsonResponse::<String>::success(None)).unwrap()
                } else {
                    format!("submission accepted")
                }
                Err(e) => if a.is_present("json-output") {
                    serde_json::to_string(&JsonResponse::<String>::error(e.to_string(), None)).unwrap()
                } else {
                    format!("{:?}", e)
                }
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
                let cs: Vec<ContractSummary> = DocumentUI::<ContractRecord>::list(wallet).iter().map(|cr| cr.into()).collect();
                serde_json::to_string(&JsonResponse::success(Some(cs))).unwrap()
            } else {
                DocumentUI::<ContractRecord>::list(wallet).iter().map(|cr| format!("{:?}", ContractSummary::from(cr))).collect::<Vec<String>>().join("\n")
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
        .about("payout subcommand")
        .settings(&[AppSettings::NoBinaryName, AppSettings::SubcommandRequiredElseHelp,
            AppSettings::VersionlessSubcommands])
        .subcommands(vec![
            SubCommand::with_name("new").about("create a new payout")
                .arg(Arg::with_name("cxid")
                    .index(1)
                    .help("which contract to pay out")
                    .required(true)
                    .takes_value(true))
                .arg(Arg::with_name("p1_amount")
                    .index(2)
                    .help("how much to pay player one")
                    .required(true)
                    .takes_value(true))
                .arg(Arg::with_name("p2_amount")
                    .index(3)
                    .help("how much to pay player two")
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
                    .help("contract id of payout")
                    .required(true)
                    .takes_value(true)),
            SubCommand::with_name("summary").about("show payout summary")
                .arg(Arg::with_name("cxid")
                    .index(1)
                    .help("contract id")
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
                    .required(false)
                    .help("payout script sig")
                    .takes_value(true))
                .arg(Arg::with_name("password")
                    .long("password")
                    .required(true)
                    .takes_value(true)
                    .help("wallet password")),
            SubCommand::with_name("send").about("send payout to other player")
                .arg(Arg::with_name("cxid")
                    .index(1)
                    .value_name("CXID")
                    .help("contract id for the payout")
                    .required(true)
                    .takes_value(true)),
            SubCommand::with_name("receive").about("receive a payout for one of your players")
                .arg(Arg::with_name("player-name")
                    .index(1)
                    .help("player to receive payout for")
                    .required(true)
                    .takes_value(true))
                .arg(Arg::with_name("password")
                    .long("password")
                    .required(true)
                    .takes_value(true)
                    .help("wallet password")),
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
                    p1_amount: Amount::from_sat(a.value_of("p1_amount").unwrap().parse::<u64>().unwrap()),
                    p2_amount: Amount::from_sat(a.value_of("p2_amount").unwrap().parse::<u64>().unwrap()),
                }) {
                Ok(payout_record) => if a.is_present("json-output") {
                    serde_json::to_string(&JsonResponse::success(Some(payout_record))).unwrap()
                } else {
                    format!("payout created for contract {}", payout_record.cxid)
                }
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
                Some(pr) => format!("{:?}\n{:?}", pr, consensus::deserialize::<PartiallySignedTransaction>(&hex::decode(pr.clone().psbt).unwrap()).unwrap()),
                None => format!("no such payout"),
            }
            "summary" => match DocumentUI::<PayoutRecord>::get(wallet, a.value_of("cxid").unwrap()) {
                Some(pr) => if a.is_present("json-output") {
                    serde_json::to_string(&JsonResponse::success(Some(PayoutSummary::get(wallet, &pr)))).unwrap()
                } else {
                    format!("{:?}", PayoutSummary::get(wallet, &pr))
                }
                None => if a.is_present("json-output") {
                    serde_json::to_string(&JsonResponse::<String>::success(None)).unwrap()
                } else {
                    format!("no such payout")
                }
            }
            "sign" => match DocumentUI::<PayoutRecord>::sign(
                    wallet,
                    SignDocumentParams::SignPayoutParams {
                        cxid: a.value_of("cxid").unwrap().to_string(),
                        script_sig: match a.value_of("script_sig") {
                            Some(sig_hex) => Some(Signature::from_der(&hex::decode(sig_hex).unwrap()).unwrap()),
                            None => None,
                        },
                    },
                    Secret::new(a.value_of("password").unwrap().to_owned())) {
                Ok(()) => format!("payout signed"),
                Err(e) => format!("{}", e),
            }
            "send" => match DocumentUI::<PayoutRecord>::send(wallet, a.value_of("cxid").unwrap()) {
                Ok(()) => format!("payout sent"),
                Err(e) => format!("{}", e),
            }
            "receive" => match DocumentUI::<PayoutRecord>::receive(
                wallet, 
                PlayerName(a.value_of("player-name").unwrap().to_string()),
                Secret::new(a.value_of("password").unwrap().to_owned())) {
                Ok(cxid) => if a.is_present("json-output") {
                    serde_json::to_string(&JsonResponse::<String>::success(cxid)).unwrap()
                } else {
                    if cxid.is_some() {
                        format!("payout received")
                    } else {
                        format!("no payout to receive")
                    }
                }
                Err(e) => if a.is_present("json-output") {
                    serde_json::to_string(&JsonResponse::<String>::error(e.to_string(), None)).unwrap()
                } else {
                    format!("{:?}", e)
                }
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
            "list" => if a.is_present("json-output") {
                serde_json::to_string(&JsonResponse::success(Some(DocumentUI::<PayoutRecord>::list(wallet)))).unwrap()
            } else {
                DocumentUI::<PayoutRecord>::list(wallet).iter().map(|pr| format!("{:?}", pr)).collect::<Vec<String>>().join("\n")
            }
            _ => format!("command '{}' is not implemented", c)
        }            
    }
    else { format!("invalid command") }
}

