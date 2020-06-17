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
    json::{
        PubKeyOrAddress,
        AddressType,
        CreateRawTransactionInput,
        SignRawTransactionInput,
    },
};

pub const PAYOUT_SCRIPT_MAX_SIZE: usize = 32;
pub const TERMS_SIZE: usize = 100;
pub const LOCALHOST: &'static str = "0.0.0.0";
pub const TESTNET_RPC_PORT: usize = 18332;
pub const MINER_FEE: Amount = Amount::ZERO;//satoshis
pub const RAKE: usize = 1;//percent

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

#[derive(Hash)]
pub struct PayoutScript {
    body: [u8; PAYOUT_SCRIPT_MAX_SIZE],
}

pub struct Challenge {
    escrow: MultisigEscrow,
// this tx needs to be mined before the payout script can be signed
// because we need to create txs that spend from it in the payout script
// there is no problem with this for the following reasons:
// 1. the ref doesn't need to sign the funding tx since they don't contribute coins
// 2. in a 2/3 multisig, the players can recover their funds at any time, e.g.
// if the challenge doesn't proceed for some reason
    funding_tx: Transaction,
// must include data unique to this challenge e.g. funding tx id, so old signatures for similar
// payouts (e.g. rematches) can't be used, since its hash is signed and use for later verification
// including specific payout transactions in the payout script might be sufficient for uniqueness
// if they are spending from the funding tx utxos
    payout_script: PayoutScript,
// https://www.sans.org/reading-room/whitepapers/infosec/digital-signature-multiple-signature-cases-purposes-1154
// the parties sign this as well as the funding transaction
// a signed hash of the payout script
// because this contains the funding tx id, it will be unique
//
// signed hash of the payout script 
// this value is unique to this challenge because it includes the funding_tx id
    payout_script_hash_sig: Option<Signature>,
}

impl Challenge {
    pub fn state(&self) -> ChallengeState {
        ChallengeState::Issued
    }
}

pub enum ChallengeState {
// signed by p1
    Issued,
// signed by p2
    Accepted,
// signed by ref
    Certified,
}

pub struct PayoutRequest {
    challenge: Challenge,
    payout_tx: Transaction,
    payout_sig: Signature,
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
    address: Address, 
    redeem_script: Script,
    players: Vec<PublicKey>,
    referees: Vec<PublicKey>,
}

#[cfg(test)]
mod tests {
    use super::*;

    const PLAYER_1_ADDRESS_LABEL: &'static str = "Player1";
    const PLAYER_2_ADDRESS_LABEL: &'static str = "Player2";
    const REFEREE_ADDRESS_LABEL: &'static str = "Referee";

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

    fn fund_players(rpc: &Client, p1_address: &Address, p2_address: &Address) {

        let faucet = rpc.get_new_address(Some("faucet"), Some(AddressType::Legacy)).unwrap();

        let _result = rpc.generate_to_address(202, &faucet).unwrap();

        let faucet_unspent = rpc.list_unspent(None,None,Some(&[&faucet]),None,None).unwrap();
        let mut faucet_unspent = faucet_unspent.iter();

        let mut tx_inputs: Vec<CreateRawTransactionInput> = Vec::new();
        let mut total_in_amount = Amount::ZERO; 
        let target_in_amount = Amount::ONE_BTC * 2;

        for utxo in faucet_unspent {

            println!("amount: {:?} spendable: {:?}", utxo.amount, utxo.spendable);
            if utxo.amount > Amount::ZERO {
                tx_inputs.push(
                    CreateRawTransactionInput {
                        txid: utxo.txid,
                        vout: 0,
                        sequence: None,
                    }
                );

                total_in_amount += utxo.amount;
                if total_in_amount >= target_in_amount {
                    break
                }
            }
//
            println!("{:?} {:?}", total_in_amount, target_in_amount);

        }

        println!("added {:?} transactions totalling {:?}", tx_inputs.len(), total_in_amount); 

        let mut outs = HashMap::<String, Amount>::default();
        outs.insert(p1_address.to_string(), Amount::ONE_BTC);
        outs.insert(p2_address.to_string(), Amount::ONE_BTC);

        let tx = rpc.create_raw_transaction(
            &tx_inputs,
            &outs,
            None,
            None,
        ).unwrap();

        println!("tx: {:?}", tx);

        let tx = rpc.fund_raw_transaction(&tx, None, None).unwrap().hex;

        let priv_key = rpc.dump_private_key(&faucet).unwrap();

//        let unspent_0_info = rpc.get_raw_transaction_info(&faucet_unspent[0].txid,None).unwrap();
//        let unspent_1_info = rpc.get_raw_transaction_info(&faucet_unspent[1].txid,None).unwrap();
//
//        let unspent_0_script_pub_key = Script::from(unspent_0_info.vout[0].script_pub_key.clone().hex);
//        let unspent_1_script_pub_key = Script::from(unspent_1_info.vout[1].script_pub_key.clone().hex);

        let sign_result = rpc.sign_raw_transaction_with_key(
            &tx,
            &[priv_key],
//            &[],
            None,
//            Some(&[
//                SignRawTransactionInput {
//                    txid: faucet_unspent[0].txid,
//                    vout: 0,
//                    script_pub_key: unspent_0_script_pub_key,
//                    redeem_script: None,
//                    amount: Some(Amount::ONE_BTC),
//                },
//                SignRawTransactionInput {
//                    txid: faucet_unspent[1].txid,
//                    vout: 1,
//                    script_pub_key: unspent_1_script_pub_key,
//                    redeem_script: None,
//                    amount: Some(Amount::ONE_BTC),
//                },
//            ]),
            None,
        ).unwrap();

        println!("sign result: {:?}", sign_result);

        let send_result = rpc.send_raw_transaction(&sign_result.hex).unwrap();
//
        println!("{:?}", send_result);

        let _result = rpc.generate_to_address(110, &faucet).unwrap();

        let tx_info = rpc.get_raw_transaction_info(&send_result,None);

        println!("{:?}", tx_info);

    }

    #[test]
    fn create_challenge() {

        println!("create challenge test");

        let bitcoind_rpc_config = BitcoindRpcConfig::default();
        let rpc = Client::new(
            format!("http://{}:{:?}",
                bitcoind_rpc_config.hostnport.0,
                bitcoind_rpc_config.hostnport.1,
            ),
            Auth::UserPass(
            bitcoind_rpc_config.user.to_string(),
                bitcoind_rpc_config.password.to_string(),
            )
        ).unwrap();

        let p1_address = rpc.get_new_address(Some(PLAYER_1_ADDRESS_LABEL), Some(AddressType::Legacy)).unwrap();
        let p2_address = rpc.get_new_address(Some(PLAYER_2_ADDRESS_LABEL), Some(AddressType::Legacy)).unwrap();
        let ref_address = rpc.get_new_address(Some(REFEREE_ADDRESS_LABEL), Some(AddressType::Legacy)).unwrap();

        let escrow = create_2_of_3_multisig(&rpc,
            rpc.get_address_info(&p1_address).unwrap().pubkey.unwrap(),
            rpc.get_address_info(&p2_address).unwrap().pubkey.unwrap(),
            rpc.get_address_info(&ref_address).unwrap().pubkey.unwrap(),
        );

        println!("created 2 of 3 multsig {:?}", escrow.address);

        fund_players(&rpc, &p1_address, &p2_address);

//        let p1_unspent = rpc.list_unspent(None,None,Some(&[&p1_address]),None,None).unwrap();
//        let p2_unspent = rpc.list_unspent(None,None,Some(&[&p2_address]),None,None).unwrap();
////
//        println!("p1_unspent: {:?}", p1_unspent[0]);
//        println!("p2_unspent: {:?}", p2_unspent[0]);
////
        let ref_balance = rpc.get_received_by_address(&ref_address, None).unwrap();
        let p1_balance = rpc.get_received_by_address(&p1_address, None).unwrap();
        let p2_balance = rpc.get_received_by_address(&p2_address, None).unwrap();
////
        println!("ref_balance: {:?}", ref_balance);
        println!("p1_balance: {:?}", p1_balance);
        println!("p2_balance: {:?}", p2_balance);

        let mut outs = HashMap::<String, Amount>::default();
        let pot = Amount::ONE_BTC;
        let ref_fee = pot / 100;
// each put a half bitcoin in the pot
        outs.insert(format!("{:?}",escrow.address), pot );
// pay the ref
        outs.insert(format!("{:?}",ref_address), ref_fee );
// send remainders back to players
        outs.insert(format!("{:?}",ref_address), ref_fee );
        outs.insert(format!("{:?}",ref_address), ref_fee );

//        let funding_tx = rpc.create_raw_transaction(
//            &[
//                CreateRawTransactionInput{
//                    txid: p1_unspent[0].txid,
//                    vout: 0,
//                    sequence: None,
//                },
//                CreateRawTransactionInput{
//                    txid: p2_unspent[0].txid,
//                    vout: 1,
//                    sequence: None,
//                },
//            ],
//            &outs,
//            None,
//            None,
//        ).unwrap();
//
//        println!("funding_tx: {:?}", funding_tx);
//
//        let challenge = Challenge {
//            escrow,
//            funding_tx,
//            terms: [1; TERMS_SIZE],
//            terms_sig: None,
//        };
    }
}
