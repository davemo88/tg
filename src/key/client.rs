use crate::{
    LOCALHOST,
    HostNPort,
    key::{
//        PrivateKeyServiceApi,
        PRIVATE_KEY_SERVICE_PORT,
    },
};


pub struct PrivateKeyServiceClient {
    hostnport: HostNPort, 
}

impl Default for PrivateKeyServiceClient {
    fn default() -> Self {
        PrivateKeyServiceClient {
            hostnport: HostNPort(LOCALHOST, PRIVATE_KEY_SERVICE_PORT),
        }
    }
}

//impl PrivateKeyServiceApi for PrivateKeyServiceClient {
//
//    fn sign_message(&self, pubkey: PublicKey, msg: Message) -> Result<Signature, &'static str> {
//        self.pk_service.sign_message(pubkey, msg)
//    } 
//
//    fn sign_transaction(&self, pubkey: PublicKey, tx: Transaction) -> Result<Transaction, &'static str>{
//        self.pk_service.sign_transaction(pubkey, tx)
//    }
//}
