use std::{
    convert::TryInto,
    str::FromStr,
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
    TgError,
    contract::Contract,
    payout::Payout,
    script::{
        TgScript,
        TgScriptEnv,
    },
    Result as TgResult,
    mock::{
        ESCROW_KIX,
    }
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
    fn sign_tx(&self, pstx: PartiallySignedTransaction, descriptor: String) -> TgResult<PartiallySignedTransaction>;
    fn sign_message(&self, msg: Message, path: DerivationPath) -> TgResult<Signature>;
}

pub trait EscrowWallet {
    fn get_escrow_pubkey(&self) -> PublicKey;
    fn validate_contract(&self, contract: &Contract) -> TgResult<()>;
    fn validate_payout(&self, payout: &Payout) -> TgResult<()> {
        if self.validate_contract(&payout.contract).is_ok() {
// payouts require fully signed contracts
            let fully_signed = payout.contract.sigs.len() == 3 as usize;
// the payout tx must be an expected one
            let payout_address = &payout.address().unwrap();
            let payout_tx = payout.psbt.clone().extract_tx();
            let matching_tx = payout_tx.txid() == create_payout(&payout.contract, &payout_address).psbt.clone().extract_tx().txid();
            if !fully_signed {
                return Err(TgError("invalid payout - not fully signed"))
            }
            if !matching_tx {
                return Err(TgError("invalid payout - no matching tx"))
            }
            let recipient_pubkey = payout.recipient_pubkey();
            if recipient_pubkey.is_err() {
                return Err(TgError("invalid payout - invalid recipient"))
            }
            println!("{:?}", payout.psbt);
            if payout.psbt.inputs[0].partial_sigs.len() != 1 { //|| 
//                !payout.psbt.inputs[0].partial_sigs.contains_key(&recipient_pubkey.unwrap()) {
// TODO: need to create psbt correctly.  options: 
//  manually set values following bdk::wallet::Wallet::create_tx
//  is it a foreign UTXO ? no but it's to an address not in the descriptor
//  need to test one-off escrow wallet using multisig desriptor
//  set wallet subaccount/address descriptor to something like m/7/0 instead of m/0/*
//  when creating payout tx psbt
//      this might work great
//
//
//
                return Err(TgError("invalid payout - psbt signed improperly"))
            };
            let mut env = TgScriptEnv::new(payout.clone());
            env.validate_payout()
        } else {
            Err(TgError("invalid payout"))
        }
    }
}

pub fn sign_contract<T>(wallet: &T, contract: &mut Contract) -> TgResult<Signature> 
where T: EscrowWallet + SigningWallet {
    Ok(wallet.sign_message(Message::from_slice(&contract.cxid()).unwrap(), 
                DerivationPath::from_str(&format!("m/{}/{}", ESCROW_SUBACCOUNT, ESCROW_KIX)).unwrap()).unwrap())
}

pub fn sign_payout<T>(wallet: &T, payout: &mut Payout) -> TgResult<PartiallySignedTransaction> 
where T: EscrowWallet + SigningWallet{
    wallet.sign_tx(payout.psbt.clone(), "".to_string())
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
    let mut escrow_txout = contract.funding_tx.output.iter().filter(|txout| txout.script_pubkey == escrow_address.script_pubkey());
    let mut psbt: PartiallySignedTransaction = PartiallySignedTransaction::from_unsigned_tx(create_payout_tx(&contract.funding_tx, &escrow_address, &payout_address).unwrap()).unwrap();
    if let Some(txout) = escrow_txout.next() {
        psbt.inputs[0].witness_utxo = Some(txout.clone());
        psbt.inputs[0].witness_script = Some(create_escrow_script(&contract.p1_pubkey, &contract.p2_pubkey, &contract.arbiter_pubkey));
    }
    Payout::new(contract.clone(), psbt)
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
// TODO: should be a pubkeyhash instead of full pubkey, same reasons as bitcoin addresses
// that requires the pubkey to also be given as input as in standard pay to pubkey hash
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

#[cfg(test)]
mod tests {

    use super::*;
    use hex;
    use crate::{
        mock::{
            Trezor,
            ARBITER_MNEMONIC,
            ESCROW_KIX,
            ESCROW_SUBACCOUNT,
            NETWORK,
            PLAYER_1_MNEMONIC,
            PLAYER_2_MNEMONIC,
        },
    };

    const CONTRACT: &'static str = "010344ef4fe364c72338081a390e3311c4640d98160cee450752196df7992270189f0340ae7715992335778916e592d46ba5820e0d0b29df09d1db49ef7f858698d39c0321c5107071c453264592ae948fc124f9b9ff46e286f9eb47510cb9bd2e6b4116000000e4010000000287bda987fe2804df6385b3c6057d4387f59f10ef6456b72d2e817a705f4d0c63010000000000000000d83b84dc7d1937afd7f7082ed19817f4c5cb1aecffb863fee82db61d60909a3500000000000000000004511100000000000022002028b7b4e22b42a8d96c633b8822a96e2f8cf488df63b2f74e1b7dbde61f4f7c122c000000000000001600148cd86fcd528f929d1aa329e4f5069b9098847b7f42d8f50500000000160014dbd9ebc5e9498496628dc7c9d53a9db4f3b8e70d42d8f5050000000016001451382c18691f35dea9802ea3aac88d109de790f7000000000000006fd1210321c5107071c453264592ae948fc124f9b9ff46e286f9eb47510cb9bd2e6b411653d1202de119a07f0d5256e4fbe5394d800f71055401c80bb4e4b49df1e4e2b646b2a9c1f101f2d12045210638061238c16b3d4f1c62802452d2bbb38099f6efca8be4b26a79065206c1f3f4";

    fn all_sign(contract: &mut Contract) {
        let p1_wallet = Trezor::new(Mnemonic::parse(PLAYER_1_MNEMONIC).unwrap());
        let p2_wallet = Trezor::new(Mnemonic::parse(PLAYER_2_MNEMONIC).unwrap());
        let arbiter_wallet = Trezor::new(Mnemonic::parse(ARBITER_MNEMONIC).unwrap());
        let sig = sign_contract(&p1_wallet, contract).unwrap();
        contract.sigs.push(sig);
        let sig = sign_contract(&p2_wallet, contract).unwrap();
        contract.sigs.push(sig);
        let sig = sign_contract(&arbiter_wallet, contract).unwrap();
        contract.sigs.push(sig);
    }

    #[test]
    fn pass_p1_payout() {
        let mut contract = Contract::from_bytes(hex::decode(CONTRACT).unwrap()).unwrap();
        all_sign(&mut contract);

        let address = Address::p2wpkh(&contract.p1_pubkey, NETWORK).unwrap();
        let mut payout = create_payout(&contract, &address);
        let wallet = Trezor::new(Mnemonic::parse(PLAYER_1_MNEMONIC).unwrap());
        payout.psbt = sign_payout(&wallet, &mut payout).unwrap();
        let arbiter_wallet = Trezor::new(Mnemonic::parse(ARBITER_MNEMONIC).unwrap());
        payout.script_sig = Some(arbiter_wallet.sign_message(Message::from_slice(&payout.psbt.clone().extract_tx().txid()).unwrap(), 
                DerivationPath::from_str(&format!("m/{}/{}", ESCROW_SUBACCOUNT, ESCROW_KIX)).unwrap()).unwrap());

        assert!(arbiter_wallet.validate_payout(&payout).is_ok())
    }

    #[test]
    fn pass_p2_payout() {
        let mut contract = Contract::from_bytes(hex::decode(CONTRACT).unwrap()).unwrap();
        all_sign(&mut contract);

        let address = Address::p2wpkh(&contract.p2_pubkey, NETWORK).unwrap();
        let mut payout = create_payout(&contract, &address);
        let wallet = Trezor::new(Mnemonic::parse(PLAYER_2_MNEMONIC).unwrap());
        payout.psbt = sign_payout(&wallet, &mut payout).unwrap();
        let arbiter_wallet = Trezor::new(Mnemonic::parse(ARBITER_MNEMONIC).unwrap());
        payout.script_sig = Some(arbiter_wallet.sign_message(Message::from_slice(&payout.psbt.clone().extract_tx().txid()).unwrap(), 
                DerivationPath::from_str(&format!("m/{}/{}", ESCROW_SUBACCOUNT, ESCROW_KIX)).unwrap()).unwrap());

        let r = arbiter_wallet.validate_payout(&payout);
        assert!(r.is_ok())
    }

    #[test]
    fn fail_unsigned_contract() {
        let contract = Contract::from_bytes(hex::decode(CONTRACT).unwrap()).unwrap();

        let address = Address::p2wpkh(&contract.p1_pubkey, NETWORK).unwrap();
        let mut payout = create_payout(&contract, &address);
        let wallet = Trezor::new(Mnemonic::parse(PLAYER_1_MNEMONIC).unwrap());
        payout.psbt = sign_payout(&wallet, &mut payout).unwrap();
        let arbiter_wallet = Trezor::new(Mnemonic::parse(ARBITER_MNEMONIC).unwrap());
        payout.script_sig = Some(arbiter_wallet.sign_message(Message::from_slice(&payout.psbt.clone().extract_tx().txid()).unwrap(), 
                DerivationPath::from_str(&format!("m/{}/{}", ESCROW_SUBACCOUNT, ESCROW_KIX)).unwrap()).unwrap());

        assert!(arbiter_wallet.validate_payout(&payout).is_err())
    }

    #[test]
    fn fail_no_script_sig() {
        let mut contract = Contract::from_bytes(hex::decode(CONTRACT).unwrap()).unwrap();
        all_sign(&mut contract);

        let address = Address::p2wpkh(&contract.p1_pubkey, NETWORK).unwrap();
        let mut payout = create_payout(&contract, &address);
        let wallet = Trezor::new(Mnemonic::parse(PLAYER_1_MNEMONIC).unwrap());
        payout.psbt = sign_payout(&wallet, &mut payout).unwrap();
        let arbiter_wallet = Trezor::new(Mnemonic::parse(ARBITER_MNEMONIC).unwrap());

        assert!(arbiter_wallet.validate_payout(&payout).is_err())
    }

    #[test]
    fn fail_invalid_script_sig() {
        let mut contract = Contract::from_bytes(hex::decode(CONTRACT).unwrap()).unwrap();
        all_sign(&mut contract);

        let address = Address::p2wpkh(&contract.p1_pubkey, NETWORK).unwrap();
        let mut payout = create_payout(&contract, &address);
        let wallet = Trezor::new(Mnemonic::parse(PLAYER_1_MNEMONIC).unwrap());
        payout.psbt = sign_payout(&wallet, &mut payout).unwrap();
        let arbiter_wallet = Trezor::new(Mnemonic::parse(ARBITER_MNEMONIC).unwrap());
// signing with the player's wallet incorrectly
        payout.script_sig = Some(wallet.sign_message(Message::from_slice(&payout.psbt.clone().extract_tx().txid()).unwrap(), 
                DerivationPath::from_str(&format!("m/{}/{}", ESCROW_SUBACCOUNT, ESCROW_KIX)).unwrap()).unwrap());

        assert!(arbiter_wallet.validate_payout(&payout).is_err())
    }

    #[test]
    fn fail_unsigned_payout_tx() {
        let mut contract = Contract::from_bytes(hex::decode(CONTRACT).unwrap()).unwrap();
        all_sign(&mut contract);

        let address = Address::p2wpkh(&contract.p1_pubkey, NETWORK).unwrap();
        let mut payout = create_payout(&contract, &address);
        let arbiter_wallet = Trezor::new(Mnemonic::parse(ARBITER_MNEMONIC).unwrap());
        payout.script_sig = Some(arbiter_wallet.sign_message(Message::from_slice(&payout.psbt.clone().extract_tx().txid()).unwrap(), 
                DerivationPath::from_str(&format!("m/{}/{}", ESCROW_SUBACCOUNT, ESCROW_KIX)).unwrap()).unwrap());

        assert!(arbiter_wallet.validate_payout(&payout).is_err())

    }

    #[test]
    fn fail_invalid_payout_tx() {
        let mut contract = Contract::from_bytes(hex::decode(CONTRACT).unwrap()).unwrap();
        all_sign(&mut contract);

        let wallet = Trezor::new(Mnemonic::parse(PLAYER_1_MNEMONIC).unwrap());
// giving a new address for the payout tx instead of the ones baked into the payout script
        let mut payout = create_payout(&contract, &wallet.wallet.get_new_address().unwrap());
        payout.psbt = sign_payout(&wallet, &mut payout).unwrap();
        let arbiter_wallet = Trezor::new(Mnemonic::parse(ARBITER_MNEMONIC).unwrap());
        payout.script_sig = Some(arbiter_wallet.sign_message(Message::from_slice(&payout.psbt.clone().extract_tx().txid()).unwrap(), 
                DerivationPath::from_str(&format!("m/{}/{}", ESCROW_SUBACCOUNT, ESCROW_KIX)).unwrap()).unwrap());

        assert!(arbiter_wallet.validate_payout(&payout).is_err())
    }

}
