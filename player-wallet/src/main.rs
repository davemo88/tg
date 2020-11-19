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
        Path,
    },
    process::ChildStderr,
    str::FromStr,
    thread,
};
use log::debug;
use serde::{
    Serialize,
};
use bdk::{
    bitcoin::{
        Address,
        Amount,
        util::{
            psbt::PartiallySignedTransaction,
        }
    },
    blockchain::{
        noop_progress,
    },
    Error,
};
use bip39::Mnemonic;
use clap::ArgMatches;
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
        Creation,
        Signing,
    }
};

mod db;
mod mock;
mod ui;
mod wallet;
use mock::{
    Trezor,
    NETWORK,
    PLAYER_MNEMONIC,
};
use wallet::{
    PlayerWallet,
    SigningWallet,
};

const DB_NAME: &'static str = "dev-app.db";

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
                        n => panic!("{} removed, should be impossible", n),//this is impossible
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

    let work_dir: PathBuf = PathBuf::from("./");
    let mut history_file = work_dir.clone();
    history_file.push(&NETWORK.to_string());
    history_file.push("history.txt");
    let history_file = history_file.as_path();

    let signing_wallet = Trezor::new(Mnemonic::parse(PLAYER_MNEMONIC).unwrap());

    let player_wallet = PlayerWallet::new(signing_wallet.fingerprint(), signing_wallet.xpubkey(), NETWORK);

    player_wallet.wallet.sync(noop_progress(), None)?;

    let mut rl = Editor::<()>::new();

    if rl.load_history(history_file).is_err() {
        println!("No previous history.");
    }

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
                            "quit" => {
                                break;
                            }
                            "player" => {
                                player_subcommand(a.subcommand(), &db);
                            }
                            "balance" => {
                                println!("{}", player_wallet.balance());
                            }
                            "deposit" => {
//                               let deposit_addr = api::deposit_addr();
                                println!("deposit address");//: {}", deposit_addr);
                            }
                            "withdraw" => {
                                // passphrase: String, address: Address, fee_per_vbyte: u64, amount: Option<u64>
                                let password = a.value_of("password").unwrap().to_string();
                                let address = Address::from_str(a.value_of("address").unwrap()).unwrap();
                                let fee = a.value_of("fee").unwrap().parse::<u64>().unwrap();
                                let amount = Some(a.value_of("amount").unwrap().parse::<u64>().unwrap());
//                                let withdraw_tx = api::withdraw(password, address, fee, amount).unwrap();
                                println!("withdraw tx");// id: {}, fee: {}", withdraw_tx.txid, withdraw_tx.fee);
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
    println!("stopped");

    Ok(())

}

#[cfg(test)]
mod tests {

    use super::*;
    use mock::BITCOIN_RPC_URL;
    use bitcoincore_rpc::{Auth, Client as RpcClient, RpcApi, json::EstimateMode};

    const PLAYER_1_MNEMONIC: &'static str = "deny income tiger glove special recycle cup surface unusual sleep speed scene enroll finger protect dice powder unit";
    const PLAYER_2_MNEMONIC: &'static str = "carry tooth vague volcano refuse purity bike owner diary dignity toe body notable foil hedgehog mesh dream shock";

    const MIN_BALANCE: Amount = Amount::ONE_BTC;

    #[test]
    fn fund_players() {
        let p1_signing_wallet = Trezor::new(Mnemonic::parse(PLAYER_1_MNEMONIC).unwrap()); 
        let p1_wallet = PlayerWallet::new(p1_signing_wallet.fingerprint(), p1_signing_wallet.xpubkey(), NETWORK);
        let p1_addr = p1_wallet.new_address();

        let p2_signing_wallet = Trezor::new(Mnemonic::parse(PLAYER_2_MNEMONIC).unwrap()); 
        let p2_wallet = PlayerWallet::new(p2_signing_wallet.fingerprint(), p2_signing_wallet.xpubkey(), NETWORK);
        let p2_addr = p2_wallet.new_address();

        let rpc = RpcClient::new(BITCOIN_RPC_URL.to_string(), Auth::UserPass("admin".to_string(), "passw".to_string())).unwrap();
//        println!("{:?}", rpc.get_blockchain_info());
        println!("bitcoincore balance before sending {:?}", rpc.get_balance(None, None).unwrap());
        let coinbase_addr = rpc.get_new_address(None, None).unwrap();
        let txid1 = rpc.send_to_address(&p1_addr, Amount::ONE_BTC, None, None, None, None, None, Some(EstimateMode::Conservative)).unwrap();
        let txid2 = rpc.send_to_address(&p2_addr, Amount::ONE_BTC, None, None, None, None, None,Some(EstimateMode::Conservative)).unwrap();
//        println!("funding txids- p1: {} p2: {}", txid1, txid2);
        let blockhashes = rpc.generate_to_address(150, &coinbase_addr).unwrap();
        println!("bitcoincore balance after sending {:?}", rpc.get_balance(None, None).unwrap());

        println!("p1 balance after sending {:?}", p1_wallet.balance());
        println!("p2 balance after sending {:?}", p2_wallet.balance());

        assert_eq!(p1_wallet.balance(), Amount::ONE_BTC);
        assert_eq!(p2_wallet.balance(), Amount::ONE_BTC);
    }

    #[test]
    fn create_contract() {

    }
}
