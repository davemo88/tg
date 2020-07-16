use std::{
    collections::HashMap,
    convert::TryInto,
};
use bitcoin::{
    Transaction,
    Address,
    Amount,
    util::key::PrivateKey,
    util::key::PublicKey,
};

use bitcoincore_rpc::{
    Client,
    RpcApi,
    RawTx,
    json::CreateRawTransactionInput,
};

use crate::{
    MultisigEscrow,
    Challenge,
    Result,
    TgError,
    NETWORK,
    MINER_FEE,
};

pub struct TgRpcClient(pub Client);

pub trait TgRpcClientApi {
    fn create_challenge_funding_transaction(&self, escrow: &MultisigEscrow, pot: Amount) -> Result<Transaction>;
    fn create_challenge_payout_transaction(&self, escrow: &MultisigEscrow, funding_tx: &Transaction, player: &PublicKey) -> Result<Transaction>;
    fn get_inputs_for_address(&self, address: &Address, amount: Amount) -> (Vec<CreateRawTransactionInput>, Amount,);
// TODO: make sure secp contexts are correctly set up, they shouldn't all be able to sign
// TODO: this function might not belong here because it requires access to a private key
    fn sign_challenge_tx(&self, key: PrivateKey, challenge: &mut Challenge);
}

impl TgRpcClientApi for TgRpcClient {
    fn create_challenge_funding_transaction(&self, escrow: &MultisigEscrow, pot: Amount) -> Result<Transaction> {
        let p1_address = Address::p2wpkh(&escrow.players[0], NETWORK);
        let p2_address = Address::p2wpkh(&escrow.players[1], NETWORK);
        let arbiter_address = Address::p2wpkh(&escrow.arbiters[0], NETWORK);

//        let pot = Amount::ONE_SAT * 2000000;

        let arbiter_fee = pot / 100;
        let buyin = (pot + arbiter_fee + Amount::from_sat(MINER_FEE)) / 2;

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
        outs.insert(arbiter_address.to_string(), arbiter_fee);
        outs.insert(p1_address.to_string(), p1_change);
        outs.insert(p2_address.to_string(), p2_change);

        let tx = self.0.create_raw_transaction(
            &tx_inputs,
            &outs,
            None,
            None,
        ).unwrap();

        let total_in = p1_total_in + p2_total_in;
        let _total_out: Amount = Amount::ONE_SAT * tx.output.iter().map(|txout| txout.value).sum();

//        println!("
//challenge funding tx:
//        p1 total in: {:?}
//        p2 total in: {:?}
//        total in: {:?}
//        total out: {:?}
//        pot: {:?}
//        arbiter fee: {:?}
//        miner fee: {:?}
//        p1 change: {:?}
//        p2 change: {:?} 
//        total in minus change and fees: {:?}",
//            p1_total_in,
//            p2_total_in,
//            total_in,
//            total_out,
//            pot, 
//            arbiter_fee,
//            MINER_FEE, 
//            p1_change, 
//            p2_change,
//            total_in - p1_change - p2_change - arbiter_fee - Amount::from_sat(MINER_FEE),
//        );
        assert!(total_in - p1_change - p2_change - arbiter_fee - Amount::from_sat(MINER_FEE) == pot);
        Ok(tx)
    }

    fn create_challenge_payout_transaction(&self, escrow: &MultisigEscrow, funding_tx: &Transaction, player: &PublicKey) -> Result<Transaction> {
// TODO: should check if the funding tx is in the blockchain since it probably shouldn't be
        let escrow_script_pubkey = escrow.address.script_pubkey();
        let mut vout: Option<(u32, Amount)> = None;

        for (i, txout,) in funding_tx.output.iter().enumerate() {
//            let address = Address::from_script(&txout.script_pubkey, NETWORK);
            if escrow_script_pubkey == txout.script_pubkey {
                vout = Some((i.try_into().unwrap(), Amount::from_sat(txout.value)));
            }
        }

        if vout.is_none() {
            return Err(TgError("could not create payout tx. funding tx does not spend to escrow address"));
        }
        let vout = vout.unwrap();

        let payout_address = Address::p2wpkh(&player, NETWORK);
        let payout_amount = vout.1 - Amount::from_sat(MINER_FEE);

        let mut outs = HashMap::<String, Amount>::default();
        outs.insert(payout_address.to_string(), payout_amount);

        let payout_tx = self.0.create_raw_transaction(
            &[
                CreateRawTransactionInput {
                   txid: funding_tx.txid(),
                   vout: vout.0,
                   sequence: None,
                }
            ],
            &outs,
            None,
            None,
        ).unwrap();

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
}

#[cfg(test)]
mod tests {
//    use super::*;

}
