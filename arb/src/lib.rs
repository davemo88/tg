use std::{
    collections::HashMap,
    fmt,
    convert::TryInto,
};

use secp256k1::{
    Secp256k1,
    Message,
    Signature,
};

use bitcoin::{
    Transaction,
    Address,
    Network,
    blockdata::{
        script::{
            Script as BitcoinScript,
        },
    },
    util::key::{PublicKey, PrivateKey},
    hashes::{
        Hash,
        HashEngine,
        sha256::HashEngine as Sha2Engine,
        sha256::Hash as Sha2Hash,
    },
};

mod key;
mod script;
mod rpc;

use script::{
    TgScript,
};

pub const PAYOUT_SCRIPT_MAX_SIZE: usize = 32;
pub const LOCALHOST: &'static str = "0.0.0.0";
pub const TESTNET_RPC_PORT: usize = 18332;
pub const NETWORK: Network = Network::Regtest;
pub const MINER_FEE: u64 = 10000;
pub const NUM_PLAYERS: u64 = 2;

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

struct TgPlayer(PublicKey);

trait Player {

}

impl Player for TgPlayer {

}

struct Arbiter(PublicKey);

impl Arbiter {

}

#[derive(Clone)]
pub struct Challenge {
    pub escrow: MultisigEscrow,
// this tx needs to be mined before the payout script can be signed
// because we need to create txs that spend from it in the payout script
// there is no problem with this for the following reasons:
// 1. the arbiter doesn't need to sign the funding tx since they don't contribute coins
// 2. in a 2/3 multisig, the players can recover their funds at any time, e.g.
// if the challenge doesn't proceed for some reason
// above is only the case if don't use segwit, but no reason not to
// segwit makes it possible to create transaction that spend from unmined txs
// because the txid will not change
//    pub funding_tx_hex: Vec<u8>,
    pub funding_tx_hex: String,
// must include data unique to this challenge e.g. funding tx id, so old signatures for similar
// payouts (e.g. rematches) can't be used, since its hash is signed and use for later verification
// including specific payout transactions in the payout script might be sufficient for uniqueness
// if they are spending from the funding tx utxos
// just put other bitcoin transactions here which uses the multisig's utxo from the challenge
// that are the only ones which can be approved in case of a payout request
// this could just be its hash
    pub payout_script: script::TgScript,
// https://www.sans.org/reading-room/whitepapers/infosec/digital-signature-multiple-signature-cases-purposes-1154
// the parties sign this as well as the funding transaction
// a signed hash of the payout script
// because this contains the funding tx id, it will be unique
//
// signed hash of the payout script 
// this value is unique to this challenge because it includes the funding txid
//    pub payout_script_hash_sig: Option<Signature>,
// TODO: probably better for serialization to make this a list
    pub payout_script_hash_sigs: HashMap<PublicKey, Signature>,
}

pub trait ChallengeApi {
    fn payout_script_hash(&self) -> Vec<u8>;
}

impl ChallengeApi for Challenge {
    fn payout_script_hash(&self) -> Vec<u8> {
        let mut hash_engine = Sha2Engine::default();
        hash_engine.input(&Vec::from(self.payout_script.clone()));
        let payout_script_hash: &[u8] = &Sha2Hash::from_engine(hash_engine);
        payout_script_hash.to_vec()
    }
}

#[derive(Debug, PartialEq)]
pub enum ChallengeState {
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
impl Challenge {

    #[allow(dead_code)]
    fn sign_payout_script(&mut self, key: PrivateKey) {
// if there is a sig there already, verify it and then add ours on top
        let secp = Secp256k1::new();
        let payout_script_hash = self.payout_script_hash();
        self.payout_script_hash_sigs.insert(key.public_key(&secp),secp.sign(&Message::from_slice(&payout_script_hash).unwrap(), &key.key));
    }

    #[allow(dead_code)]
    fn verify_player_sigs(&self, player: PublicKey,  challenge: &Challenge) -> bool {
//TODO: check funding tx sig too
        if challenge.escrow.players.contains(&player) {
            let secp = Secp256k1::new();        
            return secp.verify(
                &Message::from_slice(&challenge.payout_script_hash()).unwrap(),
                &challenge.payout_script_hash_sigs[&player],
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

    fn state(&self) -> ChallengeState {

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
                    NUM_PLAYERS => ChallengeState::Certified,
                    _ => ChallengeState::Invalid,
                }
            },
            false => {
                match num_player_sigs {
                    NUM_PLAYERS => ChallengeState::Accepted,
                    0 => ChallengeState::Unsigned,
                    _ => ChallengeState::Issued,
                }
            }
        }
    }
}

#[derive(Clone)]
pub struct PayoutRequest {
    pub challenge: Challenge,
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

pub fn create_tx_fork_script(pubkey: &PublicKey, tx1: &Transaction, tx2: &Transaction) -> TgScript {
// this script is for the following case
// player1 and player2 want to play a 1v1 winner-take-all match with no possibility for a draw
// this means there are only 2 possible payouts: everything to p1 or everything p2
// there is also some 4th party (pubkey) which can certify the results of the match
// we create unsigned transactions for both these cases (tx1 and tx2)
// e.g. if p1 wins, the 4th party can certify that, i.e. sign a token 
// in this case it will sign the txid of the correct payout transaction
// if the players won't cooperate to release the funds later, the winner acquires the signed token
// and includes it in the payout request
// the script itself checks if the signed token is valid and uses it to determine which player
// won and therefore which transaction should be signed. the player also submits the transaction
// which the escrow service should sign to release the funds
//
// the script takes input (signature, pubkey):
//
// if (verify(signature, pubkey, p1_wins_msg) && payout_tx == tx1)
// || (verify(signature, pubkey, p2_wins_msg) && payout_tx == tx2):
//   valid
// else:
//   invalid
//
// if the script is valid, the key service will sign the payout_tx
// the signature is the signed token certifying the match result
    use script::TgOpcode::*;

    let txid1: &[u8] = &tx1.txid();
    let txid2: &[u8] = &tx2.txid();
// TODO should be a pubkeyhash instead of full pubkey, same reasons as bitcoin addresses
// that requires the pubkey to also be given as input as in standard pay to pubkey hash
    let pubkey_bytes = pubkey.to_bytes();

    TgScript(vec![         
        OP_PUSHDATA1(pubkey_bytes.len().try_into().unwrap(), pubkey_bytes.clone()),
        OP_2DUP,
        OP_PUSHDATA1(txid1.len().try_into().unwrap(), Vec::from(txid1)),
        OP_VERIFYSIG,
        OP_IF(
            TgScript(vec![
                OP_1,
            ]),
            Some(TgScript(vec![
                OP_PUSHDATA1(txid2.len().try_into().unwrap(), Vec::from(txid2)),
                OP_VERIFYSIG,
            ]))
        ),
        OP_VALIDATE,
    ])
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        script::TgScriptEnv,
        rpc::{
            self,
            TgRpcClientApi,
            TgRpcClient,
        },
    };
    use bitcoin::{
        Amount,
        hashes::hex::{FromHex, ToHex},
        consensus::encode,
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
        },
    };

    const PLAYER_1_ADDRESS_LABEL: &'static str = "Player1";
    const PLAYER_2_ADDRESS_LABEL: &'static str = "Player2";
    const REFEREE_ADDRESS_LABEL: &'static str = "Referee";
    const POT_AMOUNT: Amount =  Amount::ONE_BTC;

    fn fund_players(client: &rpc::TgRpcClient, p1_address: &Address, p2_address: &Address) {

        let faucet = client.0.get_new_address(Some("faucet"), Some(AddressType::Legacy)).unwrap();
//        println!("faucet address: {:?}", faucet);

        let _result = client.0.generate_to_address(302, &faucet).unwrap();

        let faucet_unspent = client.0.list_unspent(None,None,Some(&[&faucet]),None,None).unwrap();

        let mut tx_inputs: Vec<CreateRawTransactionInput> = Vec::new();
        let mut total_in_amount = Amount::ZERO; 
        let target_in_amount = Amount::ONE_BTC * 2;

        for utxo in faucet_unspent {
            if utxo.spendable {
                tx_inputs.push(
                    CreateRawTransactionInput {
                        txid: utxo.txid,
                        vout: utxo.vout,
                        sequence: None,
                    }
                );

                total_in_amount += utxo.amount;
                if total_in_amount >= target_in_amount {
                    break
                }
            }
//            println!("{:?} {:?}", total_in_amount, target_in_amount);

        }

//        println!("added {:?} transactions totalling {:?}", tx_inputs.len(), total_in_amount); 

        let mut outs = HashMap::<String, Amount>::default();
        outs.insert(p1_address.to_string(), Amount::ONE_BTC);
        outs.insert(p2_address.to_string(), Amount::ONE_BTC);
        outs.insert(faucet.to_string(), total_in_amount - target_in_amount - Amount::from_sat(MINER_FEE));

        let tx = client.0.create_raw_transaction(
            &tx_inputs,
            &outs,
            None,
            None,
        ).unwrap();

//        println!("tx: {:?}", tx);

        let priv_key = client.0.dump_private_key(&faucet).unwrap();

        let sign_result = client.0.sign_raw_transaction_with_key(
            &tx,
            &[priv_key],
            None,
            None,
        ).unwrap();

//        println!("sign result: {:?}", sign_result);

        let send_result = client.0.send_raw_transaction(&sign_result.hex).unwrap();
//
//        println!("{:?}", send_result);

        let _result = client.0.generate_to_address(110, &faucet).unwrap();

        let _tx_info = client.0.get_raw_transaction_info(&send_result,None);

//        println!("{:?}", _tx_info);

    }

    fn create_2_of_3_multisig(client: &rpc::TgRpcClient, p1_pubkey: PublicKey, p2_pubkey: PublicKey, arbiter_pubkey: PublicKey) -> MultisigEscrow {

        let result = client.0.add_multisig_address(
            2,
            &[
                PubKeyOrAddress::PubKey(&p1_pubkey),
                PubKeyOrAddress::PubKey(&p2_pubkey),
                PubKeyOrAddress::PubKey(&arbiter_pubkey)
            ],
            None,
            None,
        ).unwrap();

        MultisigEscrow {
            address: result.address,
            redeem_script: result.redeem_script,
            players: vec!(p1_pubkey.clone(),p2_pubkey.clone()),
            arbiters: vec!(arbiter_pubkey.clone()),
        }
    }

    fn create_challenge(client: &TgRpcClient) -> Challenge {

        let p1_address = client.0.get_new_address(Some(PLAYER_1_ADDRESS_LABEL), Some(AddressType::Bech32)).unwrap();
        let p2_address = client.0.get_new_address(Some(PLAYER_2_ADDRESS_LABEL), Some(AddressType::Bech32)).unwrap();
        let arbiter_address = client.0.get_new_address(Some(REFEREE_ADDRESS_LABEL), Some(AddressType::Bech32)).unwrap();
//
        println!("fund players 1BTC each");

        fund_players(&client, &p1_address, &p2_address);

        let p1_balance = client.0.get_received_by_address(&p1_address, None).unwrap();
        let p2_balance = client.0.get_received_by_address(&p2_address, None).unwrap();
        println!("{:?} balance: {:?}", p1_address.to_string(), p1_balance);
        println!("{:?} balance: {:?}", p2_address.to_string(), p2_balance);

        println!("creating challenge:

players:
{:?} 
{:?} 

arbiter:
{:?}

pot: {:?}
arbiter fee: {:?}
miner fee: {:?}
buyin: {:?}
",
            p1_address, 
            p2_address, 
            arbiter_address, 
            POT_AMOUNT,
            POT_AMOUNT/100,
            Amount::from_sat(MINER_FEE),
            (POT_AMOUNT + (POT_AMOUNT/100) + Amount::from_sat(MINER_FEE)) / 2,
        );

        let p1_pubkey = client.0.get_address_info(&p1_address).unwrap().pubkey.unwrap();
        let p2_pubkey = client.0.get_address_info(&p2_address).unwrap().pubkey.unwrap();
        let arbiter_pubkey = client.0.get_address_info(&arbiter_address).unwrap().pubkey.unwrap();

        let escrow = create_2_of_3_multisig(&client,
            p1_pubkey,
            p2_pubkey,
            arbiter_pubkey,
        );
        println!("escrow {:?} created", escrow.address.to_string());

        let funding_tx = client.create_challenge_funding_transaction(&escrow, POT_AMOUNT).unwrap();
        println!("funding tx {:?} created", funding_tx.txid());

        let payout_tx_p1 = client.create_challenge_payout_transaction(&escrow, &funding_tx, &p1_pubkey).unwrap();
        println!("payout tx for_p1 {:?} created", payout_tx_p1.txid());
        let payout_tx_p2 = client.create_challenge_payout_transaction(&escrow, &funding_tx, &p2_pubkey).unwrap();
        println!("payout tx for p2 {:?} created", payout_tx_p2.txid());

        let payout_script = create_tx_fork_script(&arbiter_pubkey, &payout_tx_p1, &payout_tx_p2);
        let mut hash_engine = Sha2Engine::default();
        hash_engine.input(&Vec::from(payout_script.clone()));
        let payout_script_hash: &[u8] = &Sha2Hash::from_engine(hash_engine);
        println!("payout script {:?} created", payout_script_hash.to_hex());

        Challenge {
            escrow,
            funding_tx_hex: funding_tx.raw_hex(),
            payout_script,
            payout_script_hash_sigs: HashMap::<PublicKey, Signature>::default(),
        }
    }

    fn sign_challenge(client: &TgRpcClient, mut challenge: &mut Challenge) {

        let p1_address = Address::p2wpkh(&challenge.escrow.players[0], NETWORK);   
        let p2_address = Address::p2wpkh(&challenge.escrow.players[1], NETWORK);   
        let arbiter_address = Address::p2wpkh(&challenge.escrow.arbiters[0], NETWORK);   

        println!("challenge state: {:?}", challenge.state());
        println!("p1 signing");
        let p1_key = client.0.dump_private_key(&p1_address).unwrap();
        client.sign_challenge_tx(p1_key, &mut challenge);
        sign_challenge_payout_script(p1_key, &mut challenge);
        println!("challenge state: {:?}", challenge.state());

        println!("p2 signing");
        let p2_key = client.0.dump_private_key(&p2_address).unwrap();
        client.sign_challenge_tx(p2_key, &mut challenge);
        sign_challenge_payout_script(p2_key, &mut challenge);
        println!("challenge state: {:?}", challenge.state());

        println!("arbiter signing");
        let arbiter_key = client.0.dump_private_key(&arbiter_address).unwrap();
        sign_challenge_payout_script(arbiter_key, &mut challenge);
        println!("challenge state: {:?}", challenge.state());
    }

    fn sign_challenge_payout_script(key: PrivateKey, challenge: &mut Challenge) {
// if it were sequential dependent then different protocol:
// if there is a sig there already, verify it and then add ours on top
// but here it's sequential and independent, we add each sig by itself
        let secp = Secp256k1::new();
        let payout_script_hash = challenge.payout_script_hash();
        challenge.payout_script_hash_sigs.insert(key.public_key(&secp),secp.sign(&Message::from_slice(&payout_script_hash).unwrap(), &key.key));
    }

    fn broadcast_challenge_tx(client: &TgRpcClient, challenge: &Challenge) {

        println!("broadcasting signed challenge funding transaction");
        let arbiter_address = Address::p2wpkh(&challenge.escrow.arbiters[0], NETWORK);   

        let _result = client.0.send_raw_transaction(challenge.funding_tx_hex.clone());

        let _address = client.0.get_new_address(None, Some(AddressType::Legacy)).unwrap();
        let _result = client.0.generate_to_address(10, &_address);

        let arbiter_balance = client.0.get_received_by_address(&arbiter_address, None).unwrap();
        println!("arbiter {:?} balance: {:?}", arbiter_address.to_string(), arbiter_balance);
        let arbiter_balance = client.0.get_received_by_address(&challenge.escrow.address, None).unwrap();
        println!("multsig {:?} balance: {:?}", &challenge.escrow.address.to_string(), arbiter_balance);

    }
    
    fn create_signed_payout_request(client: &TgRpcClient, challenge: &Challenge, player: &PublicKey, arbiter: &PublicKey) -> PayoutRequest {
        let funding_tx: bitcoin::Transaction = encode::deserialize(&Vec::<u8>::from_hex(&challenge.funding_tx_hex).unwrap()).unwrap();
        let payout_tx = client.create_challenge_payout_transaction(&challenge.escrow, &funding_tx, &player).unwrap();
        let key = client.0.dump_private_key(&Address::p2wpkh(&player, NETWORK)).unwrap();
        let payout_tx = client.0.sign_raw_transaction_with_key(
            payout_tx.raw_hex(),
            &[key],
            None,
            None,
        ).unwrap().hex.raw_hex();
        let payout_tx: bitcoin::Transaction = encode::deserialize(&Vec::<u8>::from_hex(&payout_tx).unwrap()).unwrap();

        let arbiter_key = client.0.dump_private_key(&Address::p2wpkh(&arbiter, NETWORK)).unwrap();
        let msg = Message::from_slice(&payout_tx.txid()).unwrap();
        let secp = Secp256k1::new();
        let sig = secp.sign(&msg, &arbiter_key.key);
        let sig: Vec<u8> = sig.serialize_der().to_vec();
        let payout_script_sig: Vec<Vec<u8>> = vec!(sig); 

        PayoutRequest {
            challenge: challenge.clone(),
            payout_tx,
            payout_script_sig,
        }
    }

    fn validate_payout_request(payout_request: PayoutRequest) -> Result<()> {

        let mut env = TgScriptEnv::new(payout_request);

        env.validate_payout_request()
    }

    #[test]
    fn challenge_with_tx_fork_script() {

        let bitcoind_rpc_config = BitcoindRpcConfig::default();

        let client = rpc::TgRpcClient(Client::new(
            format!("http://{}:{:?}",
                bitcoind_rpc_config.hostnport.0,
                bitcoind_rpc_config.hostnport.1,
            ),
            Auth::UserPass(
            bitcoind_rpc_config.user.to_string(),
                bitcoind_rpc_config.password.to_string(),
            )
        ).unwrap());
        
        let mut challenge = create_challenge(&client);
        sign_challenge(&client, &mut challenge);
        broadcast_challenge_tx(&client, &challenge);

        println!("\np1 payout request");
        let payout_request_p1 = create_signed_payout_request(&client, &challenge, &challenge.escrow.players[0], &challenge.escrow.arbiters[0]);
        let validation_result = validate_payout_request(payout_request_p1);
        assert!(validation_result.is_ok());

        println!("\np2 payout request");
        let payout_request_p2 = create_signed_payout_request(&client, &challenge, &challenge.escrow.players[1], &challenge.escrow.arbiters[0]);
        let validation_result = validate_payout_request(payout_request_p2);
        assert!(validation_result.is_ok());

// TODO add invalid payout requests for both players

    }
}

