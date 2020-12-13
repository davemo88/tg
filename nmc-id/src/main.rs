use std::{
    collections::HashMap,
    str::FromStr,
};
use reqwest;
use warp::{
    Filter,
    Reply,
    Rejection,
};
use serde_json;
use tglib::{
    bdk::{
        bitcoin::{
            Network,
            PublicKey,
            secp256k1::{
                Signature,
            },
            util::{
                base58,
            },
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

mod rpc;
use rpc::{
    TxIn,
    TxOut,
    NamecoinRpc,
    NameOp,
    NamecoinRpcClient,
    NAMECOIN_RPC_URL,
};


type WebResult<T> = std::result::Result<T, Rejection>;
type NamecoinAddress = String;

// mainnet
//const NAMECOIN_VERSION_BYTE: u8 = 0x34;//52
// testnet / regtest, same as bitcoin?
const NAMECOIN_TESTNET_VERSION_BYTE: u8 = 0x6F;//111

#[derive(Clone)]
struct NmcId {
    pub rpc_client: NamecoinRpcClient,
}

impl NmcId {
    pub fn new() -> Self {
        NmcId {
            rpc_client: NamecoinRpcClient::new(NAMECOIN_RPC_URL),
        }
    }

    fn generate(&self, nblocks: u8) -> Result<(), String> {
        let address = self.rpc_client.get_new_address().unwrap();
        let _r = self.rpc_client.generate_to_address(nblocks, address);
        Ok(())
    }

    fn get_txid(&self, tx_hex: String) -> Result<String, String> {
        let decoded = self.rpc_client.decode_raw_transaction(tx_hex, false).unwrap();
        let result = decoded.result.unwrap();
        Ok(result.txid)
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
        let _r = self.rpc_client.import_pubkey(pubkey);
        let name_address = get_namecoin_address(pubkey, NETWORK).unwrap();
        println!("name_address: {}", name_address);
        let name = format!("player/{}",name.0);
// then build a transaction for the name operation
// then do name_new followed by name_firstupdate and keep track of the RAND salt value
// use our address for the name_new so we can spend it later for name_firstupdate
        let new_address = self.rpc_client.get_new_address().unwrap();
        let (name_new_txid, rand) = self.rpc_client.name_new(&name, &new_address).unwrap();
        let _r = self.generate(13);
        let name_firstupdate_txid = self.rpc_client.name_firstupdate(&name, &rand, &name_new_txid, Some("hello world"), &name_address).unwrap();
        println!("name_firstupdate_txid: {}", name_firstupdate_txid);
        let _r = self.generate(1);
        let r = self.rpc_client.name_list(None).unwrap();
        println!("name list:\n{:?}", r);
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
    let nmc_id = NmcId::new();
    
    let nmc_id = warp::any().map(move || nmc_id.clone());

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
        wallet::NAMECOIN_ACCOUNT_PATH,
        mock::{
            ESCROW_SUBACCOUNT,
            ESCROW_KIX,
            PLAYER_1_MNEMONIC,
        },
    };

    const PUBKEY: &'static str = "02123e6a7816f2149f90cca1ea1ba41b73e77db44cd71f01c184defd10961d03fc";
    const TESTNET_ADDRESS_FROM_NAMECOIND: &'static str = "mfuf8qvMsMJMgBqtEGBt8aCQPQi1qgANzo";
    const TEST_NAME: &'static str = "player/test";

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
        
        let mut engine = sha256::HashEngine::default();
        engine.input(TEST_NAME.as_bytes());
        let hash: &[u8] = &sha256::Hash::from_engine(engine);

        let sig = wallet.sign_message(
            Message::from_slice(hash).unwrap(),
            DerivationPath::from_str(&format!("m/{}/{}/{}", NAMECOIN_ACCOUNT_PATH, ESCROW_SUBACCOUNT, ESCROW_KIX)).unwrap(),
        ).unwrap();

        let nmc_id = NmcId::new();
        let _r = nmc_id.rpc_client.load_wallet("testwallet");
//        nmc_id.generate(150);
        let name = nmc_id.register_name(PlayerName("AustinPompeii".to_string()), &pubkey, sig);
    }
}
