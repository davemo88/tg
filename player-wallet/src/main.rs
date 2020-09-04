use std::{
    cmp::max,
    collections::HashMap,
    convert::TryFrom,
    env::current_dir,
    net::{
        AddrParseError, 
        SocketAddr
    },
    path::{
        PathBuf, 
        Path},
    str::FromStr,
    thread,
};
use fern;
use log::{debug, error, info, warn, LevelFilter};
use serde::{
    Serialize,
};
use bdk::api::{balance, deposit_addr, init_config, start, stop, update_config, withdraw};
use bdk::api;
use bdk::config::Config;
use bdk::error::Error;
use std::process::ChildStderr;
use bitcoin::{
    Address,
    Amount,
    Network,
    Transaction,
    util::bip32::ExtendedPubKey,
};
use bdk::{
    wallet::Wallet,
};
use clap::ArgMatches;
use rustyline::Editor;
use rustyline::error::ReadlineError;
use shell_words;
use tglib::{
    Result as TgResult,
    TgError,
    TgScriptSig,
    arbiter::{
        ArbiterId,
    },
    contract::{
        Contract,
        ContractBuilder,
        ContractSignature,
    },
    payout::{
        Payout,
    },
    player::{
        PlayerId,
    },
    script::TgScript,
    wallet::{
        Creation,
        Signing,
    }
};

mod ui;
mod db;

const DB_NAME: &'static str = "player-wallet.db";

pub struct PlayerWallet(Wallet);

impl PlayerWallet {
    pub fn player_id(&self) -> PlayerId {
        PlayerId(String::from("hi"))
    }
    
}

impl Creation for PlayerWallet {
    fn create_contract(&self,
        p1_id:         PlayerId,
        p2_id:         PlayerId,
        arbiter_id:    ArbiterId,
        amount:         Amount,
        payout_script:  TgScript,
        funding_tx:     Option<Transaction>,
    ) -> Contract {
        let mut contract = ContractBuilder::default();
        contract.p1_id(p1_id);
        contract.p2_id(p2_id);
        contract.arbiter_id(arbiter_id);
        contract.amount(amount);
        contract.payout_script(payout_script);

        if let Some(tx) = funding_tx {
           contract.funding_tx(tx);
        }
        else {
// TODO: generate one using the player ids, arbiter id, and amount from the contract
        }


        contract.build()
    }

    fn create_payout(&self, contract: &Contract, payout_tx: Transaction, payout_script_sig: TgScriptSig) -> Payout {
        Payout::new(
            &contract,
            payout_tx,
            payout_script_sig,
        )
    }
    

}

impl Signing for PlayerWallet {
    fn sign_contract(&self, _contract: Contract) -> TgResult<Contract> {
        Err(TgError("cannot sign contract"))
    }

    fn sign_payout(&self, _payout: Payout) -> TgResult<Payout>{
        Err(TgError("cannot sign payout request"))
    }
}

fn player_subcommand(subcommand: (&str, Option<&ArgMatches>), db: &db::DB) -> TgResult<()> {
    if let (c, Some(a)) = subcommand {
        match c {
            "add" => {
                let player = db::PlayerRecord {
                    id:         PlayerId(a.args["id"].vals[0].clone().into_string().unwrap()),
                    name:       a.args["name"].vals[0].clone().into_string().unwrap(),
                };
                match db.insert_player(player.clone()) {
                    Ok(()) => println!("added player {} named {}", player.id.0, player.name),
                    Err(e) => println!("{:?}", e),
                }
            }
            "list" => {
                let players = db.all_players().unwrap();
                if (players.len() == 0) {
                    println!("no players");
                }
                else {
                    for p in players {
                        println!("id: {}, name: {}", p.id.0, p.name);
                    }
                }
            }
            "remove" => {
                let player_id = PlayerId(a.args["id"].vals[0].clone().into_string().unwrap());
                match db.delete_player(player_id.clone()) {
                    Ok(num_deleted) => match num_deleted {
                        0 => println!("no player with that id"),
                        1 => println!("deleted player {}", player_id.0),
                        _ => (),
                    }
                    Err(e) => println!("{:?}", e),
                }
            }
            _ => {
                println!("command '{}' is not implemented", c);
            }
        }
 
    }
    Ok(())
}

fn contract_subcommand(subcommand: (&str, Option<&ArgMatches>), db: &db::DB) -> TgResult<()> {
    if let (c, Some(a)) = subcommand {
        match c {
            "new" => {
                println!("{}", c);
            }
            "list" => {
                println!("{}", c);
            }
            "details" => {
                println!("{}", c);
            }
            "sign" => {
                println!("{}", c);
            }
            _ => {
                println!("command '{}' is not implemented", c);
            }
        }            
    }
    Ok(())
}

fn payout_subcommand(subcommand: (&str, Option<&ArgMatches>), db: &db::DB) -> TgResult<()> {
    if let (c, Some(a)) = subcommand {
        match c {
            "new" => {
                println!("{}", c);
            }
            "list" => {
                println!("{}", c);
            }
            "details" => {
                println!("{}", c);
            }
            "sign" => {
                println!("{}", c);
            }
            _ => {
                println!("command '{}' is not implemented", c);
            }
        }            
    }
    Ok(())
}

fn main() -> Result<(), Error> {

    let cli = ui::cli().get_matches();
    let log_level = cli.value_of("logging").unwrap_or("info");

    let connections = cli.value_of("connections").map(|c| c.parse::<usize>().unwrap()).unwrap_or(5);
    let directory = cli.value_of("directory").unwrap_or(".");
    let discovery = cli.value_of("discovery").map(|d| d == "on").unwrap_or(true);
    let network = cli.value_of("network").unwrap_or("testnet");
    let password = cli.value_of("password").expect("password is required");
    let peers = cli.values_of("peers").map(|a| a.collect::<Vec<&str>>()).unwrap_or(Vec::new());

    let work_dir: PathBuf = PathBuf::from(directory);
    let mut log_file = work_dir.clone();
    log_file.push(network);
    log_file.push("wallet.log");
    let log_file = log_file.as_path();
    let log_level = LevelFilter::from_str(log_level).unwrap();

    setup_logger(log_file, log_level);

    let mut history_file = work_dir.clone();
    history_file.push(network);
    history_file.push("history.txt");
    let history_file = history_file.as_path();
//    info!("history file: {:?}", history_file);

    let network = network.parse::<Network>().unwrap();

    println!("logging level: {}", log_level);
    println!("working directory: {:?}", work_dir);
    println!("discovery: {:?}", discovery);
    println!("network: {}", network);
    println!("peers: {:?}", peers);

    let init_result = api::init_config(work_dir.clone(), network, password, None);

    match init_result {
        Ok(Some(init_result)) => {
            println!("created new wallet, seed words: {}", init_result.mnemonic_words);
            println!("first deposit address: {}", init_result.deposit_address);
        }
        Ok(None) => {
            println!("wallet exists");
        }
        Err(e) => {
            println!("config error: {:?}", e);
        }
    };

    let peers = peers.into_iter()
        .map(|p| SocketAddr::from_str(p))
        .collect::<Result<Vec<SocketAddr>, AddrParseError>>()?;

    let connections = max(peers.len(), connections);

    println!("peer connections: {}", connections);

    let config = api::update_config(work_dir.clone(), network, peers, connections, discovery).unwrap();
    debug!("config: {:?}", config);

    let mut rl = Editor::<()>::new();

    if rl.load_history(history_file).is_err() {
        println!("No previous history.");
    }

    let p2p_thread = thread::spawn(move || {
        println!("starting p2p thread");
        api::start(work_dir.clone(), network, false);
    });

    let mut db_path = current_dir().unwrap();
    db_path.push(DB_NAME);
    let db = db::DB::new(&db_path).unwrap();
    db.create_tables();

    loop {
        let readline = rl.readline(">> ");
        match readline {
            Ok(line) => {
                let split_line = shell_words::split(&line).unwrap();
                let matches = ui::repl().get_matches_from_safe(split_line);
                if matches.is_ok() {
                    if let (c, Some(a)) = matches.unwrap().subcommand() {
                        debug!("command: {}, args: {:?}", c, a);
                        rl.add_history_entry(line.as_str());
                        match c {
                            "stop" => {
                                break;
                            }
                            "player" => {
                                player_subcommand(a.subcommand(), &db);
                            }
                            "balance" => {
                                let balance_amt = api::balance().unwrap();
                                println!("balance: {}, confirmed: {}", balance_amt.balance, balance_amt.confirmed);
                            }
                            "deposit" => {
                               let deposit_addr = api::deposit_addr();
                                println!("deposit address: {}", deposit_addr);
                            }
                            "withdraw" => {
                                // passphrase: String, address: Address, fee_per_vbyte: u64, amount: Option<u64>
                                let password = a.value_of("password").unwrap().to_string();
                                let address = Address::from_str(a.value_of("address").unwrap()).unwrap();
                                let fee = a.value_of("fee").unwrap().parse::<u64>().unwrap();
                                let amount = Some(a.value_of("amount").unwrap().parse::<u64>().unwrap());
                                let withdraw_tx = api::withdraw(password, address, fee, amount).unwrap();
                                println!("withdraw tx id: {}, fee: {}", withdraw_tx.txid, withdraw_tx.fee);
                            }
                            "contract" => {
                                contract_subcommand(a.subcommand(), &db);
                            }
                            "payout" => {
                                payout_subcommand(a.subcommand(), &db);
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
    api::stop();
    p2p_thread.join().unwrap();
    println!("stopped");

    Ok(())

}

fn setup_logger(file: &Path, level: LevelFilter) -> Result<(), fern::InitError> {
    fern::Dispatch::new()
        .format(|out, message, record| {
            out.finish(format_args!(
                "{}[{}][{}] {}",
                chrono::Local::now().format("[%Y-%m-%d][%H:%M:%S]"),
                record.target(),
                record.level(),
                message
            ))
        })
        .level(level)
        .chain(fern::log_file(file)?)
        .apply()?;
    Ok(())
}
