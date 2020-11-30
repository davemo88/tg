use std::{
    str::FromStr,
    convert::TryInto,
};
use bdk::bitcoin::{
    Address,
    Network,
    PublicKey,
    Script,
    Transaction,
    TxIn,
    TxOut,
    blockdata::{
        script::Builder,
        opcodes::all as Opcodes,
        transaction::OutPoint,
    },
    secp256k1::{
        Secp256k1,
        Message,
        Signature,
    },
    util::{
        bip32::{
            ExtendedPubKey,
            ExtendedPrivKey,
            DerivationPath,
            Fingerprint,
        },
        psbt::PartiallySignedTransaction,
    }
};
use bip39::Mnemonic;
use crate::{
    contract::Contract,
    payout::Payout,
    script::TgScript,
    Result as TgResult,
};

pub const BITCOIN_ACCOUNT_PATH: &'static str = "44'/0'/0'";
pub const ESCROW_SUBACCOUNT: &'static str = "7";

// TODO: need to clarify. this is signing in the normal bitcoin / crypto sense
// and the Signing trait is for signing our contracts and payouts only
// this is here because we will delegate from the app wallet to e.g.
// a hardware wallet for key storage and signing

// we'll only do pubkey-related tasks in our application wallet
// and delegate key storage and signing to a better tested wallet 
// e.g. trezor
pub trait SigningWallet {
    fn fingerprint(&self) -> Fingerprint;
    fn xpubkey(&self) -> ExtendedPubKey;
    fn sign_tx(&self, pstx: PartiallySignedTransaction, descriptor: String) -> TgResult<Transaction>;
    fn sign_message(&self, msg: Message, path: DerivationPath) -> TgResult<Signature>;
}

pub trait EscrowWallet {
    fn get_escrow_pubkey(&self) -> PublicKey;
    fn validate_contract(&self, contract: &Contract) -> TgResult<()>;
    fn validate_payout(&self, payout: &Payout) -> TgResult<()>;
}

pub fn create_escrow_address(p1_pubkey: &PublicKey, p2_pubkey: &PublicKey, arbiter_pubkey: &PublicKey, network: Network) -> TgResult<Address> {
    let escrow_address = Address::p2wsh(
        &create_escrow_script(p1_pubkey, p2_pubkey, arbiter_pubkey),
        network,
    );
    Ok(escrow_address)
}

fn create_escrow_script(p1_pubkey: &PublicKey, p2_pubkey: &PublicKey, arbiter_pubkey: &PublicKey) -> Script {
// standard multisig transaction script
// https://en.bitcoin.it/wiki/BIP_0011
    let b = Builder::new()
        .push_opcode(Opcodes::OP_PUSHBYTES_2)
        .push_slice(&p1_pubkey.to_bytes())
        .push_slice(&p2_pubkey.to_bytes())
        .push_slice(&arbiter_pubkey.to_bytes())
        .push_opcode(Opcodes::OP_PUSHBYTES_3)
        .push_opcode(Opcodes::OP_CHECKMULTISIG);
    b.into_script()
}

pub fn create_payout(contract: &Contract, payout_address: &Address) -> Payout {
    let escrow_address = create_escrow_address(
        &contract.p1_pubkey,
        &contract.p2_pubkey,
        &contract.arbiter_pubkey,
        payout_address.network,
        ).unwrap();
    Payout::new(contract.clone(), create_payout_tx(&contract.funding_tx, &escrow_address, &payout_address).unwrap())
}

// we are ignoring specification of the game master pubkey and substituting
// the arbiter pubkey for the game master here out of laziness
pub fn create_payout_script(p1_pubkey: &PublicKey, p2_pubkey: &PublicKey, arbiter_pubkey: &PublicKey, funding_tx: &Transaction, network: Network) -> TgScript {
    let escrow_address = create_escrow_address(&p1_pubkey, &p2_pubkey, &arbiter_pubkey, network).unwrap();
    let p1_payout_address = Address::p2wpkh(&p1_pubkey, network).unwrap();
    let p1_payout_tx = create_payout_tx(&funding_tx, &escrow_address, &p1_payout_address).unwrap();
    let p2_payout_address = Address::p2wpkh(&p2_pubkey, network).unwrap();
    let p2_payout_tx = create_payout_tx(&funding_tx, &escrow_address, &p2_payout_address).unwrap();
    use crate::script::TgOpcode::*;

    let txid1: &[u8] = &p1_payout_tx.txid();
    let txid2: &[u8] = &p2_payout_tx.txid();
//    ODO should be a pubkeyhash instead of full pubkey, same reasons as bitcoin addresses
//    hat requires the pubkey to also be given as input as in standard pay to pubkey hash
    let pubkey_bytes = arbiter_pubkey.to_bytes();

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

fn create_payout_tx(funding_tx: &Transaction, escrow_address: &Address, payout_address: &Address) -> TgResult<Transaction> {

    let mut input = Vec::<TxIn>::new();
    let mut amount = 0;

    for (i, txout) in funding_tx.output.iter().enumerate() {
        if txout.script_pubkey == escrow_address.script_pubkey() {
            amount = txout.value;
            input.push(TxIn {
                previous_output: OutPoint {
                    txid: funding_tx.txid(),
                    vout: i as u32,
                },
                script_sig: Script::new(),
                sequence: 0,
                witness: Vec::new()
            });
            break;
        }
    }

    Ok(Transaction {
        version: 1,
        lock_time: 0,
        input,
        output: vec!(TxOut { 
            value: amount, 
            script_pubkey: payout_address.script_pubkey() 
        })
    })
}

pub fn derive_account_xprivkey(mnemonic: &Mnemonic, network: Network) -> ExtendedPrivKey {
        let xprivkey = ExtendedPrivKey::new_master(network, &mnemonic.to_seed("")).unwrap();
        let secp = Secp256k1::new();
        let path = DerivationPath::from_str(&String::from(format!("m/{}", BITCOIN_ACCOUNT_PATH))).unwrap();
        xprivkey.derive_priv(&secp, &path).unwrap()
}

pub fn derive_account_xpubkey(mnemonic: &Mnemonic, network: Network) -> ExtendedPubKey {
        let secp = Secp256k1::new();
        ExtendedPubKey::from_private(&secp, &derive_account_xprivkey(mnemonic, network))
}
