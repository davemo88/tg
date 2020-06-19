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

use crate::lib::{
    MultisigEscrow,
    BitcoindRpcConfig,
    Challenge,
    ChallengeApi,
    ChallengeState,
    PayoutScript,
};

pub const NETWORK: Network = Network::Regtest;
pub const MINER_FEE: u64 = 10000;
pub const NUM_PLAYERS: u64 = 2;

pub struct TgClient(Client);

trait TgClientApi {
    fn create_challenge_funding_transaction(&self, escrow: &MultisigEscrow, pot: Amount) -> Transaction;
    fn get_inputs_for_address(&self, address: &Address, amount: Amount) -> (Vec<CreateRawTransactionInput>, Amount,);
    fn sign_challenge_tx(&self, key: PrivateKey, challenge: &mut Challenge);
    fn sign_challenge_payout_script(&self, key: PrivateKey, challenge: &mut Challenge);
    fn get_challenge_state(&self, challenge: &Challenge) -> ChallengeState;
    fn verify_player_sigs(&self, player: PublicKey, challenge: &Challenge) -> bool;
}

impl TgClientApi for TgClient {
    fn create_challenge_funding_transaction(&self, escrow: &MultisigEscrow, pot: Amount) -> Transaction {
        let p1_address = Address::p2pkh(&escrow.players[0], NETWORK);
        let p2_address = Address::p2pkh(&escrow.players[1], NETWORK);
        let ref_address = Address::p2pkh(&escrow.referees[0], NETWORK);

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
        tx
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
        let mut faucet_unspent = faucet_unspent.iter();

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

        let p1_address = client.0.get_new_address(Some(PLAYER_1_ADDRESS_LABEL), Some(AddressType::Legacy)).unwrap();
        let p2_address = client.0.get_new_address(Some(PLAYER_2_ADDRESS_LABEL), Some(AddressType::Legacy)).unwrap();
        let ref_address = client.0.get_new_address(Some(REFEREE_ADDRESS_LABEL), Some(AddressType::Legacy)).unwrap();
//
        println!("fund players 1BTC each");

        fund_players(&client.0, &p1_address, &p2_address);

        let p1_balance = client.0.get_received_by_address(&p1_address, None).unwrap();
        let p2_balance = client.0.get_received_by_address(&p2_address, None).unwrap();
        println!("{:?} balance: {:?}", p1_address.to_string(), p1_balance);
        println!("{:?} balance: {:?}", p2_address.to_string(), p2_balance);

        let escrow = create_2_of_3_multisig(&client.0,
            client.0.get_address_info(&p1_address).unwrap().pubkey.unwrap(),
            client.0.get_address_info(&p2_address).unwrap().pubkey.unwrap(),
            client.0.get_address_info(&ref_address).unwrap().pubkey.unwrap(),
        );

        let escrow_address = escrow.address.clone();

        println!("create challenge:
players:
{:?} 
{:?} 
ref
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

        println!("create challenge funding tx",);
        let funding_tx = client.create_challenge_funding_transaction(&escrow, POT_AMOUNT);

        let mut challenge = Challenge {
            escrow,
            funding_tx_hex: funding_tx.raw_hex(),
            payout_script: PayoutScript::default(),
            payout_script_hash_sigs: HashMap::<PublicKey, Signature>::default(),
        };

        println!("challenge ready to sign",);

        println!("challenge state: {:?}", client.get_challenge_state(&challenge));
        println!("p1 signing");
        let p1_key = client.0.dump_private_key(&p1_address).unwrap();
        client.sign_challenge_tx(p1_key, &mut challenge);
        client.sign_challenge_payout_script(p1_key, &mut challenge);
        println!("challenge state: {:?}", client.get_challenge_state(&challenge));

        println!("p2 signing");
        let p2_key = client.0.dump_private_key(&p2_address).unwrap();
        client.sign_challenge_tx(p2_key, &mut challenge);
        client.sign_challenge_payout_script(p2_key, &mut challenge);
        println!("challenge state: {:?}", client.get_challenge_state(&challenge));

        println!("ref signing");
        let ref_key = client.0.dump_private_key(&ref_address).unwrap();
        client.sign_challenge_payout_script(ref_key, &mut challenge);
        println!("challenge state: {:?}", client.get_challenge_state(&challenge));
        println!("broadcasting signed challenge funding transaction");

        let _result = client.0.send_raw_transaction(challenge.funding_tx_hex.clone());

        let _address = client.0.get_new_address(None, Some(AddressType::Legacy)).unwrap();
        let _result = client.0.generate_to_address(10, &_address);

        let ref_balance = client.0.get_received_by_address(&ref_address, None).unwrap();
        println!("ref {:?} balance: {:?}", ref_address.to_string(), ref_balance);
        let ref_balance = client.0.get_received_by_address(&escrow_address, None).unwrap();
        println!("multsig {:?} balance: {:?}", &escrow_address.to_string(), ref_balance);
    }
}
