use std::collections::HashMap;
use reqwest;
use warp::{
    Filter,
    Reply,
    Rejection,
};
use serde_json;
use serde::{
    Serialize,
    Deserialize,
};
use tglib::{
    bdk::{
        bitcoin::{
            Network,
            PublicKey,
            secp256k1::Signature,
            util::base58,
            hashes::{
                ripemd160,
                sha256,
                HashEngine,
                Hash,
            },
        },
        blockchain::noop_progress,
        electrum_client::Client,
    },
    bip39::Mnemonic,
    contract::PlayerContractInfo,
    player::{
        PlayerId,
        PlayerIdService,
        PlayerName,
        PlayerNameService,
    },
    wallet::{
        EscrowWallet,
        SigningWallet,
    },
    mock::{
        Trezor,
        ELECTRS_SERVER,
        PLAYER_2_MNEMONIC,
        NETWORK,
    }
};
use player_wallet::wallet::PlayerWallet;

const NAMECOIN_RPC_URL: &'static str = "http://guyledouche:yodelinbabaganoush@localhost:18443";

type WebResult<T> = std::result::Result<T, Rejection>;
type NamecoinAddress = String;

const EMPTY_TX: &'static str = "01000000000000000000";
// mainnet
const NAMECOIN_VERSION_BYTE: u8 = 0x34;//52
// testnet / regtest, same as bitcoin?
const NAMECOIN_TESTNET_VERSION_BYTE: u8 = 0x6F;//111

const JSONRPC_VERSION: &'static str = "1.0";
const JSONRPC_ID: &'static str = "nmc-id-test";

#[derive(Clone)]
struct NmcId;

#[derive(Debug, Serialize, Deserialize)]
struct RpcResponse {
    result: Option<String>,
    error: Option<String>,
    message: Option<String>,
    id: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
struct NameNewResult {
    hex: String,
    rand: String,
}

#[derive(Debug, Serialize, Deserialize)]
struct NameNewResponse {
    result: Option<NameNewResult>,
    error: Option<String>,
    message: Option<String>,
    id: Option<String>,
}

impl NmcId {
    pub fn new() -> Self {
        NmcId
    }
}

impl PlayerNameService for NmcId {
    fn get_player_name(&self, pubkey: &PublicKey) -> Option<PlayerName> {
        None
    }

    fn get_contract_info(&self, name: PlayerName) -> Option<PlayerContractInfo> {
        None
    }
    fn set_contract_info(&self, name: PlayerName, info: PlayerContractInfo, sig: Signature) -> Option<PlayerContractInfo> {
        None
    }
    fn register_name(&self, name: PlayerName, pubkey: &PublicKey, sig: Signature) -> Result<(), String> {
// create namecoin address from supplied pubkey
        let address = get_namecoin_address(&pubkey, NETWORK).unwrap();
// then build a transaction for the name operation
// then do name_new followed by name_firstupdate and keep track of the RAND salt value
//
        let client = reqwest::blocking::Client::new();
        let string_body = format!("{{\"jsonrpc\": \"{}\", \"id\": \"{}\", \"method\": \"createrawtransaction\", \"params\": [[], [{{\"{}\":\"0.01\"}}]]}}",
            JSONRPC_VERSION,
            JSONRPC_ID,
            address);
        println!("{}",string_body);
        let r = client.post(NAMECOIN_RPC_URL)
            .body(string_body)
            .send()
            .unwrap();
        let r: RpcResponse = r.json().unwrap();
        let tx_hex = r.result.unwrap();
        let string_body = format!("{{\"jsonrpc\": \"{}\", \"id\": \"{}\", \"method\": \"namerawtransaction\", \"params\": [\"{}\", 0, {{\"op\":\"name_new\", \"name\":\"player/test\"}}]}}",
            JSONRPC_VERSION,
            JSONRPC_ID,
            tx_hex.clone());
        println!("{}",string_body);
        let r = client.post(NAMECOIN_RPC_URL)
            .body(string_body)
            .send()
            .unwrap();
//        let r = r.text().unwrap();
        let r: NameNewResponse = r.json().unwrap();
        println!("{:?}", r);
        let name_new_result = r.result.unwrap();
// mine (12?) blocks here or firstupdate won't be valid
// name_firstupdate
        let string_body = format!("{{\"jsonrpc\": \"{}\", \"id\": \"{}\", \"method\": \"namerawtransaction\", \"params\": [\"{}\", 0, {{\"op\":\"name_firstupdate\", \"name\":\"player/test\", \"value\":\"new value\", \"rand\":\"{}\"}}]}}",
            JSONRPC_VERSION,
            JSONRPC_ID,
            tx_hex,
            name_new_result.rand);
        println!("{}",string_body);
        let r = client.post(NAMECOIN_RPC_URL)
            .body(string_body)
            .send()
            .unwrap();
        let r = r.text().unwrap();
        Ok(())
    }
}

impl PlayerIdService for NmcId {
    fn get_player_id(&self, _pubkey: &PublicKey) -> Option<PlayerId> {
        None
    }

    fn get_player_info(&self, player_id: PlayerId) -> Option<PlayerContractInfo> {
        println!("request for info on player {}", player_id.0);
        let signing_wallet = Trezor::new(Mnemonic::parse(PLAYER_2_MNEMONIC).unwrap());
        let client = Client::new(ELECTRS_SERVER).unwrap();
        let player_wallet = PlayerWallet::new(signing_wallet.fingerprint(), signing_wallet.xpubkey(), NETWORK, client);
        let escrow_pubkey = EscrowWallet::get_escrow_pubkey(&player_wallet);
        player_wallet.wallet.sync(noop_progress(), None).unwrap();
        Some(PlayerContractInfo {
            escrow_pubkey,
            change_address: player_wallet.wallet.get_new_address().unwrap(),
            utxos: player_wallet.wallet.list_unspent().unwrap(),
        })
    }

// TODO this endpoint needs auth requiring ownership of the corresponding player_id
// e.g. if the player_id is a namecoin name, then a signature of some random data
// by the owner of the name will do nicely
//    fn set_player_info(&self, player_id: PlayerId, info: PlayerContractInfo) -> Result<()> {
//        Err(TgError("invalid signature"))
//    }
}

fn get_namecoin_address(pubkey: &PublicKey, network: Network) -> Result<NamecoinAddress, String> {
    let mut sha256_engine = sha256::HashEngine::default();
    sha256_engine.input(&pubkey.key.serialize());
    let hash: &[u8] = &sha256::Hash::from_engine(sha256_engine);

    let mut ripemd160_engine = ripemd160::HashEngine::default();
    ripemd160_engine.input(hash);
    let hash = &ripemd160::Hash::from_engine(ripemd160_engine);

    let mut hash = hash.to_vec();
    match network {
        Network::Bitcoin => {
            panic!("nice try, sucker");
//            hash.insert(0,NAMECOIN_VERSION_BYTE);
        },
        Network::Regtest | Network::Testnet => {
            hash.insert(0,NAMECOIN_TESTNET_VERSION_BYTE);
        }
    }

    Ok(base58::check_encode_slice(&hash))
}

async fn id_handler(_pubkey: String, _nmcid: NmcId) -> WebResult<impl Reply>{
    Ok("not implemented".to_string())
}

async fn info_handler(player_id: String, nmcid: NmcId) -> WebResult<impl Reply>{
    let info = nmcid.get_player_info(PlayerId(player_id)).unwrap();
    Ok(serde_json::to_string(&info).unwrap())
}

#[tokio::main]
async fn main() {
    
    let nmc_id = warp::any().map(move || NmcId::new());

    let get_player_id = warp::path("get-player-id")
        .and(warp::path::param::<String>())
        .and(nmc_id.clone())
        .and_then(id_handler);

    let get_player_info = warp::path("get-player-info")
        .and(warp::path::param::<String>())
        .and(nmc_id.clone())
        .and_then(info_handler);

    let routes = get_player_id.or(get_player_info);

    warp::serve(routes).run(([0, 0, 0, 0], 18420)).await;
}

#[cfg(test)]
mod tests {

    use super::*;
    use std::str::FromStr;
    use tglib::{
        hex,
        bdk::bitcoin::{
            secp256k1::Message,
            util::bip32::DerivationPath,
        },
        mock::{
            BITCOIN_ACCOUNT_PATH,
            ESCROW_SUBACCOUNT,
            ESCROW_KIX,
            PLAYER_1_MNEMONIC,
        },
    };

    const PUBKEY: &'static str = "02123e6a7816f2149f90cca1ea1ba41b73e77db44cd71f01c184defd10961d03fc";
    const TESTNET_ADDRESS_FROM_NAMECOIND: &'static str = "mfuf8qvMsMJMgBqtEGBt8aCQPQi1qgANzo";
    const TEST_NAME: &'static str = "player/name";

    #[test]
    fn test_get_namecoin_address() {
        let pubkey = PublicKey::from_slice(&hex::decode(PUBKEY).unwrap()).unwrap();
        let namecoin_address = get_namecoin_address(&pubkey, Network::Testnet).unwrap();
//        println!("{}", namecoin_address);
        assert_eq!(namecoin_address,TESTNET_ADDRESS_FROM_NAMECOIND)
    }

    #[test]
    fn test_register_name() {
        let wallet = Trezor::new(Mnemonic::parse(PLAYER_1_MNEMONIC).unwrap());
        let pubkey = wallet.get_escrow_pubkey();
        let namecoin_address = get_namecoin_address(&pubkey, NETWORK).unwrap();
        
        let mut engine = sha256::HashEngine::default();
        engine.input(TEST_NAME.as_bytes());
        let hash: &[u8] = &sha256::Hash::from_engine(engine);

        let nid = NmcId;
        let sig = wallet.sign_message(
            Message::from_slice(hash).unwrap(),
// TODO: NAMECOIN_ACCOUNT_PATH etc
            DerivationPath::from_str(&format!("m/{}/{}/{}", BITCOIN_ACCOUNT_PATH, ESCROW_SUBACCOUNT, ESCROW_KIX)).unwrap(),
        ).unwrap();
        let name = nid.register_name(PlayerName("AustinPompeii".to_string()), &pubkey, sig);
    }
}
