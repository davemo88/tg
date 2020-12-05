use clap::{
    App, 
    Arg
};
use tglib::{
    bdk::bitcoin::{
        PrivateKey,
        secp256k1::{
            Message,
            Secp256k1,
        },
    },
    hex,
    mock::REFEREE_PRIVKEY,
};

fn main() {
    let matches = App::new("referee-signer")
        .version("1.0")
        .author("dk")
        .about("referee signing tool")
        .arg(Arg::with_name("txid")
            .index(1)
            .required(true))
    .get_matches();

// TODO: validation
    if let Some(txid) = matches.value_of("txid") {
        let secp = Secp256k1::new();
        let key = PrivateKey::from_wif(REFEREE_PRIVKEY).unwrap();
        let msg = Message::from_slice(&hex::decode(txid).unwrap()).unwrap();
        let sig = secp.sign(&msg, &key.key);

        println!("{}", hex::encode(sig.serialize_compact()));
    }
}
