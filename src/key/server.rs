use secp256k1::{
    Secp256k1,
    Message,
    Signature,
    PublicKey,
};
use bitcoin::{
    Transaction,
    util::key::PrivateKey,
};

use bitcoincore_rpc::{
    Auth,
    Client,
    RpcApi,
};

use crate::{
    LOCALHOST,
    HostNPort,
    BitcoindRpcConfig,
    key::PRIVATE_KEY_SERVICE_PORT,
};

pub trait PrivateKeyServiceApi {
// plain old signature using secp
    fn sign_message(&self, pubkey: PublicKey, msg: Message) -> Result<Signature, &'static str>;
// sign a transaction using bitcoind
    fn sign_transaction(&self, pubkey: PublicKey, tx: Transaction) -> Result<Transaction, &'static str>;
}

pub struct PrivateKeyService {
    pub hostnport: HostNPort,
    pub bitcoind_rpc_config: BitcoindRpcConfig,
}

impl Default for PrivateKeyService {
    fn default() -> Self {
        PrivateKeyService {
            hostnport: HostNPort(LOCALHOST, PRIVATE_KEY_SERVICE_PORT),
            bitcoind_rpc_config: BitcoindRpcConfig::default(),
        }
    }
}

impl PrivateKeyService {

    fn get_private_key(&self, _pubkey: &PublicKey) -> Option<PrivateKey> {
        None
    }
}

impl PrivateKeyServiceApi for PrivateKeyService {

    fn sign_message(&self, pubkey: PublicKey, msg: Message) -> Result<Signature, &'static str> {
        let private_key = self.get_private_key(&pubkey).expect(&format!("no corresponding private key for pubkey {:?}",pubkey));
        let secp = Secp256k1::new(); 
        Ok(secp.sign(&msg, &private_key.key))
    } 

    fn sign_transaction(&self, pubkey: PublicKey, tx: Transaction) -> Result<Transaction, &'static str>{
        let private_key = self.get_private_key(&pubkey).expect(&format!("no corresponding private key for pubkey {:?}",pubkey));
        let rpc = Client::new(
            format!("http://{:?}:{:?}",
                self.bitcoind_rpc_config.hostnport.0,
                self.bitcoind_rpc_config.hostnport.1,
            ),
            Auth::UserPass(
                self.bitcoind_rpc_config.user.to_string(),
                self.bitcoind_rpc_config.password.to_string(),
            )
        ).unwrap();

        let sign_result = rpc.sign_raw_transaction_with_key(&tx, &[private_key],None,None).unwrap();

        Ok(sign_result.transaction().unwrap())
    }
}
