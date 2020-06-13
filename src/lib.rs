use std::collections::HashMap;

use secp256k1::{
    Secp256k1,
    Message,
    Signature,
    SecretKey,
    rand::{
        rngs::OsRng,
        RngCore,
    },
};
use bitcoin::{
    Transaction,
    Address,
    Script,
    Network,
    Amount,
    util::key::PrivateKey,
    util::key::PublicKey,
};

use bitcoincore_rpc::{
    Auth,
    Client,
    RpcApi,
    RawTx,
    json::PubKeyOrAddress,
};

pub const PAYOUT_SCRIPT_MAX_SIZE: usize = 100;
pub const TERMS_SIZE: usize = 100;
pub const LOCALHOST: &'static str = "localhost";
pub const TESTNET_RPC_PORT: usize = 18332;

pub struct HostNPort(pub &'static str, pub usize);

pub struct BitcoindRpcConfig {
    pub hostnport: HostNPort,
    pub username: &'static str,
    pub password: &'static str,
}

impl Default for BitcoindRpcConfig {
    fn default() -> Self {
        BitcoindRpcConfig {
            hostnport: HostNPort(LOCALHOST, TESTNET_RPC_PORT),
            username: "username",
            password: "password",
        }
    }
}

pub type Terms = [u8; PAYOUT_SCRIPT_MAX_SIZE];

pub struct Challenge {
    escrow: MultisigEscrow,
    funding_tx: Transaction,
    terms: Terms,
    terms_sig: Option<Signature>,
}

pub struct RefereeService;

pub trait RefereeServiceApi {



}

impl RefereeServiceApi for RefereeService {

}

pub struct RefereeKey(PublicKey);

//NOTE: could use dummy tx requiring signing by referee to embed info in tx
impl Challenge {
    pub fn new(id: u64, escrow: MultisigEscrow, funding_tx: Transaction, terms: Terms) -> Self {
        Challenge {
            escrow: escrow,
            funding_tx: funding_tx,
            terms: terms,
// https://www.sans.org/reading-room/whitepapers/infosec/digital-signature-multiple-signature-cases-purposes-1154
// page 5 dependent sequential multiple signatures
// signatures are just encryption of the message hash with the private key
// so can be decoded by public key
// e.g. challenge creator signs terms first, then recipient, then referee
            terms_sig: None,
        }
    }
}

//NOTE: create all possible payout txs beforehand and then branch on something for a basic payout
//script, e.g. in 1v1 winner takes all all to A or B based on some value,
//could require signature from the TO. 
//if you need resolution then somebody has to look it up the value

pub struct MultisigEscrow {
    address: Address, 
    redeem_script: Script,
    players: Vec<PublicKey>,
    referees: Vec<PublicKey>,
}

impl MultisigEscrow {
    pub fn new(address: Address, redeem_script: Script, players: Vec<PublicKey>, referees: Vec<PublicKey>, signatures_required: u8,) -> Self {
        MultisigEscrow {
            address: address,
            redeem_script: redeem_script,
            players: players,
            referees: referees,
        }
    }

    pub fn is_signed_by_players(&self, transaction: Transaction) -> bool {
        false
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn create_challenge() {

        println!("create challenge test");

        let secp = Secp256k1::new();

        let mut rng = OsRng::new().unwrap();

        let (a_secretkey, _) = secp.generate_keypair(&mut rng);
        let (b_secretkey, _) = secp.generate_keypair(&mut rng);
        let (r_secretkey, _) = secp.generate_keypair(&mut rng);

        let a_privkey = PrivateKey {
            compressed: true,
            network: Network::Testnet,
            key: a_secretkey,
        };

        let b_privkey = PrivateKey {
            compressed: true,
            network: Network::Testnet,
            key: b_secretkey,
        };

        let r_privkey = PrivateKey {
            compressed: true,
            network: Network::Testnet,
            key: r_secretkey,
        };

        let a_pubkey = &a_privkey.public_key(&secp);
        let b_pubkey = &b_privkey.public_key(&secp);
        let r_pubkey = &r_privkey.public_key(&secp);

        let bitcoind_rpc_config = BitcoindRpcConfig::default();

        let rpc = Client::new(
            format!("http://{:?}:{:?}",
                bitcoind_rpc_config.hostnport.0,
                bitcoind_rpc_config.hostnport.1,
            ),
            Auth::UserPass(
                bitcoind_rpc_config.username.to_string(),
                bitcoind_rpc_config.password.to_string(),
            )
        ).unwrap();

        let result = rpc.add_multisig_address(
            2,
            &[
                PubKeyOrAddress::PubKey(&a_pubkey),
                PubKeyOrAddress::PubKey(&b_pubkey),
                PubKeyOrAddress::PubKey(&r_pubkey)
            ],
            None,
            None,
        ).unwrap();

        let escrow = MultisigEscrow {
            address: result.address,
            redeem_script: result.redeem_script,
            players: vec!(a_pubkey.clone(), b_pubkey.clone()),
            referees: vec!(r_pubkey.clone()),
        };

        let funding_tx = rpc.create_raw_transaction(
            &[],
            &HashMap::<String, Amount>::default(),
            None,
            None,
        ).unwrap();

        let challenge = Challenge {
            escrow: escrow,
            funding_tx: funding_tx,
            terms: [1; TERMS_SIZE],
            terms_sig: None,
        };

    }
}
