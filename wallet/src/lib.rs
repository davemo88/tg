

pub trait WalletApi {
    fn createChallenge() {}
    fn createPayoutRequest() {}
    fn signChallenge() {}
    fn signPayoutRequest() {}
    fn broadcastFundingTx() {}
    fn broadcastPayoutTx() {}
}

struct BitcoinWallet {
    passphrase: String; 
}

impl WalletApi for BitcoinWallet {
    fn newLocalPlayer() {}
    fn createChallenge() {}
    fn createPayoutRequest() {}
    fn signChallenge() {}
    fn signPayoutRequest() {}
    fn broadcastFundingTx() {}
    fn broadcastPayoutTx() {}
}
