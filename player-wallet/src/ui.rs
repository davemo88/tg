use std::str::FromStr;
use clap::{App, Arg, ArgMatches, SubCommand, AppSettings};
use tglib::{
    bdk::bitcoin::{
        Address,
        Amount,
        consensus,
        hashes::{
            sha256,
            Hash,
            HashEngine,
        },
        secp256k1::{
            Message,
            Signature,
        },
        util::{
            bip32::DerivationPath,
            psbt::PartiallySignedTransaction,
        },
    },
    hex,
    bip39::Mnemonic,
    Result as TgResult,
    arbiter::ArbiterService,
    contract::Contract,
    payout::Payout,
    player::{
        PlayerName,
        PlayerNameService,
    },
    wallet::{
        EscrowWallet,
        NameWallet,
        SigningWallet,
        ESCROW_SUBACCOUNT,
        NAME_SUBACCOUNT,
        NAME_KIX,
    },
    mock::{
        Trezor,
        ARBITER_PUBLIC_URL,
        ESCROW_KIX,
        NETWORK,
        PAYOUT_VERSION,
        NAME_SERVICE_URL,
        PLAYER_1_MNEMONIC,
    },
};
use crate::{
    arbiter::ArbiterClient,
    db::{
        self,
        ContractRecord,
        PayoutRecord,
    },
    player::PlayerNameClient,
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
            SubCommand::with_name("name").about("shows local player name"),
        ])
}

pub fn player_subcommand(subcommand: (&str, Option<&ArgMatches>), wallet: &PlayerWallet) -> TgResult<()> {
    if let (c, Some(a)) = subcommand {
        match c {
            "register" => {
                let name = PlayerName(a.args["name"].vals[0].clone().into_string().unwrap());
                let name_client = PlayerNameClient::new(NAME_SERVICE_URL);
                let signing_wallet = Trezor::new(Mnemonic::parse(PLAYER_1_MNEMONIC).unwrap());
                let mut engine = sha256::HashEngine::default();
                engine.input(name.0.as_bytes());
                let hash: &[u8] = &sha256::Hash::from_engine(engine);
                let sig = signing_wallet.sign_message(
                    Message::from_slice(hash).unwrap(),
                    DerivationPath::from_str(&format!("m/{}/{}", NAME_SUBACCOUNT, NAME_KIX)).unwrap(),
                ).unwrap();
                if name_client.register_name(name.clone(), wallet.name_pubkey(), sig).is_ok() {
                    let pr = db::PlayerRecord {
                        name: name.clone(),
                    };
                    match wallet.db.insert_player(pr) {
                        Ok(_) => println!("added player {}", name.0),
                        Err(e) => println!("{:?}", e),
                    }
                } else {
                    println!("couldn't register name");
                }

            }
            "add" => {
                let player = db::PlayerRecord {
                    name:       PlayerName(a.args["name"].vals[0].clone().into_string().unwrap()),
                };
                match wallet.db.insert_player(player.clone()) {
                    Ok(_) => println!("added player {}", player.name.0),
                    Err(e) => println!("{:?}", e),
                }
            }
            "list" => {
                let players = wallet.db.all_players().unwrap();
                for p in players {
                    println!("player name: {}", p.name.0);
                }
            }
            "remove" => {
                let player_name = PlayerName(a.args["name"].vals[0].clone().into_string().unwrap());
                match wallet.db.delete_player(player_name.clone()) {
                    Ok(num_deleted) => match num_deleted {
                        0 => println!("no player with that name"),
                        1 => println!("removed player {}", player_name.0),
                        n => panic!("{} removed, should be impossible", n),//this is impossible
                    }
                    Err(e) => println!("{:?}", e),
                }
            },
            "name" => {
                println!("{}", wallet.player_name().0);
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

pub fn contract_subcommand(subcommand: (&str, Option<&ArgMatches>), wallet: &PlayerWallet) -> TgResult<()> {
    if let (c, Some(a)) = subcommand {
        match c {
            "new" => {
// TODO: confirm local ownership of p1_name
                let p1_name = PlayerName(a.value_of("player-1").unwrap().to_string());
                let p2_name = PlayerName(a.value_of("player-2").unwrap().to_string());
                let amount = Amount::from_sat(a.value_of("amount").unwrap().parse::<u64>().unwrap());
                let desc = a.value_of("desc").unwrap_or("");
                let arbiter_client = ArbiterClient::new(ARBITER_PUBLIC_URL);
//                let p2_contract_info = arbiter_client.get_player_info(p2_name.clone()).unwrap();
                let player_name_client = PlayerNameClient::new(NAME_SERVICE_URL);
                let p2_contract_info = player_name_client.get_contract_info(p2_name.clone()).unwrap();
                let arbiter_pubkey = arbiter_client.get_escrow_pubkey().unwrap();

//                let signing_wallet = Trezor::new(Mnemonic::parse(PLAYER_1_MNEMONIC).unwrap());
//                let player_wallet = PlayerWallet::new(signing_wallet.fingerprint(), signing_wallet.xpubkey(), NETWORK);

                let contract = wallet.create_contract(p2_contract_info, amount, arbiter_pubkey);
                let contract_record = db::ContractRecord {
                    cxid: hex::encode(contract.cxid()),
                    p1_name,
                    p2_name,
                    hex: hex::encode(contract.to_bytes()),
                    desc: desc.to_string(),
                };

                match wallet.db.insert_contract(contract_record.clone()) {
                    Ok(_) => println!("created contract {}", contract_record.cxid),
                    Err(e) => println!("{:?}", e),
                }
            }
            "import" => {
// accept both contracts and contract records in binary
// contract record binary encoding can be defined in player-wallet libs
// is not a necessarily standardized encoding like contract
// try to parse contract record first, since it contains more info
                if let Ok(contract_record) = serde_json::from_str::<ContractRecord>(a.value_of("contract-value").unwrap()) {
                    match wallet.db.insert_contract(contract_record.clone()) {
                        Ok(_) => println!("imported contract {}", hex::encode(contract_record.cxid)),
                        Err(e) => println!("{:?}", e),
                    }
                }
// disable this for now until we decide how to address missing names
//                else if let Ok(contract) = Contract::from_bytes(hex::decode(a.value_of("contract-value").unwrap()).unwrap()) {
//// contract alone doesn't have player names
//                    let contract_record = db::ContractRecord {
//                        cxid: hex::encode(contract.cxid()),
//                        p1_name: PlayerName::from(contract.p1_pubkey),
//                        p2_name: PlayerName::from(contract.p2_pubkey),
//                        hex: hex::encode(contract.to_bytes()),
//                        desc: String::default(),
//                    };
//                    match wallet.db.insert_contract(contract_record.clone()) {
//                        Ok(_) => println!("imported contract {}", hex::encode(contract.cxid())),
//                        Err(e) => println!("{:?}", e),
//                    }
//                } 
                else {
                    println!("invalid contract");
                }
            }
            "export" => {
                if let Some(contract_record) = wallet.db.get_contract(a.value_of("cxid").unwrap()) {
                    println!("{}", contract_record.hex);

                } else {
                    println!("no such contract");
                }
            }
            "details" => {
                if let Some(contract_record) = wallet.db.get_contract(a.value_of("cxid").unwrap()) {
                    let contract = Contract::from_bytes(hex::decode(contract_record.hex.clone()).unwrap()).unwrap();
                    println!("{:?}", contract);

                } else {
                    println!("no such contract");
                }
            }
            "sign" => {
                if let Some(contract_record) = wallet.db.get_contract(a.value_of("cxid").unwrap()) {
//                    let contract = Contract::from_bytes(hex::decode(contract_record.hex.clone()).unwrap()).unwrap();
//                    println!("{:?}",a);
                    let signing_wallet = Trezor::new(Mnemonic::parse(PLAYER_1_MNEMONIC).unwrap());
                    let sig = signing_wallet.sign_message(
                        Message::from_slice(&hex::decode(contract_record.cxid.clone()).unwrap()).unwrap(),
                        DerivationPath::from_str(&format!("m/{}/{}", ESCROW_SUBACCOUNT, ESCROW_KIX)).unwrap(),
                    ).unwrap();
                    let mut contract = Contract::from_bytes(hex::decode(contract_record.hex.clone()).unwrap()).unwrap();
                    contract.sigs.push(sig);
                    if a.value_of("sign-funding-tx").is_some() {
                        contract.funding_tx = signing_wallet.sign_tx(contract.funding_tx.clone(), "".to_string()).unwrap();
                    }
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
                    let _r = wallet.wallet.broadcast(contract.funding_tx.extract_tx());
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
                    println!("cxid: {:?}, p1: {:?}, p2: {:?}, desc: {}", c.cxid, c.p1_name.0, c.p2_name.0, c.desc);
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

pub fn payout_subcommand(subcommand: (&str, Option<&ArgMatches>), wallet: &PlayerWallet) -> TgResult<()> {
    if let (c, Some(a)) = subcommand {
        match c {
            "new" => {
                let contract_record = wallet.db.get_contract(a.value_of("cxid").unwrap()).unwrap();
                let contract = Contract::from_bytes(hex::decode(contract_record.hex).unwrap()).unwrap();
                let escrow_pubkey = wallet.get_escrow_pubkey();
                let payout = tglib::wallet::create_payout(&contract, &Address::p2wpkh(&escrow_pubkey, NETWORK).unwrap());
                let _r = wallet.db.insert_payout(db::PayoutRecord::from(payout.clone()));
                println!("created new payout for contract {}", contract_record.cxid);
            }
            "import" => {
                if let Ok(payout_record) = serde_json::from_str::<PayoutRecord>(a.value_of("payout-value").unwrap()) {
                    let _r = wallet.db.insert_payout(payout_record.clone());
                    println!("import payout for contract {}", payout_record.cxid);
                }
//                if let Ok(payout) = Payout::from_bytes(hex::decode(a.value_of("payout-value").unwrap()).unwrap()) {
//                    let contract_record = db::ContractRecord {
//                        cxid: hex::encode(payout.contract.cxid()),
//                        p1_name: PlayerName::from(payout.contract.p1_pubkey),
//                        p2_name: PlayerName::from(payout.contract.p2_pubkey),
//                        hex: hex::encode(payout.contract.to_bytes()),
//                        desc: String::default(),
//                    };
//                    match wallet.db.insert_contract(contract_record.clone()) {
//                        Ok(_) => println!("imported contract {}", hex::encode(payout.contract.cxid())),
//                        Err(e) => println!("{:?}", e),
//                    }
//                    let _r = wallet.db.insert_payout(db::PayoutRecord::from(payout.clone()));
//                    println!("import payout for contract {}", contract_record.cxid);
//                } 
                else {
                    println!("invalid payout ");
                }
            }
            "export" => {
                let cxid = a.value_of("cxid").unwrap();
                let cr = wallet.db.get_contract(cxid).unwrap();
                let pr = wallet.db.get_payout(cxid).unwrap();
                let p = Payout {
                    version: PAYOUT_VERSION,
                    contract: Contract::from_bytes(hex::decode(cr.hex).unwrap()).unwrap(),
                    psbt: consensus::deserialize(&hex::decode(pr.psbt).unwrap()).unwrap(),
                    script_sig: Signature::from_compact(&hex::decode(pr.sig).unwrap()).ok()
                };
                println!("{}", hex::encode(p.to_bytes()));
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
                    let psbt: PartiallySignedTransaction = consensus::deserialize(&hex::decode(pr.psbt).unwrap()).unwrap();
                    let signing_wallet = Trezor::new(Mnemonic::parse(PLAYER_1_MNEMONIC).unwrap());
                    let psbt = signing_wallet.sign_tx(psbt, "".to_string()).unwrap();
                    let psbt = hex::encode(consensus::serialize(&psbt));
                    wallet.db.insert_payout(db::PayoutRecord {
                        cxid: pr.cxid, 
                        psbt,
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
                    psbt: consensus::deserialize(&hex::decode(pr.psbt).unwrap()).unwrap(),
                    script_sig: Signature::from_compact(&hex::decode(pr.sig).unwrap()).ok()
                };
                let arbiter_client = ArbiterClient::new(ARBITER_PUBLIC_URL);
                if let Ok(psbt) = arbiter_client.submit_payout(&p) {
                   println!("arbiter signed tx: {:?}", psbt.extract_tx().txid());
                }
                else {
                   println!("arbiter rejected payout");
                }
            }
            "broadcast" => {
                if let Some(pr) = wallet.db.get_payout(a.value_of("cxid").unwrap()) {
                    let tx: PartiallySignedTransaction = consensus::deserialize(&hex::decode(pr.psbt).unwrap()).unwrap();
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
                    println!("cxid: {}", p.cxid);
                }
            }
            _ => {
                println!("command '{}' is not implemented", c);
            }
        }            
    }
    Ok(())
}
