

pub trait WalletApi {
    fn new_local_player() {}
    fn create_challenge() {}
    fn create_payout_request() {}
    fn sign_challenge() {}
    fn sign_payout_request() {}
    fn broadcast_funding_tx() {}
    fn broadcast_payout_tx() {}
}

#[allow(dead_code)]
struct BitcoinWallet {
    passphrase: String,
}

impl WalletApi for BitcoinWallet {
    fn new_local_player() {}
    fn create_challenge() {}
    fn create_payout_request() {}
    fn sign_challenge() {}
    fn sign_payout_request() {}
    fn broadcast_funding_tx() {}
    fn broadcast_payout_tx() {}
}
