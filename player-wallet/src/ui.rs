use clap::{App, Arg, SubCommand, AppSettings};

pub fn cli<'a, 'b>() -> App<'a, 'b> {
    App::new("player-wallet")
        .version(option_env!("CARGO_PKG_VERSION").unwrap_or("unknown"))
        .author(option_env!("CARGO_PKG_AUTHORS").unwrap_or(""))
        .about("player wallet native app")
        .arg(Arg::with_name("connections")
            .short("c")
            .long("connections")
            .value_name("NUMBER")
            .help("number of peer connections")
            .required(true)
            .takes_value(true)
            .default_value("5")
        )
        .arg(Arg::with_name("data")
            .short("d")
            .long("data")
            .value_name("DIRECTORY")
            .help("data directory")
            .required(false)
            .takes_value(true)
            .default_value(".")
        )
        .arg(Arg::with_name("discovery")
            .short("i")
            .long("discovery")
            .help("turn peer discovery on or off")
            .required(false)
            .takes_value(true)
            .default_value("on")
            .possible_values(&["on", "off"])
        )
        .arg(Arg::with_name("logging")
            .short("l")
            .long("log")
            .value_name("LEVEL")
            .help("logging level")
            .required(false)
            .takes_value(true)
            .default_value("info")
            .possible_values(&["debug", "info", "warn", "error"])
        )
        .arg(Arg::with_name("network")
            .short("n")
            .long("net")
            .value_name("NETWORK")
            .help("bitcoin network")
            .required(false)
            .takes_value(true)
            .default_value("testnet")
            .possible_values(&["regtest", "testnet"])
        )
        .arg(Arg::with_name("password")
            .short("p")
            .long("password")
            .value_name("PASSWORD")
            .help("wallet password")
            .required(true)
            .takes_value(true)
        )
        .arg(Arg::with_name("peers")
            .short("a")
            .long("peers")
            .value_name("IP_ADDRESS")
            .help("ip addresses of peer nodes, eg. 127.0.0.1:9333")
            .required(false)
            .takes_value(true)
            .multiple(true)
        )
}

pub fn repl<'a, 'b>() -> App<'a, 'b> {
    App::new("wallet")
        .version(option_env!("CARGO_PKG_VERSION").unwrap_or("unknown"))
        .author(option_env!("CARGO_PKG_AUTHORS").unwrap_or(""))
        .about("wallet ops")
        .settings(&[AppSettings::NoBinaryName, AppSettings::SubcommandRequiredElseHelp,
            AppSettings::VersionlessSubcommands])
        .subcommands(vec![SubCommand::with_name("stop").about("Stop wallet"),
                          SubCommand::with_name("balance").about("Display balances (in sats)"),
                          SubCommand::with_name("deposit").about("Display deposit address"),
                          SubCommand::with_name("withdraw").about("Withdraw sats to address")
                              .arg(Arg::with_name("password")
                                  .short("p")
                                  .long("password")
                                  .value_name("PASSWORD")
                                  .help("wallet password")
                                  .required(true)
                                  .takes_value(true))
                              .arg(Arg::with_name("address")
                                  .short("d")
                                  .long("destination")
                                  .value_name("ADDRESS")
                                  .help("destination address")
                                  .required(true)
                                  .takes_value(true))
                              .arg(Arg::with_name("fee")
                                  .short("f")
                                  .long("fee")
                                  .value_name("SATS")
                                  .help("sats per vbyte")
                                  .required(true)
                                  .takes_value(true))
                              .arg(Arg::with_name("amount")
                                  .short("a")
                                  .long("amount")
                                  .value_name("SATS")
                                  .help("amount of sats to withdraw")
                                  .required(true)
                                  .takes_value(true))])
        .subcommand(contract_ui())
        .subcommand(player_ui())
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
                    .short("p2")
                    .long("player-2")
                    .value_name("PLAYER2")
                    .help("player 2's id")
                    .required(true)
                    .takes_value(true))
                .arg(Arg::with_name("amount")
                    .short("a")
                    .long("amount")
                    .value_name("AMOUNT")
                    .help("amount")
                    .required(true)
                    .takes_value(true))
                .arg(Arg::with_name("referee")
                    .short("r")
                    .long("referee")
                    .value_name("REFEREE")
                    .help("referee id")
                    .required(false)
                    .takes_value(true)),
            SubCommand::with_name("list").about("list all contracts"),
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
        ])
}
