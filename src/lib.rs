use std::{
    collections::HashMap,
    fmt,
};
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
    blockdata::{
        script::{
            Builder,
            Script,
        },
        opcodes::{
            all,
        }
    },
    Network,
    Amount,
    util::key::PrivateKey,
    util::key::PublicKey,
    hashes::{
        Hash,
    },
    consensus::{
        encode,
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

#[derive(Debug)]
pub struct TgError(pub &'static str);

impl fmt::Display for TgError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "TgError: {}", self.0)
    }
}

pub type Result<T> = std::result::Result<T, TgError>;

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
    pub script: Script,
}

impl Default for PayoutScript {
    fn default() -> Self {
        PayoutScript {
            body: rand::thread_rng().gen::<[u8;32]>(),
            script: Script::default(),
        }
    }
}

pub struct PayoutScriptBuilder;

impl PayoutScriptBuilderApi for PayoutScriptBuilder {}

pub trait PayoutScriptBuilderApi {
    fn build_sig_plus_txid_script() -> Script {
// https://bitcoin-script-debugger.visvirial.com/?input=OP_PUSHDATA1%201%200x07%20OP_PUSHDATA1%201%200x04%20OP_DUP%20OP_PUSHDATA1%201%200x04%20OP_EQUAL%20OP_IF%20OP_DROP%20OP_PUSHDATA1%201%200x07%20OP_EQUAL%20OP_ELSE%20OP_PUSHDATA1%201%200x05%20OP_EQUAL%20OP_IF%20OP_PUSHDATA1%201%200x06%20OP_EQUAL
/*
OP_PUSHDATA1 1 0x07
OP_PUSHDATA1 1 0x04
OP_DUP
OP_PUSHDATA1 1 0x04
OP_EQUAL
OP_IF 
OP_DROP OP_PUSHDATA1 1 0x07 OP_EQUAL
OP_ELSE
OP_PUSHDATA1 1 0x05 OP_EQUAL
OP_IF 
OP_PUSHDATA1 1 0x06 OP_EQUAL
*/

        let request_outcome_token: Vec<u8> = vec![0x4];
        let request_txid: Vec<u8> = vec![0x4];

        let script_outcome_token_1: Vec<u8> = vec![0x3];
        let script_txid_1: Vec<u8> = vec![0x3];

        let script_outcome_token_2: Vec<u8> = vec![0x4];
        let script_txid_2: Vec<u8> = vec![0x4];

        let b = Builder::new(); 

        let b = b.push_slice(&request_txid);

        println!("{}", b.clone().into_script().asm());

        let b = b.push_slice(&request_outcome_token);

        let b = b.push_opcode(all::OP_DUP);

        let b = b.push_slice(&script_outcome_token_1);
//
        let b = b.push_opcode(all::OP_EQUAL);
//
        let b = b.push_opcode(all::OP_IF);
//
        let b = b.push_opcode(all::OP_DROP);

        let b = b.push_slice(&script_txid_1);
//
        let b = b.push_opcode(all::OP_ELSE);
//
        let b = b.push_slice(&script_txid_2);
//
        let b = b.push_opcode(all::OP_EQUAL);
//
        let b = b.push_opcode(all::OP_IF);
//
        let b = b.push_slice(&script_txid_2);
//
        let b = b.push_opcode(all::OP_EQUAL);

        println!("{}", b.clone().into_script().asm());

        b.into_script()
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

#[cfg(test)]
mod tests {

    use super::*;

//    #[test]
//    fn test_payout_script_builder() {
//        let script = PayoutScriptBuilder::build_sig_plus_txid_script();
//        println!("\nprinting payout script instructions");
//        for i in script.iter(false) {
//            println!("{:?}",i);
//        }
//                
//    }
}
