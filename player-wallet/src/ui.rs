use std::str::FromStr;
use bdk::bitcoin::{
    Address,
    Amount,
    Transaction,
    consensus,
    secp256k1::{
        Message,
    },
    util::{
        bip32::DerivationPath,
        psbt::PartiallySignedTransaction,
    },
};
use bip39::Mnemonic;
use clap::{App, Arg, ArgMatches, SubCommand, AppSettings};
use tglib::{
    Result as TgResult,
    contract::{
        Contract,
    },
    player::{
        PlayerId,
    },
    wallet::{
        SigningWallet,
    }
};
use crate::{
    mock::{
        ArbiterService,
        PlayerInfoService,
        Trezor,
        NETWORK,
        PLAYER_1_MNEMONIC,
        BITCOIN_DERIVATION_PATH,
        ESCROW_SUBACCOUNT,
        ESCROW_KIX,
    },
    db,
    wallet::PlayerWallet,
};

pub fn wallet_ui<'a, 'b>() -> App<'a, 'b> {
    App::new("wallet")
        .version(option_env!("CARGO_PKG_VERSION").unwrap_or("unknown"))
        .author(option_env!("CARGO_PKG_AUTHORS").unwrap_or(""))
        .about("wallet commands")
        .settings(&[AppSettings::NoBinaryName, AppSettings::SubcommandRequiredElseHelp,
            AppSettings::VersionlessSubcommands])
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

pub fn wallet_subcommand(subcommand: (&str, Option<&ArgMatches>)) -> TgResult<()> {

    let signing_wallet = Trezor::new(Mnemonic::parse(PLAYER_1_MNEMONIC).unwrap());
    let wallet = PlayerWallet::new(signing_wallet.fingerprint(), signing_wallet.xpubkey(), NETWORK);

    if let (c, Some(a)) = subcommand {
        match c {
            "balance" => {
                println!("{}", wallet.balance().as_sat());
            }
            "deposit" => {
                let deposit_addr = wallet.new_address();
                println!("deposit address: {}", deposit_addr);
            }
            "withdraw" => {
                println!("withdraw tx");// id: {}, fee: {}", withdraw_tx.txid, withdraw_tx.fee);
            }
            "player" => {
                player_subcommand(a.subcommand(), &wallet);
            }
            "contract" => {
                contract_subcommand(a.subcommand(), &wallet);
            }
            "payout" => {
                payout_subcommand(a.subcommand(), &wallet);
            }
            _ => {
                println!("command '{}' is not implemented", c);
            }
        }
    };
    Ok(())
}

pub fn player_ui<'a, 'b>() -> App<'a, 'b> {
    App::new("player")
        .version(option_env!("CARGO_PKG_VERSION").unwrap_or("unknown"))
        .author(option_env!("CARGO_PKG_AUTHORS").unwrap_or(""))
        .about("player commands")
        .settings(&[AppSettings::NoBinaryName, AppSettings::SubcommandRequiredElseHelp,
            AppSettings::VersionlessSubcommands])
        .subcommands(vec![
            SubCommand::with_name("add").about("add to known players")
                .arg(Arg::with_name("id")
                    .index(1)
                    .value_name("ID")
                    .help("player id")
                    .required(true))
                .arg(Arg::with_name("name")
                    .index(2)
                    .value_name("NAME")
                    .help("player name")
                    .required(true)),
//                    .multiple(true)),
            SubCommand::with_name("remove").about("remove from known players")
                .arg(Arg::with_name("id")
                    .index(1)
                    .value_name("ID")
                    .help("id of player to remove")
                    .required(true)),
            SubCommand::with_name("list").about("list known players"),
            SubCommand::with_name("id").about("shows local player id"),
        ])
}

pub fn player_subcommand(subcommand: (&str, Option<&ArgMatches>), wallet: &PlayerWallet) -> TgResult<()> {
    if let (c, Some(a)) = subcommand {
        match c {
            "add" => {
                let player = db::PlayerRecord {
                    id:         PlayerId(a.args["id"].vals[0].clone().into_string().unwrap()),
                    name:       a.args["name"].vals[0].clone().into_string().unwrap(),
                };
                match wallet.db.insert_player(player.clone()) {
                    Ok(()) => println!("added player {} named {}", player.id.0, player.name),
                    Err(e) => println!("{:?}", e),
                }
            }
            "list" => {
                let players = wallet.db.all_players().unwrap();
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
                match wallet.db.delete_player(player_id.clone()) {
                    Ok(num_deleted) => match num_deleted {
                        0 => println!("no player with that id"),
                        1 => println!("removed player {}", player_id.0),
                        n => panic!("{} removed, should be impossible", n),//this is impossible
                    }
                    Err(e) => println!("{:?}", e),
                }
            },
            "id" => {
                let signing_wallet = Trezor::new(Mnemonic::parse(PLAYER_1_MNEMONIC).unwrap());
                let wallet = PlayerWallet::new(signing_wallet.fingerprint(), signing_wallet.xpubkey(), NETWORK);
                println!("{}", wallet.player_id().0);
            }
            _ => {
                println!("command '{}' is not implemented", c);
            }
        }
    }
    Ok(())
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
                .arg(Arg::with_name("player-2")
                    .index(1)
                    .value_name("PLAYER2")
                    .help("player 2's id")
                    .required(true)
                    .takes_value(true))
                .arg(Arg::with_name("amount")
                    .index(2)
                    .value_name("AMOUNT")
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
                .arg(Arg::with_name("contract-hex")
                    .index(1)
                    .value_name("CONTRACT_HEX")
                    .help("hex-encoded contract")
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

pub fn contract_subcommand(subcommand: (&str, Option<&ArgMatches>), wallet: &PlayerWallet) -> TgResult<()> {
    if let (c, Some(a)) = subcommand {
        match c {
            "new" => {
                let p2_id = PlayerId(a.value_of("player-2").unwrap().to_string());
                let amount = Amount::from_sat(a.value_of("amount").unwrap().parse::<u64>().unwrap());
                let desc = a.value_of("desc").unwrap_or("");
                let p2_contract_info = PlayerInfoService::get_contract_info(&p2_id);
                let arbiter_pubkey = ArbiterService::get_escrow_pubkey();

                let signing_wallet = Trezor::new(Mnemonic::parse(PLAYER_1_MNEMONIC).unwrap());
                let player_wallet = PlayerWallet::new(signing_wallet.fingerprint(), signing_wallet.xpubkey(), NETWORK);

                let contract = player_wallet.create_contract(p2_contract_info, amount, arbiter_pubkey);
                let contract_record = db::ContractRecord {
                    cxid: hex::encode(contract.cxid()),
                    p1_id: player_wallet.player_id(),
                    p2_id,
                    hex: hex::encode(contract.to_bytes()),
                    desc: desc.to_string(),
                };

                match wallet.db.insert_contract(contract_record.clone()) {
                    Ok(()) => println!("created contract {}", contract_record.cxid),
                    Err(e) => println!("{:?}", e),
                }
            }
            "import" => {

            }
            "details" => {
                let contracts = wallet.db.all_contracts().unwrap();
                for c in contracts {
                    if c.cxid == a.value_of("cxid").unwrap() {
                        let contract = Contract::from_bytes(hex::decode(c.hex).unwrap());
                        println!("{:?}", contract);
                        break;
                    }
                }
            }
            "sign" => {
                let contracts = wallet.db.all_contracts().unwrap();
                for c in contracts {
                    if c.cxid == a.value_of("cxid").unwrap() {
                        let signing_wallet = Trezor::new(Mnemonic::parse(PLAYER_1_MNEMONIC).unwrap());
                        let sig = signing_wallet.sign_message(
                            Message::from_slice(&hex::decode(c.cxid.clone()).unwrap()).unwrap(),
                            DerivationPath::from_str(&format!("m/{}/{}/{}", BITCOIN_DERIVATION_PATH, ESCROW_SUBACCOUNT, ESCROW_KIX)).unwrap(),
                        ).unwrap();
                        let mut contract = Contract::from_bytes(hex::decode(c.hex.clone()).unwrap());
                        contract.sigs.push(sig);
                        wallet.db.add_signature(c.cxid, hex::encode(contract.to_bytes()));
                        assert_ne!(hex::encode(contract.to_bytes()), c.hex);
                        break;
                    }
                }
            }
            "delete" => {
                let r = wallet.db.delete_contract(a.value_of("cxid").unwrap().to_string());
                if r.is_ok() {
                    println!("deleted contract {}", a.value_of("cxid").unwrap());
                }
            }
            "list" => {
                let contracts = wallet.db.all_contracts().unwrap();
                if (contracts.len() == 0) {
                    println!("no players");
                }
                else {
                    for c in contracts {
                        println!("cxid: {:?}, p1: {:?}, p2: {:?}, desc: {}", c.cxid, c.p1_id.0, c.p2_id.0, c.desc);
                    }
                }
            }
            _ => {
                println!("command '{}' is not implemented", c);
            }
        }            
    }
    Ok(())
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
                .arg(Arg::with_name("address")
                    .index(2)
                    .help("address to pay out to")
                    .required(true)
                    .takes_value(true)),
            SubCommand::with_name("import").about("import payout")
                .arg(Arg::with_name("payout-hex")
                    .index(1)
                    .help("hex-encoded payout")
                    .required(true)
                    .takes_value(true)),
            SubCommand::with_name("details").about("show payout details")
                .arg(Arg::with_name("cxid")
                    .index(1)
                    .value_name("CXID")
                    .help("contract id of payout")
                    .required(true)
                    .takes_value(true)),
            SubCommand::with_name("sign").about("sign payout")
                .arg(Arg::with_name("cxid")
                    .index(1)
                    .help("contract id of payout to sign")
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

pub fn payout_subcommand(subcommand: (&str, Option<&ArgMatches>), wallet: &PlayerWallet) -> TgResult<()> {
    if let (c, Some(a)) = subcommand {
        match c {
            "new" => {
                let contracts = wallet.db.all_contracts().unwrap();
                for c in contracts {
                    if c.cxid == a.value_of("cxid").unwrap() {
                        let contract = Contract::from_bytes(hex::decode(c.hex).unwrap());
                        let payout = tglib::wallet::create_payout(&contract, &Address::from_str(a.value_of("address").unwrap()).unwrap());
                        wallet.db.insert_payout(db::PayoutRecord::from(payout));
                        break;
                    }
                }
                println!("new {:?}", a);
            }
            "import" => {
                println!("import");
            }
            "details" => {
                let payouts = wallet.db.all_payouts().unwrap();
                for p in payouts {
                    if p.cxid == a.value_of("cxid").unwrap() {
                        println!("{:?}", p);
                        break;
                    }
                }
            }
            "sign" => {
                let payouts = wallet.db.all_payouts().unwrap();
                for p in payouts {
                    if p.cxid == a.value_of("cxid").unwrap() {
                        let tx: PartiallySignedTransaction = consensus::deserialize(&hex::decode(p.tx).unwrap()).unwrap();
                        let signing_wallet = Trezor::new(Mnemonic::parse(PLAYER_1_MNEMONIC).unwrap());
                        let tx = signing_wallet.sign_tx(tx, String::from("")).unwrap();
                        let tx = hex::encode(consensus::serialize(&tx));
                        wallet.db.insert_payout(db::PayoutRecord {
                            cxid: p.cxid, 
                            tx,
                            sig: p.sig}
                        ).unwrap();
                        break;
                    }
                }
            }
            "delete" => {
                let r = wallet.db.delete_payout(a.value_of("cxid").unwrap().to_string());
                if r.is_ok() {
                    println!("deleted payout for contract {}", a.value_of("cxid").unwrap());
                }
            }
            "list" => {
                println!("list {:?}", a);
                let payouts = wallet.db.all_payouts().unwrap();
                for p in payouts {
                    println!("{:?}", p);
                }
            }
            _ => {
                println!("command '{}' is not implemented", c);
            }
        }            
    }
    Ok(())
}

