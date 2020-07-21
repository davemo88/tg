use bitcoin::{
    Transaction,
};
use secp256k1::{
    PublicKey,
    Signature,
    Message,
};
use crate::{
    LOCALHOST,
    HostNPort,
    key::{
        PrivateKeyServiceApi,
        PRIVATE_KEY_SERVICE_PORT,
    },
};


pub struct PrivateKeyServiceClient {
    _hostnport: HostNPort, 
}

impl Default for PrivateKeyServiceClient {
    fn default() -> Self {
        PrivateKeyServiceClient {
            _hostnport: HostNPort(LOCALHOST, PRIVATE_KEY_SERVICE_PORT),
        }
    }
}

impl PrivateKeyServiceApi for PrivateKeyServiceClient {

    fn sign_message(&self, _pubkey: PublicKey, _msg: Message) -> Result<Signature, &'static str> {
        Err("error")
    } 

    fn sign_transaction(&self, _pubkey: PublicKey, _tx: Transaction) -> Result<Transaction, &'static str>{
        Err("error")
    }
}
