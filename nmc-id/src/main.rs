use redis::{
    self,
    Commands,
    RedisResult,
};
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
                Message,
                Secp256k1,
                Signature,
            },
            util::{
                base58,
            },
            hashes::{
                ripemd160,
                sha256,
                HashEngine,
                Hash as BitcoinHash,
            },
        },
    },
    hex,
    contract::PlayerContractInfo,
    player::{
        PlayerName,
        PlayerNameService,
    },
    mock::{
        NETWORK,
        REDIS_SERVER,
    }
};

mod rpc;
use rpc::{
    NamecoinRpc,
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
    pub redis_client: redis::Client,
    pub rpc_client: NamecoinRpcClient,
}

impl NmcId {
    pub fn new() -> Self {
        NmcId {
            redis_client: redis::Client::open(REDIS_SERVER).unwrap(),
            rpc_client: NamecoinRpcClient::new(NAMECOIN_RPC_URL),
        }
    }

    fn generate(&self, nblocks: u8) -> Result<(), String> {
        let address = self.rpc_client.get_new_address().unwrap();
        let _r = self.rpc_client.generate_to_address(nblocks, address);
        Ok(())
    }
}

impl PlayerNameService for NmcId {
    fn register_name(&self, name: PlayerName, pubkey: PublicKey, sig: Signature) -> Result<(), String> {
        let mut engine = sha256::HashEngine::default();
        engine.input(name.0.as_bytes());
        let hash: &[u8] = &sha256::Hash::from_engine(engine);

        let secp = Secp256k1::new();
        if secp.verify(&Message::from_slice(hash).unwrap(), &sig, &pubkey.key).is_err() {
            return Err("invalid signature".to_string())
        }
        let _r = self.rpc_client.import_pubkey(&pubkey);
// create namecoin address from supplied pubkey
        let name_address = get_namecoin_address(&pubkey, NETWORK).unwrap();
//        println!("name_address: {}", name_address);
        let name = format!("player/{}",name.0);
        let new_address = self.rpc_client.get_new_address().unwrap();
        let (name_new_txid, rand) = self.rpc_client.name_new(&name, &new_address).unwrap();
// TODO: need to handle case in which name is already registered
        let _r = self.generate(13);
        let _name_firstupdate_txid = self.rpc_client.name_firstupdate(&name, &rand, &name_new_txid, Some("hello world"), &name_address).unwrap();
        let _r = self.generate(1);
// confirm name_firstupdate_txid in the chain
//
        println!("registered {}", name);
        Ok(())
    }

    fn set_contract_info(&self, info: PlayerContractInfo, pubkey: PublicKey, sig: Signature) -> Result<(), String> {
        let secp = Secp256k1::new();
        if secp.verify(&Message::from_slice(&info.hash()).unwrap(), &sig, &pubkey.key).is_err() {
            return Err("invalid signature".to_string())
        }

        let mut con = self.redis_client.get_connection().unwrap();
        let _r: RedisResult<String> = con.set(info.name.clone().0, &serde_json::to_string(&info).unwrap());
        Ok(())
    }

    fn get_contract_info(&self, name: PlayerName) -> Option<PlayerContractInfo> {
        let mut con = self.redis_client.get_connection().unwrap();
        let r: RedisResult<String> = con.get(&name.0);
        if let Ok(info) = r {
            Some(serde_json::from_str(&info).unwrap())
        } else {
            None
        }
    }

    fn get_player_name(&self, _pubkey: &PublicKey) -> Option<PlayerName> {
// need to go PublicKey -> Address -> Name
        None
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

async fn register_name_handler(name: String, pubkey: String, sig: String, nmcid: NmcId) -> WebResult<impl Reply>{
    let pubkey = PublicKey::from_slice(&hex::decode(pubkey).unwrap()).unwrap();
    let sig = Signature::from_compact(&hex::decode(sig).unwrap()).unwrap();
    let _r = nmcid.register_name(PlayerName(name.clone()), pubkey, sig).unwrap();
    Ok(name)
}

async fn get_info_handler(player_name: String, nmcid: NmcId) -> WebResult<impl Reply>{
    let info = nmcid.get_contract_info(PlayerName(player_name)).unwrap();
    Ok(serde_json::to_string(&info).unwrap())
}

async fn set_info_handler(contract_info: String, pubkey: String, sig: String, nmcid: NmcId) -> WebResult<impl Reply>{
    let contract_info: PlayerContractInfo = serde_json::from_str(&contract_info).unwrap();
    let pubkey = PublicKey::from_slice(&hex::decode(pubkey).unwrap()).unwrap();
    let sig = Signature::from_compact(&hex::decode(sig).unwrap()).unwrap();
    if nmcid.set_contract_info(contract_info.clone(), pubkey, sig).is_ok() {
        Ok(format!("set contract info for {}", contract_info.name.0))
    } else {
        Err(warp::reject())
    }
}

async fn get_name_handler(pubkey: String, nmcid: NmcId) -> WebResult<impl Reply>{
    let pubkey = PublicKey::from_slice(&hex::decode(pubkey).unwrap()).unwrap();
    if let Some(name) = nmcid.get_player_name(&pubkey) {
        Ok(name.0)
    } else {
        Err(warp::reject())
    }
}

#[tokio::main]
async fn main() {
    let nmc_id = NmcId::new();
    
    let nmc_id = warp::any().map(move || nmc_id.clone());

    let register_name = warp::path("register-name")
        .and(warp::path::param::<String>())
        .and(warp::path::param::<String>())
        .and(warp::path::param::<String>())
        .and(nmc_id.clone())
        .and_then(register_name_handler);

    let set_contract_info = warp::path("set-contract-info")
        .and(warp::path::param::<String>())
        .and(warp::path::param::<String>())
        .and(warp::path::param::<String>())
        .and(nmc_id.clone())
        .and_then(set_info_handler);

    let get_contract_info = warp::path("get-contract-info")
        .and(warp::path::param::<String>())
        .and(nmc_id.clone())
        .and_then(get_info_handler);

    let get_player_name = warp::path("get-player-name")
        .and(warp::path::param::<String>())
        .and(nmc_id.clone())
        .and_then(get_name_handler);

    let routes = register_name
        .or(set_contract_info)
        .or(get_contract_info)
        .or(get_player_name);

    warp::serve(routes).run(([0, 0, 0, 0], 18420)).await;
}

#[cfg(test)]
mod tests {

    use super::*;
    use std::str::FromStr;
    use tglib::{
        hex,
        bdk::bitcoin::{
            util::bip32::DerivationPath,
        },
        bip39::Mnemonic,
        wallet::{
            EscrowWallet,
            SigningWallet,
            BITCOIN_ACCOUNT_PATH,
        },
        mock::{
            Trezor,
            ESCROW_SUBACCOUNT,
            ESCROW_KIX,
            PLAYER_1_MNEMONIC,
        },
    };

    const PUBKEY: &'static str = "02123e6a7816f2149f90cca1ea1ba41b73e77db44cd71f01c184defd10961d03fc";
    const TESTNET_ADDRESS_FROM_NAMECOIND: &'static str = "mfuf8qvMsMJMgBqtEGBt8aCQPQi1qgANzo";
    const TEST_NAME: &'static str = "Arbor";

    #[test]
    fn test_get_namecoin_address() {
        let pubkey = PublicKey::from_slice(&hex::decode(PUBKEY).unwrap()).unwrap();
        let namecoin_address = get_namecoin_address(&pubkey, Network::Testnet).unwrap();
        assert_eq!(namecoin_address,TESTNET_ADDRESS_FROM_NAMECOIND)
    }

    #[test]
    fn test_name_list() {
        let nmc_id = NmcId::new();
        let r = nmc_id.rpc_client.name_list(None);
        for name_status in r.unwrap() {
            println!("{:?} => {:?}", name_status.address, name_status.name);
        } 
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
            DerivationPath::from_str(&format!("m/{}/{}/{}", BITCOIN_ACCOUNT_PATH, ESCROW_SUBACCOUNT, ESCROW_KIX)).unwrap(),
        ).unwrap();

        let nmc_id = NmcId::new();
        let _r = nmc_id.rpc_client.load_wallet("testwallet");
        let _name = nmc_id.register_name(PlayerName(TEST_NAME.to_string()), pubkey, sig);
    }
}
