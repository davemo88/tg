use std::str::FromStr;
use bdk::bitcoin::{
    Address,
    Amount,
    consensus,
    secp256k1::{
        Message,
        Signature,
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
    arbiter::ArbiterService,
    contract::Contract,
    payout::Payout,
    player::PlayerId,
    wallet::{
        EscrowWallet,
        SigningWallet,
        ESCROW_SUBACCOUNT,
    },
    mock::{
        Trezor,
        ARBITER_PUBLIC_URL,
        ESCROW_KIX,
        NETWORK,
        PAYOUT_VERSION,
        PLAYER_1_MNEMONIC,
    },
};
use crate::{
    arbiter::ArbiterClient,
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
}

pub fn wallet_subcommand(subcommand: (&str, Option<&ArgMatches>), wallet: &PlayerWallet) -> TgResult<()> {

    if let (c, Some(_a)) = subcommand {
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
                    Ok(_) => println!("added player {} named {}", player.id.0, player.name),
                    Err(e) => println!("{:?}", e),
                }
            }
            "list" => {
                let players = wallet.db.all_players().unwrap();
                for p in players {
                    println!("id: {}, name: {}", p.id.0, p.name);
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

pub fn contract_subcommand(subcommand: (&str, Option<&ArgMatches>), wallet: &PlayerWallet) -> TgResult<()> {
    if let (c, Some(a)) = subcommand {
        match c {
            "new" => {
                let p2_id = PlayerId(a.value_of("player-2").unwrap().to_string());
                let amount = Amount::from_sat(a.value_of("amount").unwrap().parse::<u64>().unwrap());
                let desc = a.value_of("desc").unwrap_or("");
                let arbiter_client = ArbiterClient::new(ARBITER_PUBLIC_URL);
                let p2_contract_info = arbiter_client.get_player_info(p2_id.clone()).unwrap();
                let arbiter_pubkey = arbiter_client.get_escrow_pubkey().unwrap();

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
                    Ok(_) => println!("created contract {}", contract_record.cxid),
                    Err(e) => println!("{:?}", e),
                }
            }
            "import" => {
                if let Ok(contract) = Contract::from_bytes(hex::decode(a.value_of("hex").unwrap()).unwrap()) {
                    let contract_record = db::ContractRecord {
                        cxid: hex::encode(contract.cxid()),
                        p1_id: PlayerId::from(contract.p1_pubkey),
                        p2_id: PlayerId::from(contract.p2_pubkey),
                        hex: hex::encode(contract.to_bytes()),
                        desc: String::default(),
                    };
                    match wallet.db.insert_contract(contract_record.clone()) {
                        Ok(_) => println!("imported contract {}", hex::encode(contract.cxid())),
                        Err(e) => println!("{:?}", e),
                    }
                } else {
                    println!("invalid contract");
                }

            }
            "details" => {
                if let Some(contract_record) = wallet.db.get_contract(a.value_of("cxid").unwrap()) {
                    let contract = Contract::from_bytes(hex::decode(contract_record.hex.clone()).unwrap()).unwrap();
                    println!("{:?}", contract);
                    println!("hex: {}", contract_record.hex);

                } else {
                    println!("no such contract");
                }
            }
            "sign" => {
                if let Some(contract_record) = wallet.db.get_contract(a.value_of("cxid").unwrap()) {
//                    let contract = Contract::from_bytes(hex::decode(contract_record.hex.clone()).unwrap()).unwrap();
                    let signing_wallet = Trezor::new(Mnemonic::parse(PLAYER_1_MNEMONIC).unwrap());
                    let sig = signing_wallet.sign_message(
                        Message::from_slice(&hex::decode(contract_record.cxid.clone()).unwrap()).unwrap(),
                        DerivationPath::from_str(&format!("m/{}/{}", ESCROW_SUBACCOUNT, ESCROW_KIX)).unwrap(),
                    ).unwrap();
                    let mut contract = Contract::from_bytes(hex::decode(contract_record.hex.clone()).unwrap()).unwrap();
                    contract.sigs.push(sig);
                    let _r = wallet.db.add_signature(contract_record.cxid, hex::encode(contract.to_bytes()));
//                    assert_ne!(hex::encode(contract.to_bytes()), contract_record.hex);
                    println!("signed contract");

                } else {
                    println!("no such contract");
                }
            }
            "submit" => {
                if let Some(cr) = wallet.db.get_contract(a.value_of("cxid").unwrap()) {
                    let arbiter_client = ArbiterClient::new(ARBITER_PUBLIC_URL);
                    let mut contract = Contract::from_bytes(hex::decode(cr.hex.clone()).unwrap()).unwrap();
                    if let Ok(sig) = arbiter_client.submit_contract(&contract) {
                       contract.sigs.push(sig);
                       let _r = wallet.db.add_signature(cr.cxid, hex::encode(contract.to_bytes()));
                       println!("arbiter accepted and signed contract");
                    }
                    else {
                       println!("arbiter rejected contract");
                    }
                }
            }
            "broadcast" => {
                if let Some(cr) = wallet.db.get_contract(a.value_of("cxid").unwrap()) {
                    let contract = Contract::from_bytes(hex::decode(cr.hex.clone()).unwrap()).unwrap();
                    let _r = wallet.wallet.broadcast(contract.funding_tx);
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
                for c in contracts {
                    println!("cxid: {:?}, p1: {:?}, p2: {:?}, desc: {}", c.cxid, c.p1_id.0, c.p2_id.0, c.desc);
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

pub fn payout_subcommand(subcommand: (&str, Option<&ArgMatches>), wallet: &PlayerWallet) -> TgResult<()> {
    if let (c, Some(a)) = subcommand {
        match c {
            "new" => {
                let contract_record = wallet.db.get_contract(a.value_of("cxid").unwrap()).unwrap();
                let contract = Contract::from_bytes(hex::decode(contract_record.hex).unwrap()).unwrap();
                let escrow_pubkey = wallet.get_escrow_pubkey();
                let payout = tglib::wallet::create_payout(&contract, &Address::p2wpkh(&escrow_pubkey, NETWORK).unwrap());
                let _r = wallet.db.insert_payout(db::PayoutRecord::from(payout.clone()));
                println!("new payout {:?}", payout);
            }
            "import" => {
                println!("import");
            }
            "details" => {
                if let Some(pr) = wallet.db.get_payout(a.value_of("cxid").unwrap()) {
                    println!("{:?}", pr);
                }
                else {
                    println!("no such payout");
                }
            }
            "sign" => {
                if let Some(pr) = wallet.db.get_payout(a.value_of("cxid").unwrap()) {
                    let tx: PartiallySignedTransaction = consensus::deserialize(&hex::decode(pr.tx).unwrap()).unwrap();
                    let signing_wallet = Trezor::new(Mnemonic::parse(PLAYER_1_MNEMONIC).unwrap());
                    let tx = signing_wallet.sign_tx(tx, "".to_string()).unwrap(); let tx = hex::encode(consensus::serialize(&tx));
                    wallet.db.insert_payout(db::PayoutRecord {
                        cxid: pr.cxid, 
                        tx,
                        sig: a.value_of("script-sig").unwrap_or("").to_string(),
                    }).unwrap();
                    println!("signed payout");
                }
                else {
                    println!("no such payout");
                }
            }
            "submit" => {
                let cxid = a.value_of("cxid").unwrap();
                let cr = wallet.db.get_contract(cxid).unwrap();
                let pr = wallet.db.get_payout(cxid).unwrap();
                let p = Payout {
                    version: PAYOUT_VERSION,
                    contract: Contract::from_bytes(hex::decode(cr.hex).unwrap()).unwrap(),
                    tx: consensus::deserialize(&hex::decode(pr.tx).unwrap()).unwrap(),
                    script_sig: Signature::from_compact(&hex::decode(pr.sig).unwrap()).ok()
                };
                let arbiter_client = ArbiterClient::new(ARBITER_PUBLIC_URL);
                if let Ok(tx) = arbiter_client.submit_payout(&p) {
                   println!("arbiter signed tx: {:?}", tx);
                }
                else {
                   println!("arbiter rejected payout");
                }
            }
            "broadcast" => {
                if let Some(pr) = wallet.db.get_payout(a.value_of("cxid").unwrap()) {
                    let tx: PartiallySignedTransaction = consensus::deserialize(&hex::decode(pr.tx).unwrap()).unwrap();
                    let _r = wallet.wallet.broadcast(tx.extract_tx());
                }
                else {
                    println!("no such payout");
                }
            }
            "delete" => {
                let r = wallet.db.delete_payout(a.value_of("cxid").unwrap().to_string());
                if r.is_ok() {
                    println!("deleted payout for contract {}", a.value_of("cxid").unwrap());
                }
            }
            "list" => {
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
