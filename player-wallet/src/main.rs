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
    PublicKey,
    Transaction,
    Script,
    blockdata::{
        script::Builder,
        opcodes::all as Opcodes,
    },
    util::bip32::ExtendedPubKey,
};
use bdk::{
    wallet::Wallet,
    api as bdk_api,
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

mod mock;

const ARBITER_ID: &'static str = "arbiter1somebogusid";
const DB_NAME: &'static str = "dev-app.db";

pub struct PlayerWallet(Wallet);

impl PlayerWallet {
    pub fn player_id(&self) -> PlayerId {
        PlayerId::from(self.0.master.master_public().clone())
    }

    fn create_funding_tx(&self, p2_id: PlayerId, amount: Amount) -> Transaction {

        let escrow_address = self.create_escrow_address(&p2_id).unwrap();
        
        let p1_withdraw_tx = bdk_api::withdraw(
            String::from(mock::PASSPHRASE),
            escrow_address,
            1,
            None,
        );

        Transaction {
            version: 1,
            lock_time: 0,
            input: Vec::new(),
            output: Vec::new(),
        }
    }

    fn create_escrow_address(&self, p2_id: &PlayerId) -> TgResult<Address> {

        let p1_pubkey = self.get_pubkey();
        let p2_pubkey = mock::PlayerPubkeyService::get_pubkey(p2_id);
        let arbiter_pubkey = mock::ArbiterPubkeyService::get_pubkey();

        let escrow_address = Address::p2wsh(
            &self.create_escrow_script(p1_pubkey, p2_pubkey, arbiter_pubkey),
            self.0.master_public().network
        );

        Ok(escrow_address)

    }

    fn get_pubkey(&self) -> PublicKey {
        PublicKey::from_str("lol shit").unwrap()
    }

    fn create_escrow_script(&self, p1_pubkey: PublicKey, p2_pubkey: PublicKey, arbiter_pubkey: PublicKey) -> Script {
// standard multisig transaction script
// https://en.bitcoin.it/wiki/BIP_0011
        let b = Builder::new();
        b.push_opcode(Opcodes::OP_PUSHBYTES_2);
        b.push_slice(&p1_pubkey.to_bytes());
        b.push_slice(&p2_pubkey.to_bytes());
        b.push_slice(&arbiter_pubkey.to_bytes());
        b.push_opcode(Opcodes::OP_PUSHBYTES_3);
        b.push_opcode(Opcodes::OP_CHECKMULTISIG);

        b.into_script()
    }

    fn create_payout_script(&self, p2_id: PlayerId, amount: Amount, funding_tx: Transaction) -> TgScript {
        TgScript::default()
    }
    
}

impl PlayerWallet {
    pub fn create_contract(&self,
        p2_id:          PlayerId,
        amount:         Amount,
    ) -> Contract {

        let funding_tx = self.create_funding_tx(p2_id.clone(), amount);

        Contract::new(
            self.player_id(),
            p2_id.clone(),
            ArbiterId(String::from(ARBITER_ID)),
            amount,
            funding_tx.clone(),
            self.create_payout_script(p2_id, amount, funding_tx),
        )

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
                        _ => (),//this is impossible
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
// TODO: this is gonna go pretty deep so want to get out of this file asap
                println!("new {:?}", a);
                let p2_id = PlayerId(a.args["other"].vals[0].clone().into_string().unwrap());
                let amount = a.args["other"].vals[0].clone();
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
                println!("new {:?}", a);
            }
            "list" => {
                println!("list");
            }
            "details" => {
                println!("details {:?}", a);
            }
            "sign" => {
                println!("sign {:?}", a);
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
