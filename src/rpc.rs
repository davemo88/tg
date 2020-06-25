use std::collections::HashMap;
use secp256k1::{
    Secp256k1,
    Message,
    Signature,
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
    hashes::{
        Hash,
        hex::FromHex,
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
        GetRawTransactionResultVout,
        GetRawTransactionResultVoutScriptPubKey,
    },
};

use crate::lib::{
    MultisigEscrow,
    BitcoindRpcConfig,
    Challenge,
    ChallengeApi,
    ChallengeState,
    PayoutScript,
    PayoutRequest,
    Result,
    TgError,
};

use crate::script::TgScript;

pub const NETWORK: Network = Network::Regtest;
pub const MINER_FEE: u64 = 10000;
pub const NUM_PLAYERS: u64 = 2;

pub struct TgClient(Client);

trait TgClientApi {
    fn create_challenge_funding_transaction(&self, escrow: &MultisigEscrow, pot: Amount) -> Result<Transaction>;
    fn create_challenge_payout_transaction(&self, payout_request: PayoutRequest) -> Result<Transaction>;
    fn verify_payout_request(&self, payout_request: PayoutRequest) -> bool;
    fn get_inputs_for_address(&self, address: &Address, amount: Amount) -> (Vec<CreateRawTransactionInput>, Amount,);
    fn sign_challenge_tx(&self, key: PrivateKey, challenge: &mut Challenge);
    fn sign_challenge_payout_script(&self, key: PrivateKey, challenge: &mut Challenge);
    fn get_challenge_state(&self, challenge: &Challenge) -> ChallengeState;
    fn verify_player_sigs(&self, player: PublicKey, challenge: &Challenge) -> bool;
    fn verify_referee_sig(&self, referee: PublicKey, challenge: &Challenge) -> bool;
}

impl TgClientApi for TgClient {
    fn create_challenge_funding_transaction(&self, escrow: &MultisigEscrow, pot: Amount) -> Result<Transaction> {
        let p1_address = Address::p2wpkh(&escrow.players[0], NETWORK);
        let p2_address = Address::p2wpkh(&escrow.players[1], NETWORK);
        let ref_address = Address::p2wpkh(&escrow.referees[0], NETWORK);

//        let pot = Amount::ONE_SAT * 2000000;

        let ref_fee = pot / 100;
        let buyin = (pot + ref_fee + Amount::from_sat(MINER_FEE)) / 2;

        let (p1_inputs, p1_total_in,) = self.get_inputs_for_address(&p1_address, buyin);
        let (p2_inputs, p2_total_in,) = self.get_inputs_for_address(&p2_address, buyin);

        let p1_change = p1_total_in - buyin;
        let p2_change = p2_total_in - buyin;

        assert!(!p1_inputs.is_empty() && !p2_inputs.is_empty());

        let mut tx_inputs = Vec::<CreateRawTransactionInput>::new();
        tx_inputs.extend(p1_inputs);
        tx_inputs.extend(p2_inputs);

        let mut outs = HashMap::<String, Amount>::default();
        outs.insert(escrow.address.to_string(), pot);
        outs.insert(ref_address.to_string(), ref_fee);
        outs.insert(p1_address.to_string(), p1_change);
        outs.insert(p2_address.to_string(), p2_change);

        let tx = self.0.create_raw_transaction(
            &tx_inputs,
            &outs,
            None,
            None,
        ).unwrap();

        let total_in = p1_total_in + p2_total_in;
        let total_out: Amount = Amount::ONE_SAT * tx.output.iter().map(|txout| txout.value).sum();

//        println!("
//challenge funding tx:
//        p1 total in: {:?}
//        p2 total in: {:?}
//        total in: {:?}
//        total out: {:?}
//        pot: {:?}
//        ref fee: {:?}
//        miner fee: {:?}
//        p1 change: {:?}
//        p2 change: {:?} 
//        total in minus change and fees: {:?}",
//            p1_total_in,
//            p2_total_in,
//            total_in,
//            total_out,
//            pot, 
//            ref_fee,
//            MINER_FEE, 
//            p1_change, 
//            p2_change,
//            total_in - p1_change - p2_change - ref_fee - Amount::from_sat(MINER_FEE),
//        );
        assert!(total_in - p1_change - p2_change - ref_fee - Amount::from_sat(MINER_FEE) == pot);
        Ok(tx)
    }

    fn verify_payout_request(&self, payout_request: PayoutRequest) -> bool {
//NOTE: maybe just store this and the txid on challenge instead of full challenge and the full
//payout script
        let payout_scripthash_address = Address::from_script(&payout_request.challenge.payout_script.script, NETWORK).unwrap();
        let mut outs = HashMap::<String, Amount>::default();
        outs.insert(payout_scripthash_address.to_string(),Amount::ONE_BTC);
        let tx = self.0.create_raw_transaction(
            &[],
            &outs,
            None,
            None,
        ).unwrap();

        println!("dummy {:?}", tx);

        false
    }

    fn create_challenge_payout_transaction(&self, payout_request: PayoutRequest) -> Result<Transaction> {
        let challenge = &payout_request.challenge;
        let escrow = &challenge.escrow;
//        println!("{:?}", &challenge.funding_tx_hex);
        let funding_tx: bitcoin::Transaction = encode::deserialize(&Vec::<u8>::from_hex(&challenge.funding_tx_hex).unwrap()).unwrap();

        let tx_info = self.0.get_raw_transaction_info(&funding_tx.txid(), None).unwrap();

        let mut vout: Option<&GetRawTransactionResultVout> = None;
        for (i, v,) in tx_info.vout.iter().enumerate() {
            println!("{:?}: {:?}", i, v, );
            if let Some(addresses) = v.script_pub_key.addresses.clone() {
                if addresses.contains(&escrow.address) {
                    vout = Some(v);
                    break;
                }
            }
        }

        if vout.is_none() {
            return Err(TgError("can't create payout tx. funding tx doesn't spend to escrow address"));
        }
        let vout = vout.unwrap();

//        let vout: GetRawTransactionResultVout = tx_info.vout.iter().filter(|out| { println!("out {:?}", out); if let Some(addresses) = out.script_pub_key.addresses.clone() { return addresses.contains(&escrow.address) } false }).cloned().collect();
//        let p1_address = Address::p2pkh(&escrow.players[0], NETWORK);
//        let p2_address = Address::p2pkh(&escrow.players[1], NETWORK);
//
//        let mut tx_inputs = Vec::<CreateRawTransactionInput>::new();
//
        let payout_amount = vout.value - Amount::from_sat(MINER_FEE);

        let mut outs = HashMap::<String, Amount>::default();
//        let payout_address 
//        outs.insert(&p1_address.to_string(), payout_amount);

        let payout_tx = self.0.create_raw_transaction(
            &[
                CreateRawTransactionInput {
                   txid: funding_tx.txid(),
                   vout: vout.n,
                   sequence: None,
                }
            ],
            &outs,
            None,
            None,
        ).unwrap();

//        println!("{:?}", payout_tx);

//        Err(TgError("oops"))
        Ok(payout_tx)

    }

    fn get_inputs_for_address(&self, address: &Address, amount: Amount) -> (Vec<CreateRawTransactionInput>, Amount,) {

        let unspent = self.0.list_unspent(None,None,Some(&[&address]),None,None).unwrap();

        let mut tx_inputs: Vec<CreateRawTransactionInput> = Vec::new();
        let mut total_in_amount = Amount::ZERO;

        for utxo in unspent {
//            println!("amount: {:?} spendable: {:?}", utxo.amount, utxo.spendable);
            if utxo.spendable {
                tx_inputs.push(
                    CreateRawTransactionInput {
                        txid: utxo.txid,
                        vout: utxo.vout,
                        sequence: None,
                    }
                );

                total_in_amount += utxo.amount;
                if total_in_amount >= amount {
//                    println!("added {:?}, wanted {:?}", total_in_amount, amount);
                    break
                }
            }
        }

        (tx_inputs, total_in_amount)

    } 

    fn sign_challenge_tx(&self, key: PrivateKey, challenge: &mut Challenge) {
        let result = self.0.sign_raw_transaction_with_key(
            challenge.funding_tx_hex.clone(),
            &[key],
            None,
            None,
        );
//        println!("sign tx result: {:?}", result);
        challenge.funding_tx_hex = result.unwrap().hex.raw_hex();
    }

    fn sign_challenge_payout_script(&self, key: PrivateKey, challenge: &mut Challenge) {
// if there is a sig there already, verify it and then add ours on top
        let secp = Secp256k1::new();
        let payout_script_hash = challenge.payout_script_hash();
        challenge.payout_script_hash_sigs.insert(key.public_key(&secp),secp.sign(&Message::from_slice(&payout_script_hash).unwrap(), &key.key));
    }

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
        false
    }

    fn verify_referee_sig(&self, referee: PublicKey,  challenge: &Challenge) -> bool {
//TODO: check funding tx sig too
        if challenge.escrow.referees.contains(&referee) {
            let secp = Secp256k1::new();        
            return secp.verify(
                &Message::from_slice(&challenge.payout_script_hash()).unwrap(),
                &challenge.payout_script_hash_sigs[&referee],
                &referee.key
            ).is_ok()
        }
        false
    }

    fn get_challenge_state(&self, challenge: &Challenge) -> ChallengeState {

        let mut num_player_sigs = 0;
        let secp = Secp256k1::new();
        let payout_script_hash = bitcoin::hashes::sha256::Hash::from_slice(&challenge.payout_script.body).unwrap();
        let msg = Message::from_slice(&payout_script_hash.to_vec()).unwrap();
        for player in &challenge.escrow.players {
            if challenge.payout_script_hash_sigs.contains_key(player) && secp.verify(
                &msg,
                &challenge.payout_script_hash_sigs[player],
                &player.key).is_ok() {
                num_player_sigs += 1;
            }
        }

        let ref_sig: bool = challenge.payout_script_hash_sigs.contains_key(&challenge.escrow.referees[0]) && secp.verify(
            &msg,
            &challenge.payout_script_hash_sigs[&challenge.escrow.referees[0]],
            &challenge.escrow.referees[0].key).is_ok();

        match ref_sig {
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

#[cfg(test)]
mod tests {
    use super::*;

    const PLAYER_1_ADDRESS_LABEL: &'static str = "Player1";
    const PLAYER_2_ADDRESS_LABEL: &'static str = "Player2";
    const REFEREE_ADDRESS_LABEL: &'static str = "Referee";
    const POT_AMOUNT: Amount =  Amount::ONE_BTC;

    fn fund_players(rpc: &Client, p1_address: &Address, p2_address: &Address) {

        let faucet = rpc.get_new_address(Some("faucet"), Some(AddressType::Legacy)).unwrap();
//        println!("faucet address: {:?}", faucet);

        let _result = rpc.generate_to_address(302, &faucet).unwrap();

        let faucet_unspent = rpc.list_unspent(None,None,Some(&[&faucet]),None,None).unwrap();

        let mut tx_inputs: Vec<CreateRawTransactionInput> = Vec::new();
        let mut total_in_amount = Amount::ZERO; 
        let target_in_amount = Amount::ONE_BTC * 2;

        for utxo in faucet_unspent {

//            println!("amount: {:?} spendable: {:?}", utxo.amount, utxo.spendable);
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
//                    println!("hit the goal");
                    break
                }
            }
//
//            println!("{:?} {:?}", total_in_amount, target_in_amount);

        }

//        println!("added {:?} transactions totalling {:?}", tx_inputs.len(), total_in_amount); 

        let mut outs = HashMap::<String, Amount>::default();
        outs.insert(p1_address.to_string(), Amount::ONE_BTC);
        outs.insert(p2_address.to_string(), Amount::ONE_BTC);
        outs.insert(faucet.to_string(), total_in_amount - target_in_amount - Amount::from_sat(MINER_FEE));

        let tx = rpc.create_raw_transaction(
            &tx_inputs,
            &outs,
            None,
            None,
        ).unwrap();

//        println!("tx: {:?}", tx);

        let priv_key = rpc.dump_private_key(&faucet).unwrap();

        let sign_result = rpc.sign_raw_transaction_with_key(
            &tx,
            &[priv_key],
            None,
            None,
        ).unwrap();

//        println!("sign result: {:?}", sign_result);

        let send_result = rpc.send_raw_transaction(&sign_result.hex).unwrap();
//
//        println!("{:?}", send_result);

        let _result = rpc.generate_to_address(110, &faucet).unwrap();

        let tx_info = rpc.get_raw_transaction_info(&send_result,None);

//        println!("{:?}", tx_info);

    }

    fn create_2_of_3_multisig(rpc: &Client, p1_pubkey: PublicKey, p2_pubkey: PublicKey, ref_pubkey: PublicKey) -> MultisigEscrow {

        let result = rpc.add_multisig_address(
            2,
            &[
                PubKeyOrAddress::PubKey(&p1_pubkey),
                PubKeyOrAddress::PubKey(&p2_pubkey),
                PubKeyOrAddress::PubKey(&ref_pubkey)
            ],
            None,
            None,
        ).unwrap();

        MultisigEscrow {
            address: result.address,
            redeem_script: result.redeem_script,
            players: vec!(p1_pubkey.clone(),p2_pubkey.clone()),
            referees: vec!(ref_pubkey.clone()),
        }
    }

    #[test]
    fn create_challenge() {

        let bitcoind_rpc_config = BitcoindRpcConfig::default();

        let client = TgClient(Client::new(
            format!("http://{}:{:?}",
                bitcoind_rpc_config.hostnport.0,
                bitcoind_rpc_config.hostnport.1,
            ),
            Auth::UserPass(
            bitcoind_rpc_config.user.to_string(),
                bitcoind_rpc_config.password.to_string(),
            )
        ).unwrap());

        let p1_address = client.0.get_new_address(Some(PLAYER_1_ADDRESS_LABEL), Some(AddressType::Bech32)).unwrap();
        let p2_address = client.0.get_new_address(Some(PLAYER_2_ADDRESS_LABEL), Some(AddressType::Bech32)).unwrap();
        let ref_address = client.0.get_new_address(Some(REFEREE_ADDRESS_LABEL), Some(AddressType::Bech32)).unwrap();
//
        println!("fund players 1BTC each");

        fund_players(&client.0, &p1_address, &p2_address);

        let p1_balance = client.0.get_received_by_address(&p1_address, None).unwrap();
        let p2_balance = client.0.get_received_by_address(&p2_address, None).unwrap();
        println!("{:?} balance: {:?}", p1_address.to_string(), p1_balance);
        println!("{:?} balance: {:?}", p2_address.to_string(), p2_balance);

        println!("creating challenge:

players:
{:?} 
{:?} 

ref:
{:?}

pot: {:?}
ref fee: {:?}
miner fee: {:?}
buyin: {:?}
",
            p1_address, 
            p2_address, 
            ref_address, 
            POT_AMOUNT,
            POT_AMOUNT/100,
            Amount::from_sat(MINER_FEE),
            (POT_AMOUNT + (POT_AMOUNT/100) + Amount::from_sat(MINER_FEE)) / 2,
            );

        let escrow = create_2_of_3_multisig(&client.0,
            client.0.get_address_info(&p1_address).unwrap().pubkey.unwrap(),
            client.0.get_address_info(&p2_address).unwrap().pubkey.unwrap(),
            client.0.get_address_info(&ref_address).unwrap().pubkey.unwrap(),
        );
        println!("escrow {:?} created", escrow.address.to_string());

        let escrow_address = escrow.address.clone();

        let funding_tx = client.create_challenge_funding_transaction(&escrow, POT_AMOUNT).unwrap();
        println!("funding tx {:?} created", funding_tx.txid());

        let payout_script = PayoutScript::default();
        let payout_script_hash = bitcoin::hashes::sha256::Hash::from_slice(&payout_script.body).unwrap().to_vec();
        println!("payout script {:?} created", bitcoin::consensus::encode::serialize_hex(&payout_script_hash));

        let mut challenge = Challenge {
            escrow,
            funding_tx_hex: funding_tx.raw_hex(),
            payout_script,
            payout_script_hash_sigs: HashMap::<PublicKey, Signature>::default(),
        };
        println!("challenge created",);

        println!("challenge state: {:?}", client.get_challenge_state(&challenge));
        println!("funding tx txid: {:?}", funding_tx.txid());
        println!("p1 signing");
        let p1_key = client.0.dump_private_key(&p1_address).unwrap();
        client.sign_challenge_tx(p1_key, &mut challenge);
        client.sign_challenge_payout_script(p1_key, &mut challenge);
        let funding_tx: bitcoin::Transaction = encode::deserialize(&Vec::<u8>::from_hex(&challenge.funding_tx_hex).unwrap()).unwrap();
        println!("challenge state: {:?}", client.get_challenge_state(&challenge));
        println!("funding tx txid: {:?}", funding_tx.txid());

        println!("p2 signing");
        let p2_key = client.0.dump_private_key(&p2_address).unwrap();
        client.sign_challenge_tx(p2_key, &mut challenge);
        client.sign_challenge_payout_script(p2_key, &mut challenge);
        let funding_tx: bitcoin::Transaction = encode::deserialize(&Vec::<u8>::from_hex(&challenge.funding_tx_hex).unwrap()).unwrap();
        println!("challenge state: {:?}", client.get_challenge_state(&challenge));
        println!("funding tx txid: {:?}", funding_tx.txid());

        println!("ref signing");
        let ref_key = client.0.dump_private_key(&ref_address).unwrap();
        client.sign_challenge_payout_script(ref_key, &mut challenge);
        let funding_tx: bitcoin::Transaction = encode::deserialize(&Vec::<u8>::from_hex(&challenge.funding_tx_hex).unwrap()).unwrap();
        println!("challenge state: {:?}", client.get_challenge_state(&challenge));
        println!("funding tx txid: {:?}", funding_tx.txid());

        println!("broadcasting signed challenge funding transaction");

        let _result = client.0.send_raw_transaction(challenge.funding_tx_hex.clone());

        let _address = client.0.get_new_address(None, Some(AddressType::Legacy)).unwrap();
        let _result = client.0.generate_to_address(10, &_address);

        let ref_balance = client.0.get_received_by_address(&ref_address, None).unwrap();
        println!("ref {:?} balance: {:?}", ref_address.to_string(), ref_balance);
        let ref_balance = client.0.get_received_by_address(&escrow_address, None).unwrap();
        println!("multsig {:?} balance: {:?}", &escrow_address.to_string(), ref_balance);

        let payout_request = PayoutRequest {
            challenge,
            payout_tx: None,
            payout_sig: None,
        };

//        let result = client.verify_payout_request(payout_request);

    }
}
