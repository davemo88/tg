use clap::{App, Arg, SubCommand, AppSettings};

pub fn repl<'a, 'b>() -> App<'a, 'b> {
    App::new("wallet")
        .version(option_env!("CARGO_PKG_VERSION").unwrap_or("unknown"))
        .author(option_env!("CARGO_PKG_AUTHORS").unwrap_or(""))
        .about("wallet ops")
        .settings(&[AppSettings::NoBinaryName, AppSettings::SubcommandRequiredElseHelp,
            AppSettings::VersionlessSubcommands])
        .subcommand(SubCommand::with_name("balance").about("Display balances (in sats)"))
        .subcommand(player_ui())
        .subcommand(contract_ui())
}

pub fn player_ui<'a, 'b>() -> App<'a, 'b> {
    App::new("player")
        .version(option_env!("CARGO_PKG_VERSION").unwrap_or("unknown"))
        .author(option_env!("CARGO_PKG_AUTHORS").unwrap_or(""))
        .about("player ops")
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

pub fn contract_ui<'a, 'b>() -> App<'a, 'b> {
    App::new("contract")
        .version(option_env!("CARGO_PKG_VERSION").unwrap_or("unknown"))
        .author(option_env!("CARGO_PKG_AUTHORS").unwrap_or(""))
        .about("contract ops")
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