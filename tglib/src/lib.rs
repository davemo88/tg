use std::{
    collections::HashMap,
    fmt,
};
use bitcoin::{
    Transaction,
    Address,
    util::key::{
        PublicKey,
        PrivateKey,
    },
    hashes::{
        Hash,
        HashEngine,
        sha256::HashEngine as Sha2Engine,
        sha256::Hash as Sha2Hash,
    },
    Script as BitcoinScript,
};

use secp256k1::{
    Secp256k1,
    Message,
    Signature,
};

pub mod script;

#[derive(Debug)]
pub struct TgError(pub &'static str);

impl fmt::Display for TgError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "TgError: {}", self.0)
    }
}

pub type Result<T> = std::result::Result<T, TgError>;

struct TgPlayer(PublicKey);

trait Player {

}

impl Player for TgPlayer {

}

struct Arbiter(PublicKey);

impl Arbiter {

}

#[derive(Clone)]
pub struct Contract {
    pub escrow: MultisigEscrow,
// this tx needs to be mined before the payout script can be signed
// because we need to create txs that spend from it in the payout script
// there is no problem with this for the following reasons:
// 1. the arbiter doesn't need to sign the funding tx since they don't contribute coins
// 2. in a 2/3 multisig, the players can recover their funds at any time, e.g.
// if the contract doesn't proceed for some reason
// above is only the case if don't use segwit, but no reason not to
// segwit makes it possible to create transaction that spend from unmined txs
// because the txid will not change
//    pub funding_tx_hex: Vec<u8>,
//    maybe this should be a Transaction
    pub funding_tx_hex: String,
// must include data unique to this contract e.g. funding tx id, so old signatures for similar
// payouts (e.g. rematches) can't be used, since its hash is signed and use for later verification
// including specific payout transactions in the payout script might be sufficient for uniqueness
// if they are spending from the funding tx utxos
// just put other bitcoin transactions here which uses the multisig's utxo from the contract
// that are the only ones which can be approved in case of a payout request
// this could just be its hash
    pub payout_script: script::TgScript,
// https://www.sans.org/reading-room/whitepapers/infosec/digital-signature-multiple-signature-cases-purposes-1154
// the parties sign this as well as the funding transaction
// a signed hash of the payout script
// because this contains the funding tx id, it will be unique
//
// signed hash of the payout script 
// this value is unique to this contract because it includes the funding txid
//    pub payout_script_hash_sig: Option<Signature>,
// TODO: probably better for serialization to make this a list
// TODO: needs to sign more than just payout script.
// can a signed payout script be paired with the wrong / a fraudulent contract. is any reference
// made to the contract during payout request validation? or simply the script? is that secure?
    pub payout_script_hash_sigs: HashMap<PublicKey, Signature>,

// need to add contract id a la tx id (hash of contract data) and sign that instead of just the
// payout script
}

pub trait ContractApi {
    fn payout_script_hash(&self) -> Vec<u8>;
}

impl ContractApi for Contract {
    fn payout_script_hash(&self) -> Vec<u8> {
        let mut hash_engine = Sha2Engine::default();
        hash_engine.input(&Vec::from(self.payout_script.clone()));
        let payout_script_hash: &[u8] = &Sha2Hash::from_engine(hash_engine);
        payout_script_hash.to_vec()
    }
}

#[derive(Debug, PartialEq)]
pub enum ContractState {
// unsigned
    Unsigned,
// signed by one player 
    Issued,
// signed by both players
    Accepted,
// signed by both players and arbiter
    Certified,
// signed by the arbiter but not both players
    Invalid,
}

//NOTE: could use dummy tx requiring signing by arbiter to embed info in tx
impl Contract {

    #[allow(dead_code)]
    fn sign_payout_script(&mut self, key: PrivateKey) {
// if there is a sig there already, verify it and then add ours on top
        let secp = Secp256k1::new();
        let payout_script_hash = self.payout_script_hash();
        self.payout_script_hash_sigs.insert(key.public_key(&secp),secp.sign(&Message::from_slice(&payout_script_hash).unwrap(), &key.key));
    }

    #[allow(dead_code)]
    fn verify_player_sigs(&self, player: PublicKey,  contract: &Contract) -> bool {
//TODO: check funding tx sig too
        if contract.escrow.players.contains(&player) {
            let secp = Secp256k1::new();        
            return secp.verify(
                &Message::from_slice(&contract.payout_script_hash()).unwrap(),
                &contract.payout_script_hash_sigs[&player],
                &player.key
            ).is_ok()
        }
        else {
            false
        }
    }

    #[allow(dead_code)]
    fn verify_arbiter_sig(&self, arbiter: PublicKey) -> bool {
//TODO: check funding tx sig too
        if self.escrow.arbiters.contains(&arbiter) {
            let secp = Secp256k1::new();        
            return secp.verify(
                &Message::from_slice(&self.payout_script_hash()).unwrap(),
                &self.payout_script_hash_sigs[&arbiter],
                &arbiter.key
            ).is_ok()
        }
        false
    }

    fn state(&self) -> ContractState {

        let mut hash_engine = Sha2Engine::default();
        hash_engine.input(&Vec::from(self.payout_script.clone()));
        let payout_script_hash: &[u8] = &Sha2Hash::from_engine(hash_engine);
        let mut num_player_sigs = 0;
        let secp = Secp256k1::new();
        let msg = Message::from_slice(&payout_script_hash.to_vec()).unwrap();
        for player in &self.escrow.players {
            if self.payout_script_hash_sigs.contains_key(player) && secp.verify(
                &msg,
                &self.payout_script_hash_sigs[player],
                &player.key).is_ok() {
                num_player_sigs += 1;
            }
        }

        let arbiter_sig: bool = self.payout_script_hash_sigs.contains_key(&self.escrow.arbiters[0]) && secp.verify(
            &msg,
            &self.payout_script_hash_sigs[&self.escrow.arbiters[0]],
            &self.escrow.arbiters[0].key).is_ok();

        match arbiter_sig {
            true => {
                match num_player_sigs {
                    NUM_PLAYERS => ContractState::Certified,
                    _ => ContractState::Invalid,
                }
            },
            false => {
                match num_player_sigs {
                    NUM_PLAYERS => ContractState::Accepted,
                    0 => ContractState::Unsigned,
                    _ => ContractState::Issued,
                }
            }
        }
    }
}

#[derive(Clone)]
pub struct PayoutRequest {
    pub contract: Contract,
// in bitcoin, a script is always invaluated in the context of a transaction
// therefore the tx data, e.g. txid doesn't need to be pushed to the stack
// explicitly by the script since it is guaranteed to be available in the context
// that is similar in our case (there will always be a tx) but the tx is only part
// of the PayoutRequest context. 
    pub payout_tx: Transaction,
// in bitcoin, signatures are not necessarily required for scripts to be satisfied
// that is why signatures are given as explicit input to scripts while transactions 
// are not, even though both are used commonly together e.g. in OP_CHECKSIG
// if the way we use sigs is standardized, we could puts sigs in the context too
// e.g. if a signature is required by all payout requests, then it can be reasonably
// stored in the context instead of being pushed onto the stack as arbitrary input
    pub payout_script_sig: Vec<Vec<u8>>,
}

//NOTE: create all possible payout txs beforehand and then branch on something for a basic payout
//script, e.g. in 1v1 winner takes all all to A or B based on some value,
//could require signature from the TO. 
//if you need resolution then somebody has to look it up the value

#[derive(Clone)]
pub struct MultisigEscrow {
    pub address: Address, 
    pub redeem_script: BitcoinScript,
    pub players: Vec<PublicKey>,
    pub arbiters: Vec<PublicKey>,
}

#[cfg(test)]
mod tests {
    #[test]
    fn it_works() {
        assert_eq!(2 + 2, 4);
    }
}
