use std::{
    env::current_dir,
    fs::File,
    io::Write,
    path::PathBuf,
    str::FromStr,
};
use serde::{Deserialize, Serialize};
use clap::{App, Arg, ArgMatches, SubCommand, AppSettings};
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
    contract::Contract,
    player::PlayerName,
    wallet::{
        TX_FEE,
        SavedSeed,
    },
    mock::{
        NETWORK,
        SEED_NAME,
    },
    JsonResponse,
};
use libexchange::{
    ExchangeService,
    TokenContractRecord,
    PayoutRecord,
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
    pub txid:           String,
    pub p1_token_desc:  String,
    pub p2_token_desc:  String,
}

impl From<&TokenContractRecord> for ContractSummary {
    fn from(tcr: &TokenContractRecord) -> Self {
        let cr = &tcr.contract_record;
        let contract = Contract::from_bytes(hex::decode(&cr.hex).unwrap()).unwrap();
        ContractSummary {
            cxid: cr.cxid.clone(),
            p1_name: cr.p1_name.clone(),
            p2_name: cr.p2_name.clone(),
            amount: contract.amount().unwrap().as_sat(),
            desc: cr.desc.clone(),
            p1_sig: !contract.sigs.is_empty(),
            p2_sig: contract.sigs.len() > 1,
            arbiter_sig: contract.sigs.len() > 2,
            txid: contract.funding_tx.extract_tx().txid().to_string(),
            p1_token_desc: tcr.p1_token.desc.clone(),
            p2_token_desc: tcr.p2_token.desc.clone(),
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
    pub script_sig:     String,
    pub txid:           String,
}

impl PayoutSummary {
        fn get(player_wallet: &PlayerWallet, pr: &PayoutRecord) -> Option<PayoutSummary> {
            let psbt: PartiallySignedTransaction = consensus::deserialize(&hex::decode(&pr.psbt).unwrap()).unwrap();
            let txid = psbt.clone().extract_tx().txid().to_string();
            let tcr = DocumentUI::<TokenContractRecord>::get(player_wallet, &pr.cxid)?; 
            let contract = Contract::from_bytes(hex::decode(tcr.contract_record.hex).unwrap()).unwrap();
// TODO: this is a bug since adding the payout addresses to the TCR
// it will always show the balance as going to the other player since
// we aren't paying out to the escrow pubkey any more
//            let my_script_pubkey = Address::p2wpkh(&player_wallet.get_escrow_pubkey(), NETWORK).unwrap().script_pubkey();
            let p1_script_pubkey = Address::from_str(&tcr.p1_token.address).unwrap().script_pubkey();
            let p1_amount = Amount::from_sat(psbt.global.unsigned_tx.output.iter().filter_map(|txout| if txout.script_pubkey == p1_script_pubkey { Some(txout.value) } else { None}).sum()).as_sat();
            let contract_amount = contract.amount().ok()?;
//            let p1_amount = if contract.p1_pubkey == player_wallet.get_escrow_pubkey() {
//                my_amount.as_sat()
//            } else {
//                contract_amount.as_sat() - my_amount.as_sat() - TX_FEE
//            };
            let p2_amount = contract_amount.as_sat() - p1_amount - TX_FEE;

            Some(PayoutSummary {
                cxid: pr.cxid.clone(),
                p1_sig: psbt.inputs.iter().any(|input| input.partial_sigs.get(&contract.p1_pubkey).is_some()),
                p2_sig: psbt.inputs.iter().any(|input| input.partial_sigs.get(&contract.p2_pubkey).is_some()),
                arbiter_sig: psbt.inputs.iter().any(|input| input.partial_sigs.get(&contract.arbiter_pubkey).is_some()),
                script_sig: pr.sig.clone(),
                p1_amount,
                p2_amount,
                txid,
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
        .subcommand(SubCommand::with_name("get-tx").about("get tx from blockchain")
            .arg(Arg::with_name("txid")
                .index(1)
                .help("txid of tx to get")
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
    pub exchange_url: String,
}

pub fn cli(line: String, conf: Conf) -> String {
    let split_line = match shell_words::split(&line) {
        Ok(line) => line,
        Err(e) => return format!("invalid command: {:?}", e),
    };
    let matches = player_cli().get_matches_from_safe(split_line);
    if let Ok(matches) = matches {
        if let (c, Some(a)) = matches.subcommand() {

            let wallet_dir = match a.value_of("wallet-dir") {
                Some(p) => PathBuf::from(p),
                None => current_dir().unwrap(),
            };

            let mut seed_path = wallet_dir.clone();
            seed_path.push(SEED_NAME);
            let _saved_seed: SavedSeed = match File::open(&seed_path) {
                Ok(reader) => match c {
                    "init" => return "cannot init, seed already exists".to_string(),
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
                            Ok(mut writer) => writer.write_all(serde_json::to_string(&new_seed).unwrap().as_bytes()).unwrap(),
                            Err(e) => return format!("{:?}", e),
                        };
                        if a.is_present("json-output") {
                            return serde_json::to_string(&JsonResponse::<String>::success(None)).unwrap()
                        } else {
                            return "wallet initialized".to_string()
                        }
                    }
                    _ => {
                        let msg = "no seed. initialize wallet first".to_string();
                        if a.is_present("json-output") {
                            return serde_json::to_string(&JsonResponse::<String>::error(msg, None)).unwrap()
                        } else {
                            return msg
                        }
                    }
                }
            };

            let wallet = PlayerWallet::new(wallet_dir, NETWORK, conf.electrum_url, conf.name_url, conf.arbiter_url, conf.exchange_url);
            match c {
                "balance" => match wallet.balance() {
                    Ok(balance) => if a.is_present("json-output") {
                        serde_json::to_string(&JsonResponse::<u64>::success(Some(balance.as_sat()))).unwrap()
                    } else {
                        format!("{}", balance.as_sat())
                    }
                    Err(e) => if a.is_present("json-output") {
                        serde_json::to_string(&JsonResponse::<String>::error(e.to_string(), None)).unwrap()
                    } else {
                        format!("{:?}", e)
                    }
                }
                "deposit" => format!("{}", wallet.deposit()),
                "fund" => {
                    let funding_txid = wallet.fund().unwrap();
                    if a.is_present("json-output") {
                        serde_json::to_string(&JsonResponse::<String>::success(Some(funding_txid.to_string()))).unwrap()
                    } else {
                        format!("{}", funding_txid)
                    }
                }
                "get-tx" => match wallet.get_tx(a.value_of("txid").unwrap()) {
                    Ok(found_it) => if a.is_present("json-output") {
// TODO: if not     returning data, need to specify a dummy type
                        serde_json::to_string(&JsonResponse::<bool>::success(Some(found_it))).unwrap()
                    } else if found_it {
                        "found it".to_string()
                    } else {
                        "didn't find it".to_string()
                    }
                    Err(e) => if a.is_present("json-output") {
                        serde_json::to_string(&JsonResponse::<String>::error(e.to_string(), None)).unwrap()
                    } else {
                        format!("{:?}", e)
                    }
                }
                "player" => player_subcommand(a.subcommand(), &wallet),
                "contract" => contract_subcommand(a.subcommand(), &wallet),
                "payout" => payout_subcommand(a.subcommand(), &wallet),
                _ => format!("command '{}' is not implemented", c),
            }
        } else { 
            "invalid command".to_string()
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
                    "registered player".to_string()
                }
                Err(e) => if a.is_present("json-output") {
                    serde_json::to_string(&JsonResponse::<String>::error(e.to_string(), None)).unwrap()
                } else {
                    format!("{:?}", e)
                }
            }
            "add" => match wallet.add(PlayerName(a.value_of("name").unwrap().to_string())) {
                Ok(()) => "added player".to_string(),
                Err(e) => format!("{}", e),
            }
            "remove" => match wallet.remove(PlayerName(a.value_of("name").unwrap().to_string())) {
                Ok(()) => "removed player".to_string(),
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
                    "posted contract info".to_string()
                }
                Err(e) => if a.is_present("json-output") {
                    serde_json::to_string(&JsonResponse::<String>::error(e.to_string(), None)).unwrap()
                } else {
                    format!("{:?}", e)
                }
            }
            "posted" => match wallet.exchange_client().get_contract_info(PlayerName(a.value_of("name").unwrap().to_string())) {
                Ok(Some(info)) => if a.is_present("json-output") {
                    serde_json::to_string(&JsonResponse::success(Some(info.utxos.iter().map(|(_, sats, _)| sats).sum::<u64>()))).unwrap()
                } else {
                    format!("{} has posted {} worth of utxos", info.name.0, info.utxos.iter().map(|(_, sats, _)| sats).sum::<u64>())
                }
                Ok(None) => if a.is_present("json-output") {
                    serde_json::to_string(&JsonResponse::success(Some(0))).unwrap()
                } else {
                    "no info posted".to_string()
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
    else { "invalid command".to_string() }
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
                .arg(Arg::with_name("event")
                    .index(4)
                    .help("event in json format")
                    .required(true)
                    .takes_value(true))
                .arg(Arg::with_name("event-payouts")
                    .index(5)
                    .help("which player to pay for each event outcome. player order should coincide with outcome order in event")
                    .required(true)
                    .multiple(true)),
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
            "new" => match DocumentUI::<TokenContractRecord>::new(
                wallet,
                NewDocumentParams::NewContractParams {
                    p1_name: PlayerName(a.value_of("player-1").unwrap().to_string()),
                    p2_name: PlayerName(a.value_of("player-2").unwrap().to_string()),
                    amount: Amount::from_sat(a.value_of("amount").unwrap().parse::<u64>().unwrap()),
                    event: serde_json::from_str(a.value_of("event").unwrap()).unwrap(),
                    event_payouts: a.values_of("event-payouts").unwrap().collect::<Vec<&str>>().iter().map(|player| {
                            PlayerName(player.to_string()) 
                        }).collect(),
                }) {
                Ok(tcr) => if a.is_present("json-output") {
                    serde_json::to_string(&JsonResponse::success(Some(ContractSummary::from(&tcr)))).unwrap()
                } else {
                    format!("contract {} created", tcr.contract_record.cxid)
                }
                Err(e) => if a.is_present("json-output") {
                    serde_json::to_string(&JsonResponse::<String>::error(e.to_string(), None)).unwrap()
                } else {
                    format!("{:?}", e)
                }
            }
            "import" => match DocumentUI::<TokenContractRecord>::import(wallet, &a.value_of("contract-value").unwrap()) {
// TODO: print cxid of imported contract
                Ok(()) => "contract imported".to_string(),
                Err(e) => format!("{}", e),
            }
            "export" => match DocumentUI::<TokenContractRecord>::export(wallet, &a.value_of("cxid").unwrap()) {
                Some(hex) => hex,
                None => "no such contract".to_string(),
            }
            "details" => match DocumentUI::<TokenContractRecord>::get(wallet, a.value_of("cxid").unwrap()) {
                Some(tcr) => if a.is_present("json-output") {
                    serde_json::to_string(&JsonResponse::success(Some(Contract::from_bytes(hex::decode(&tcr.contract_record.hex).unwrap()).unwrap()))).unwrap()
                } else {
                    format!("{:?}", Contract::from_bytes(hex::decode(&tcr.contract_record.hex).unwrap()).unwrap())
                }
                None => if a.is_present("json-output") {
                    serde_json::to_string(&JsonResponse::<String>::success(None)).unwrap()
                } else {
                    "no such contract".to_string()
                }
            }
            "summary" => match DocumentUI::<TokenContractRecord>::get(wallet, a.value_of("cxid").unwrap()) {
                Some(cr) => if a.is_present("json-output") {
                    serde_json::to_string(&JsonResponse::success(Some(ContractSummary::from(&cr)))).unwrap()
                } else {
                    format!("{:?}", ContractSummary::from(&cr))
                }
                None => if a.is_present("json-output") {
                    serde_json::to_string(&JsonResponse::<String>::success(None)).unwrap()
                } else {
                    "no such contract".to_string()
                }
            }
            "sign" => match DocumentUI::<TokenContractRecord>::sign(
                    wallet,
                    SignDocumentParams::SignContractParams {
                        cxid: a.value_of("cxid").unwrap().to_string(),
                        sign_funding_tx: a.is_present("sign-funding-tx"),
                    },
                    Secret::new(a.value_of("password").unwrap().to_owned())) {
                Ok(()) => if a.is_present("json-output") {
                    serde_json::to_string(&JsonResponse::<String>::success(None)).unwrap()
                } else {
                    "contract signed".to_string()
                }
                Err(e) => if a.is_present("json-output") {
                    serde_json::to_string(&JsonResponse::<String>::error(e.to_string(), None)).unwrap()
                } else {
                    format!("{:?}", e)
                }
            }
            "send" => match DocumentUI::<TokenContractRecord>::send(wallet, a.value_of("cxid").unwrap()) {
                Ok(()) => if a.is_present("json-output") {
                    serde_json::to_string(&JsonResponse::<String>::success(None)).unwrap()
                } else {
                    "contract sent".to_string()
                }
                Err(e) => if a.is_present("json-output") {
                    serde_json::to_string(&JsonResponse::<String>::error(e.to_string(), None)).unwrap()
                } else {
                    format!("{:?}", e)
                }
            }
            "receive" => match DocumentUI::<TokenContractRecord>::receive(
                wallet, 
                PlayerName(a.value_of("player-name").unwrap().to_string()),
                Secret::new(a.value_of("password").unwrap().to_owned())) {
                Ok(cxid) => if a.is_present("json-output") {
                    serde_json::to_string(&JsonResponse::<String>::success(cxid)).unwrap()
                } else  if let Some(cxid) = cxid {
                    format!("contract {} received", cxid)
                } else {
                    "no contract to receive".to_string()
                }
                Err(e) => if a.is_present("json-output") {
                    serde_json::to_string(&JsonResponse::<String>::error(e.to_string(), None)).unwrap()
                } else {
                    format!("{:?}", e)
                }
            }
            "submit" => match DocumentUI::<TokenContractRecord>::submit(wallet, a.value_of("cxid").unwrap()) {
                Ok(()) => if a.is_present("json-output") {
                    serde_json::to_string(&JsonResponse::<String>::success(None)).unwrap()
                } else {
                    "submission accepted".to_string()
                }
                Err(e) => if a.is_present("json-output") {
                    serde_json::to_string(&JsonResponse::<String>::error(e.to_string(), None)).unwrap()
                } else {
                    format!("{:?}", e)
                }
            }
            "broadcast" => match DocumentUI::<TokenContractRecord>::broadcast(wallet, a.value_of("cxid").unwrap()) {
                Ok(()) => if a.is_present("json-output") {
                    serde_json::to_string(&JsonResponse::<String>::success(None)).unwrap()
                } else {
                    "funding tx broadcast to network".to_string()
                }
                Err(e) => if a.is_present("json-output") {
                    serde_json::to_string(&JsonResponse::<String>::error(e.to_string(), None)).unwrap()
                } else {
                    format!("{:?}", e)
                }
            }
            "delete" => match DocumentUI::<TokenContractRecord>::delete(wallet, a.value_of("cxid").unwrap()) {
                Ok(()) => if a.is_present("json-output") {
                    serde_json::to_string(&JsonResponse::<String>::success(None)).unwrap()
                } else {
                    "contract deleted".to_string()
                }
                Err(e) => if a.is_present("json-output") {
                    serde_json::to_string(&JsonResponse::<String>::error(e.to_string(), None)).unwrap()
                } else {
                    format!("{:?}", e)
                }
            }
            "list" => if a.is_present("json-output") {
                let cs: Vec<ContractSummary> = DocumentUI::<TokenContractRecord>::list(wallet).iter().map(|cr| cr.into()).collect();
                serde_json::to_string(&JsonResponse::success(Some(cs))).unwrap()
            } else {
                DocumentUI::<TokenContractRecord>::list(wallet).iter().map(|cr| format!("{:?}", ContractSummary::from(cr))).collect::<Vec<String>>().join("\n")
            }
            _ => {
                format!("command '{}' is not implemented", c)
            }
        }            
    }
    else { "invalid command".to_string() }
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
                    serde_json::to_string(&JsonResponse::success(Some(PayoutSummary::get(wallet, &payout_record)))).unwrap()
                } else {
                    format!("payout created for contract {}", payout_record.cxid)
                }
                Err(e) => format!("{}", e),
            }
            "import" => match DocumentUI::<PayoutRecord>::import(wallet, a.value_of("payout-value").unwrap()) {
// TODO: print cxid of imported payout
                Ok(()) => "payout imported".to_string(),
                Err(e) => format!("{}", e),
            }
            "export" => match DocumentUI::<PayoutRecord>::export(wallet, a.value_of("cxid").unwrap()) {
                Some(hex) => hex,
                None => "no such payout".to_string(),
            }
            "details" => match DocumentUI::<PayoutRecord>::get(wallet, a.value_of("cxid").unwrap()) {
                Some(pr) => if a.is_present("json-output") {
                    serde_json::to_string(&JsonResponse::success(Some(consensus::deserialize::<PartiallySignedTransaction>(&hex::decode(pr.psbt).unwrap()).unwrap()))).unwrap()
                } else {
                    format!("{:?}", consensus::deserialize::<PartiallySignedTransaction>(&hex::decode(pr.clone().psbt).unwrap()).unwrap())
                }
                None => if a.is_present("json-output") {
                    serde_json::to_string(&JsonResponse::<String>::success(None)).unwrap()
                } else {
                    "no such payout".to_string()
                }
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
                    "no such payout".to_string()
                }
            }
            "sign" => match DocumentUI::<PayoutRecord>::sign(
                    wallet,
                    SignDocumentParams::SignPayoutParams {
                        cxid: a.value_of("cxid").unwrap().to_string(),
                        script_sig: match a.value_of("script-sig") {
                            Some(sig_hex) => Some(Signature::from_der(&hex::decode(sig_hex).unwrap()).unwrap()),
                            None => None,
                        },
                    },
                    Secret::new(a.value_of("password").unwrap().to_owned())) {
                Ok(()) => if a.is_present("json-output") {
                    serde_json::to_string(&JsonResponse::<String>::success(None)).unwrap()
                } else {
                    "payout signed".to_string()
                }
                Err(e) => if a.is_present("json-output") {
                    serde_json::to_string(&JsonResponse::<String>::error(e.to_string(), None)).unwrap()
                } else {
                    format!("{:?}", e)
                }
            }
            "send" => match DocumentUI::<PayoutRecord>::send(wallet, a.value_of("cxid").unwrap()) {
                Ok(()) => if a.is_present("json-output") {
                    serde_json::to_string(&JsonResponse::<String>::success(None)).unwrap()
                } else {
                    "payout sent".to_string()
                }
                Err(e) => if a.is_present("json-output") {
                    serde_json::to_string(&JsonResponse::<String>::error(e.to_string(), None)).unwrap()
                } else {
                    format!("{:?}", e)
                }
            }
            "receive" => match DocumentUI::<PayoutRecord>::receive(
                wallet, 
                PlayerName(a.value_of("player-name").unwrap().to_string()),
                Secret::new(a.value_of("password").unwrap().to_owned())) {
                Ok(cxid) => if a.is_present("json-output") {
                    serde_json::to_string(&JsonResponse::<String>::success(cxid)).unwrap()
                } else  if let Some(cxid) = cxid {
                    format!("payout {} received", cxid)
                } else {
                    "no payout to receive".to_string()
                }
                Err(e) => if a.is_present("json-output") {
                    serde_json::to_string(&JsonResponse::<String>::error(e.to_string(), None)).unwrap()
                } else {
                    format!("{:?}", e)
                }
            }
            "submit" => match DocumentUI::<PayoutRecord>::submit(wallet, a.value_of("cxid").unwrap()) {
                Ok(()) => if a.is_present("json-output") {
                    serde_json::to_string(&JsonResponse::<String>::success(None)).unwrap()
                } else {
                    "submission accepted".to_string()
                }
                Err(e) => if a.is_present("json-output") {
                    serde_json::to_string(&JsonResponse::<String>::error(e.to_string(), None)).unwrap()
                } else {
                    format!("{:?}", e)
                }
            }
            "broadcast" => match DocumentUI::<PayoutRecord>::broadcast(wallet, a.value_of("cxid").unwrap()) {
                Ok(()) => if a.is_present("json-output") {
                    serde_json::to_string(&JsonResponse::<String>::success(None)).unwrap()
                } else {
                    "payout tx broadcast to network".to_string()
                }
                Err(e) => if a.is_present("json-output") {
                    serde_json::to_string(&JsonResponse::<String>::error(e.to_string(), None)).unwrap()
                } else {
                    format!("{:?}", e)
                }
            }
            "delete" => match DocumentUI::<PayoutRecord>::delete(wallet, a.value_of("cxid").unwrap()) {
                Ok(()) => if a.is_present("json-output") {
                    serde_json::to_string(&JsonResponse::<String>::success(None)).unwrap()
                } else {
                    "payout deleted".to_string()
                }
                Err(e) => if a.is_present("json-output") {
                    serde_json::to_string(&JsonResponse::<String>::error(e.to_string(), None)).unwrap()
                } else {
                    format!("{:?}", e)
                }
            }
            "list" => if a.is_present("json-output") {
                let ps: Vec<PayoutSummary> = DocumentUI::<PayoutRecord>::list(wallet).iter().filter_map(|pr| PayoutSummary::get(&wallet, pr)).collect();
                serde_json::to_string(&JsonResponse::success(Some(ps))).unwrap()
            } else {
                DocumentUI::<PayoutRecord>::list(wallet).iter().map(|pr| format!("{:?}", pr)).collect::<Vec<String>>().join("\n")
            }
            _ => format!("command '{}' is not implemented", c)
        }            
    }
    else { "invalid command".to_string() }
}

#[cfg(test)]
mod test {
    
    use super::{
        cli,
        Conf,
        ContractSummary,
    };
    use libexchange::Event;
    use tglib::JsonResponse;

    const DIR_1: &'static str = "/tmp/wallet1";
    const DIR_2: &'static str = "/tmp/wallet2";

    const PW: &'static str = "boguspw";

    const EVENT: &'static str = "{
  \"desc\": \"Rays at Blue Jays on 2021-07-04\",
  \"outcomes\": [
    {
      \"desc\": \"Blue Jays win\",
      \"token\": \"74cd75ca2d2bf9254e9b841db85984c2f8d24689717f1b2dc56d86443f318f22\"
    },
    {
      \"desc\": \"Rays win\",
      \"token\": \"d07953a7bccd9fc214119282edd4f9d4bb527e1a222f9bb2c30b74dec8065b46\"
    }
  ],
  \"oracle_pubkey\": \"025c571f77d693246e64f01ef740064a0b024a228813c94ae7e1e4ee73e991e0ba\"
}";

    fn conf() -> Conf {
        Conf {
            electrum_url: "tcp://localhost:60401".into(),
            name_url: "http://localhost:18420".into(),
            arbiter_url: "http://localhost:5000".into(),
            exchange_url: "http://localhost:5050".into(),
        }
    }
    
    fn init_funded_wallet(wallet_dir: &str) {
        std::fs::create_dir(wallet_dir).unwrap();
        cli(format!("init --wallet-dir {} --password {}", wallet_dir, PW), conf());
        let txid = cli(format!("fund --wallet-dir {}", wallet_dir), conf());
// wait until the funding tx is indexed by electrum
        assert!(poll_get_tx(wallet_dir, &txid));
    }

    fn poll_get_tx(wallet_dir: &str, txid: &str) -> bool {
        let mut waiting_time = std::time::Duration::from_millis(100);
        let mut retries = 0;
        let max_retries = 5;
        loop {
            std::thread::sleep(waiting_time);
            let response: JsonResponse<bool> = serde_json::from_str(&cli(format!("get-tx {} --wallet-dir {} --json-output", txid, wallet_dir), conf())).unwrap();
            match response.data {
                Some(true) => return true,
                _ => (),
            }
            if retries < max_retries {
                retries += 1;
                waiting_time *= 2;
            } else {
                return false 
            }
        }
    }

    fn remove_test_wallets() {
        std::fs::remove_dir_all(DIR_1).unwrap();
        std::fs::remove_dir_all(DIR_2).unwrap();
    }

    fn random_player() -> String {
        use rand::{
            distributions::Alphanumeric,
            thread_rng, 
            Rng,
        };
        let player: String = thread_rng()
            .sample_iter(&Alphanumeric)
            .take(10)
            .map(char::from)
            .collect();
        player
    }

    fn sign_token(token: &str) -> String {
        use tglib::bdk::bitcoin::secp256k1;
        let secp = secp256k1::Secp256k1::new();
        let key = tglib::bdk::bitcoin::PrivateKey::from_wif(tglib::mock::REFEREE_PRIVKEY).unwrap();
        let token_bytes = tglib::hex::decode(token).unwrap();
        let msg = secp256k1::Message::from_slice(&token_bytes).unwrap();
        let sig = secp.sign(&msg, &key.key);

        tglib::hex::encode(sig.serialize_der())
    }
    
    fn setup_live_contract(player1: &str, player2: &str) -> String {

        println!("init p1 wallet");
        init_funded_wallet(DIR_1);
        println!("init p2 wallet");
        init_funded_wallet(DIR_2);

        println!("register p1: {}", cli(format!("player register {} --wallet-dir {} --password {}", player1, DIR_1, PW), conf()));
        println!("register p2: {}", cli(format!("player register {} --wallet-dir {} --password {}", player2, DIR_2, PW), conf()));
        cli(format!("player add {} --wallet-dir {}", player2, DIR_1), conf());
        cli(format!("player add {} --wallet-dir {}", player1, DIR_2), conf());

        println!("p2 posts");
        cli(format!("player post {} 100000000 --wallet-dir {} --password {}", player2, DIR_2, PW), conf());

        println!("p1 creates contract");
        let response: JsonResponse<ContractSummary> = serde_json::from_str(&
            cli(format!("contract new {} {} 100000000 --wallet-dir {} '{}' {} {} --json-output", player1, player2, DIR_1, EVENT, player1, player2), conf())).unwrap();
//        println!("contract new response: {:?}", response);
        let contract_summary = response.data.unwrap();
        let cxid = contract_summary.cxid;
        println!("p1 signs");
        cli(format!("contract sign {} --sign-funding-tx --wallet-dir {} --password {}", cxid, DIR_1, PW), conf());
        println!("p1 sends to p2");
        cli(format!("contract send {} --wallet-dir {}", cxid, DIR_1), conf());

        println!("p2 receives");
        cli(format!("contract receive {} --wallet-dir {} --password {}", player2, DIR_2, PW), conf());
        println!("p2 signs");
        cli(format!("contract sign {} --sign-funding-tx --wallet-dir {} --password {}", cxid, DIR_2, PW), conf());
        println!("p2 submits to arbiter");
        cli(format!("contract submit {} --wallet-dir {}", cxid, DIR_2), conf());
        println!("p2 sends fully-signed contract back to p1");
        cli(format!("contract send {} --wallet-dir {}", cxid, DIR_2), conf());

        println!("p1 receives");
        cli(format!("contract receive {} --wallet-dir {} --password {}", player1, DIR_1, PW), conf());
        println!("p1 broadcasts funding tx");
        cli(format!("contract broadcast {} --wallet-dir {}", cxid, DIR_1), conf());

        let response: JsonResponse<tglib::contract::Contract> = serde_json::from_str(&
            cli(format!("contract details {} --wallet-dir {} --json-output", cxid, DIR_1), conf())).unwrap();
        let contract = response.data.unwrap();
        let funding_txid = contract.funding_tx.extract_tx().txid().to_string();

        assert_eq!(contract.sigs.len(), 3);
        assert!(poll_get_tx(DIR_1, &funding_txid));

        println!("contract live");
        cxid
    }
    
    #[test]
    fn test_contract_with_player_payout() {

        simple_logger::SimpleLogger::new()
            .with_level(tglib::log::LevelFilter::Debug)
            .with_module_level("hyper", tglib::log::LevelFilter::Warn)
            .with_module_level("reqwest", tglib::log::LevelFilter::Warn)
            .with_module_level("sled", tglib::log::LevelFilter::Warn)
            .with_module_level("bdk", tglib::log::LevelFilter::Warn)
            .init()
            .unwrap();

        let (p1, p2) = (random_player(), random_player());
        let cxid = setup_live_contract(&p1, &p2);

        println!("p1 creates payout");
        cli(format!("payout new {} --wallet-dir {} 50000000 50000000", cxid, DIR_1), conf());
//        println!("{}", cli(format!("payout summary {} --wallet-dir {}", cxid, DIR_1), conf()));
        println!("p1 signs");
        cli(format!("payout sign {} --wallet-dir {} --password {}", cxid, DIR_1, PW), conf());
//        println!("{}", cli(format!("payout summary {} --wallet-dir {}", cxid, DIR_1), conf()));
        println!("p1 sends");
        cli(format!("payout send {} --wallet-dir {}", cxid, DIR_1), conf());

        println!("p2 receives");
        cli(format!("payout receive {} --wallet-dir {} --password {}", p2, DIR_2, PW), conf());
        println!("p2 signs");
        cli(format!("payout sign {} --wallet-dir {} --password {}", cxid, DIR_2, PW), conf());
//        println!("{}", cli(format!("payout summary {} --wallet-dir {}", cxid, DIR_2), conf()));
        println!("p2 broadcasts payout tx");
        println!("{}", cli(format!("payout broadcast {} --wallet-dir {}", cxid, DIR_2), conf()));

        let response: JsonResponse<tglib::bdk::bitcoin::util::psbt::PartiallySignedTransaction> = serde_json::from_str(&
            cli(format!("payout details {} --wallet-dir {} --json-output", cxid, DIR_2), conf())).unwrap();
//        println!("payout details response: {:?}", response);
        let psbt = response.data.unwrap();
        let payout_txid = psbt.extract_tx().txid().to_string();

        assert!(poll_get_tx(DIR_1, &payout_txid));

        remove_test_wallets();
    }
    
    #[test]
    fn test_contract_with_arbiter_payout_p1() {
        let (p1, p2) = (random_player(), random_player());
        let cxid = setup_live_contract(&p1, &p2);
        let event: Event = serde_json::from_str(EVENT).unwrap();

        println!("p1 creates payout");
        cli(format!("payout new {} --wallet-dir {} 100000000 0", cxid, DIR_1), conf());
//        let response: JsonResponse<tglib::bdk::bitcoin::util::psbt::PartiallySignedTransaction> = serde_json::from_str(&
//            cli(format!("payout details {} --wallet-dir {} --json-output", cxid, DIR_1), conf())).unwrap();
//        let payout_psbt = response.data.unwrap();
        let payout_script_sig = sign_token(&event.outcomes[0].token);
        println!("p1 signs with oracle token");
        cli(format!("payout sign {} {} --wallet-dir {} --password {}", cxid, payout_script_sig, DIR_1, PW), conf());
//        println!("{}", cli(format!("payout summary {} --wallet-dir {}", cxid, DIR_1), conf()));
        println!("p1 submits payout to arbiter");
        cli(format!("payout submit {} --wallet-dir {}", cxid, DIR_1), conf());
//        println!("{}", cli(format!("payout summary {} --wallet-dir {}", cxid, DIR_1), conf()));
        println!("p1 broadcasts payout tx");
        cli(format!("payout broadcast {} --wallet-dir {}", cxid, DIR_1), conf());

        let response: JsonResponse<tglib::bdk::bitcoin::util::psbt::PartiallySignedTransaction> = serde_json::from_str(&
            cli(format!("payout details {} --wallet-dir {} --json-output", cxid, DIR_1), conf())).unwrap();
        let psbt = response.data.unwrap();
        let payout_txid = psbt.extract_tx().txid().to_string();

        assert!(poll_get_tx(DIR_1, &payout_txid));
        println!("payout tx live");

        remove_test_wallets()
    }
    
    #[test]
    fn test_contract_with_arbiter_payout_p2() {
        let (p1, p2) = (random_player(), random_player());
        let cxid = setup_live_contract(&p1, &p2);
        let event: Event = serde_json::from_str(EVENT).unwrap();

        println!("p2 creates payout");
        cli(format!("payout new {} --wallet-dir {} 0 100000000", cxid, DIR_2), conf());
//        let response: JsonResponse<tglib::bdk::bitcoin::util::psbt::PartiallySignedTransaction> = serde_json::from_str(&
//            cli(format!("payout details {} --wallet-dir {} --json-output", cxid, DIR_2), conf())).unwrap();
//        let payout_psbt = response.data.unwrap();
        let payout_script_sig = sign_token(&event.outcomes[1].token);
        println!("p2 signs with oracle token");
        cli(format!("payout sign {} {} --wallet-dir {} --password {}", cxid, payout_script_sig, DIR_2, PW), conf());
 //       println!("{}", cli(format!("payout summary {} --wallet-dir {}", cxid, DIR_2), conf()));
        println!("p2 submits payout to arbiter");
        cli(format!("payout submit {} --wallet-dir {}", cxid, DIR_2), conf());
//        println!("{}", cli(format!("payout summary {} --wallet-dir {}", cxid, DIR_2), conf()));
        println!("p2 broadcasts payout tx");
        cli(format!("payout broadcast {} --wallet-dir {}", cxid, DIR_2), conf());

        let response: JsonResponse<tglib::bdk::bitcoin::util::psbt::PartiallySignedTransaction> = serde_json::from_str(&
            cli(format!("payout details {} --wallet-dir {} --json-output", cxid, DIR_2), conf())).unwrap();
        let psbt = response.data.unwrap();
        let payout_txid = psbt.extract_tx().txid().to_string();

        assert!(poll_get_tx(DIR_2, &payout_txid));
        println!("payout tx live");

        remove_test_wallets();
    }
    
    #[test]
    fn test_invalid_payout_mismatched_tx_script_sig() {
        let invalid_payout_response = "JsonResponse(\"Adhoc(invalid payout request)\")";

        let (p1, p2) = (random_player(), random_player());
        let cxid = setup_live_contract(&p1, &p2);
        let event: Event = serde_json::from_str(EVENT).unwrap();

        println!("create payout for p1");
        cli(format!("payout new {} --wallet-dir {} 100000000 0", cxid, DIR_1), conf());
        let payout_script_sig = sign_token(&event.outcomes[1].token);
        println!("sign with p2 token");
        cli(format!("payout sign {} {} --wallet-dir {} --password {}", cxid, payout_script_sig, DIR_1, PW), conf());
 //       println!("{}", cli(format!("payout summary {} --wallet-dir {}", cxid, DIR_2), conf()));
        println!("submit payout to arbiter");
        let response = cli(format!("payout submit {} --wallet-dir {}", cxid, DIR_1), conf());
        assert_eq!(response, invalid_payout_response);

        println!("create payout for p2");
        cli(format!("payout new {} --wallet-dir {} 0 100000000", cxid, DIR_2), conf());
        let payout_script_sig = sign_token(&event.outcomes[0].token);
        println!("sign with p1 token");
        cli(format!("payout sign {} {} --wallet-dir {} --password {}", cxid, payout_script_sig, DIR_2, PW), conf());
 //       println!("{}", cli(format!("payout summary {} --wallet-dir {}", cxid, DIR_2), conf()));
        println!("submit payout to arbiter");
        let response = cli(format!("payout submit {} --wallet-dir {}", cxid, DIR_2), conf());
        assert_eq!(response, invalid_payout_response);
        remove_test_wallets();
    }
    
    #[test]
    fn test_invalid_payout_wrong_tx() {
        let invalid_payout_response = "JsonResponse(\"Adhoc(couldn\\\'t determine payout address)\")";

        let (p1, p2) = (random_player(), random_player());
        let cxid = setup_live_contract(&p1, &p2);
        let event: Event = serde_json::from_str(EVENT).unwrap();

        println!("create incorrect payout for p1");
        cli(format!("payout new {} --wallet-dir {} 50000000 50000000", cxid, DIR_1), conf());
        let payout_script_sig = sign_token(&event.outcomes[0].token);
        println!("sign with p1 token");
        cli(format!("payout sign {} {} --wallet-dir {} --password {}", cxid, payout_script_sig, DIR_1, PW), conf());
 //       println!("{}", cli(format!("payout summary {} --wallet-dir {}", cxid, DIR_2), conf()));
        println!("submit payout to arbiter");
        let response = cli(format!("payout submit {} --wallet-dir {}", cxid, DIR_1), conf());
        assert_eq!(response, invalid_payout_response);

        println!("create incorrect payout for p2");
        cli(format!("payout new {} --wallet-dir {} 50000000 50000000", cxid, DIR_2), conf());
        let payout_script_sig = sign_token(&event.outcomes[1].token);
        println!("sign with p2 token");
        cli(format!("payout sign {} {} --wallet-dir {} --password {}", cxid, payout_script_sig, DIR_2, PW), conf());
 //       println!("{}", cli(format!("payout summary {} --wallet-dir {}", cxid, DIR_2), conf()));
        println!("submit payout to arbiter");
        let response = cli(format!("payout submit {} --wallet-dir {}", cxid, DIR_2), conf());
        assert_eq!(response, invalid_payout_response);
        remove_test_wallets();
    }
}
