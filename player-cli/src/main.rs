use std::{
    env::current_dir,
    path::PathBuf,
};
use rustyline::Editor;
use rustyline::error::ReadlineError;
use tglib::{
    bdk::Error,
    mock::NETWORK,
};

use libcli;

fn main() -> Result<(), Error> {

    let work_dir: PathBuf = current_dir().unwrap();
    let mut history_file = work_dir.clone();
    history_file.push(&NETWORK.to_string());
    history_file.push("history.txt");
    let history_file = history_file.as_path();

    let mut rl = Editor::<()>::new();

    if rl.load_history(history_file).is_err() {
        println!("No previous history.");
    }

    loop {
        let readline = rl.readline(">> ");
        match readline {
            Ok(line) => {
                let conf = libcli::Conf {
                    electrum_url: "tcp://localhost:60401".into(),
                    name_url: "http://localhost:18420".into(),
                    arbiter_url: "http://localhost:5000".into(),
                };
                rl.add_history_entry(line.clone());
                println!("{}", libcli::cli(line, conf));
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
