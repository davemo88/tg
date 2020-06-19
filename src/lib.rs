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

use rand::Rng;

use bitcoin::{
    Transaction,
    Address,
    Script,
    Network,
    Amount,
    util::key::PrivateKey,
    util::key::PublicKey,
    hashes::{
        Hash,
    }
};

use bitcoincore_rpc::{
    Auth,
    Client,
    RpcApi,
    RawTx,
    json::{
        PubKeyOrAddress,
        AddressType,
        CreateRawTransactionInput,
        SignRawTransactionInput,
    },
};

pub const PAYOUT_SCRIPT_MAX_SIZE: usize = 32;
pub const LOCALHOST: &'static str = "0.0.0.0";
pub const TESTNET_RPC_PORT: usize = 18332;

pub struct HostNPort(pub &'static str, pub usize);

pub struct BitcoindRpcConfig {
    pub hostnport: HostNPort,
    pub user: &'static str,
    pub password: &'static str,
}

impl Default for BitcoindRpcConfig {
    fn default() -> Self {
        BitcoindRpcConfig {
            hostnport: HostNPort(LOCALHOST, TESTNET_RPC_PORT),
            user: "user",
            password: "password",
        }
    }
}

pub struct PayoutScript {
    pub body: [u8; PAYOUT_SCRIPT_MAX_SIZE],
}

impl Default for PayoutScript {
    fn default() -> Self {
        PayoutScript {
            body: rand::thread_rng().gen::<[u8;32]>(),
        }
    }
}

pub struct Challenge {
    pub escrow: MultisigEscrow,
// this tx needs to be mined before the payout script can be signed
// because we need to create txs that spend from it in the payout script
// there is no problem with this for the following reasons:
// 1. the ref doesn't need to sign the funding tx since they don't contribute coins
// 2. in a 2/3 multisig, the players can recover their funds at any time, e.g.
// if the challenge doesn't proceed for some reason
//    pub funding_tx_hex: Vec<u8>,
    pub funding_tx_hex: String,
// must include data unique to this challenge e.g. funding tx id, so old signatures for similar
// payouts (e.g. rematches) can't be used, since its hash is signed and use for later verification
// including specific payout transactions in the payout script might be sufficient for uniqueness
// if they are spending from the funding tx utxos
// just put other bitcoin transactions here which uses the multisig's utxo from the challenge
// that are the only ones which can be approved in case of a payout request
// this could just be its hash
    pub payout_script: PayoutScript,
// https://www.sans.org/reading-room/whitepapers/infosec/digital-signature-multiple-signature-cases-purposes-1154
// the parties sign this as well as the funding transaction
// a signed hash of the payout script
// because this contains the funding tx id, it will be unique
//
// signed hash of the payout script 
// this value is unique to this challenge because it includes the funding_tx id
//    pub payout_script_hash_sig: Option<Signature>,
    pub payout_script_hash_sigs: HashMap<PublicKey, Signature>,
}

pub trait ChallengeApi {
    fn payout_script_hash(&self) -> Vec<u8>;
}

impl ChallengeApi for Challenge {
    fn payout_script_hash(&self) -> Vec<u8> {
        bitcoin::hashes::sha256::Hash::from_slice(&self.payout_script.body).unwrap().to_vec()
    }
}

#[derive(Debug)]
pub enum ChallengeState {
// unsigned
    Unsigned,
// signed by one player 
    Issued,
// signed by both players
    Accepted,
// signed by both players and ref in that order
    Certified,
// fucked up somehow
    Invalid,
}

pub struct PayoutRequest {
    pub challenge: Challenge,
    pub payout_tx: Option<Transaction>,
    pub payout_sig: Option<Signature>,
}

pub struct RefereeService;

pub trait RefereeServiceApi {


}

impl RefereeServiceApi for RefereeService {

}

//NOTE: could use dummy tx requiring signing by referee to embed info in tx
impl Challenge {
}

//NOTE: create all possible payout txs beforehand and then branch on something for a basic payout
//script, e.g. in 1v1 winner takes all all to A or B based on some value,
//could require signature from the TO. 
//if you need resolution then somebody has to look it up the value

pub struct MultisigEscrow {
    pub address: Address, 
    pub redeem_script: Script,
    pub players: Vec<PublicKey>,
    pub referees: Vec<PublicKey>,
}
