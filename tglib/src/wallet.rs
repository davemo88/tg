use std::{
    convert::TryInto,
    io::{
        Read,
        Write,
    },
    str::FromStr,
};
use secrecy::ExposeSecret;
use age;
use argon2::{
    self,
    Config,
};
use rand::Rng;
use serde::{
    Deserialize,
    Serialize,
};
use secrecy::Secret;
use bip39::Mnemonic;
use bdk::bitcoin::{
    Address,
    Network,
    PublicKey,
    Script,
    Transaction,
    Txid,
    TxIn,
    TxOut,
    blockdata::{
        script::Builder,
        opcodes::all as Opcodes,
        transaction::OutPoint,
    },
    hashes::{
        ripemd160,
        sha256,
        HashEngine,
        Hash as BitcoinHash,
    },
    secp256k1::{
        Secp256k1,
        Message,
        Signature,
    },
    util::{
        base58,
        bip32::{
            ExtendedPubKey,
            ExtendedPrivKey,
            DerivationPath,
            Fingerprint,
        },
        psbt::PartiallySignedTransaction,
    },
};
use crate::{
    Error,
    contract::Contract,
    payout::Payout,
    script::{
        TgOpcode,
        TgScript,
        TgScriptEnv,
    },
    Result,
    mock::{
        referee_pubkey,
        ESCROW_KIX,
        NETWORK,
    }
};

pub const BITCOIN_ACCOUNT_PATH: &'static str = "44'/0'/0'";
pub const NAMECOIN_ACCOUNT_PATH: &'static str = "44'/7'/0'";
pub const ESCROW_SUBACCOUNT: &'static str = "7";
pub const NAME_SUBACCOUNT: &'static str = "17";
pub const NAME_KIX: &'static str = "0";
pub const TX_FEE: u64 = 20000;

// mainnet
//const NAMECOIN_VERSION_BYTE: u8 = 0x34;//52
// testnet / regtest, same as bitcoin?
const NAMECOIN_TESTNET_VERSION_BYTE: u8 = 0x6F;//111

#[derive(Serialize, Deserialize)]
pub struct SavedSeed {
    pub fingerprint: Fingerprint,
// this is the bitcoin account extended pubkey derived from the encrypted seed
// here for convenience in initializing pubkey-only wallet without having to decrypt seed
    pub xpubkey: ExtendedPubKey,
    pub encrypted_seed: Vec<u8>,
    pub pw_hash: String,
}

impl SavedSeed {
    pub fn new(pw: Secret<String>, mnemonic: Option<Secret<String>>) -> Result<Self> {
//  generate salt
        let pw_salt = rand::thread_rng().gen::<[u8; 32]>().to_vec();
        let config = Config::default();
//  hash pw
        let pw_hash = argon2::hash_encoded(pw.expose_secret().as_bytes(), &pw_salt, &config).unwrap();
        let seed = match mnemonic {
//  if mnemonic provided, derive seed 
            Some(m) => match Mnemonic::from_str(m.expose_secret()) {
                Ok(m) => Secret::new(m.to_seed("").to_vec()),
                Err(_) => return Err(Error::Adhoc("bad mnemonic")),
            }
//  else generate random BIP 39 seed
            None => Secret::new(Mnemonic::from_entropy(&rand::thread_rng().gen::<[u8; 32]>()).unwrap().to_seed("").to_vec()),
        };
//  encrypt seed with pw
        let encrypted_seed = {
            let encryptor = age::Encryptor::with_user_passphrase(pw);
            let mut encrypted = vec![];
            let mut writer = encryptor.wrap_output(&mut encrypted).unwrap();
            writer.write_all(&seed.expose_secret()).unwrap();
            writer.finish().unwrap();

            encrypted
        };

//  compute root key fingerprint and bitcoin account xpubkey
        let root_key = ExtendedPrivKey::new_master(NETWORK, seed.expose_secret()).unwrap();
        let secp = Secp256k1::new();
        let fingerprint = root_key.fingerprint(&secp);
        let xpubkey = derive_account_xpubkey(seed, NETWORK);

        Ok(SavedSeed { 
            fingerprint,
            xpubkey,
            pw_hash,
            encrypted_seed,
        })
    }

    pub fn get_seed(&self, pw: Secret<String>) -> Result<Secret<Vec<u8>>> {
// TODO: fix unwrap on wrong password
        if argon2::verify_encoded(&self.pw_hash, pw.expose_secret().as_bytes()).unwrap() {
            {
                let decryptor = match age::Decryptor::new(&self.encrypted_seed[..]).unwrap() {
                    age::Decryptor::Passphrase(d) => d,
                    _ => unreachable!(),
                };
                let mut decrypted = vec![];
                let mut reader = decryptor.decrypt(&pw, None).unwrap();
                let _r = reader.read_to_end(&mut decrypted);

                Ok(Secret::new(decrypted))
            }
        } else {
            Err(Error::WrongPassword)
        }
    }
}


pub trait NameWallet {
    fn name_pubkey(&self) -> PublicKey;
}

// TODO: enable external signers e.g. hardware wallets
pub trait SigningWallet {
    fn sign_tx(&self, psbt: PartiallySignedTransaction, path: Option<DerivationPath>, pw: Secret<String>) -> Result<PartiallySignedTransaction>;
    fn sign_message(&self, msg: Message, path: DerivationPath, pw: Secret<String>) -> Result<Signature>;
}

pub trait EscrowWallet {
    fn get_escrow_pubkey(&self) -> PublicKey;
    fn validate_contract(&self, contract: &Contract) -> Result<()>;
    fn validate_payout(&self, payout: &Payout) -> Result<()> {
        self.validate_contract(&payout.contract)?;
// payouts require fully signed contracts
        let fully_signed = payout.contract.sigs.len() == 3 as usize;
        if !fully_signed {
            return Err(Error::InvalidPayout("contract is not fully signed"))
        }
        let payout_address = &payout.address().unwrap();
// the payout tx must be an expected one
        let payout_tx = payout.psbt.clone().extract_tx();
        let matching_tx = payout_tx.txid() == create_payout(&payout.contract, &payout_address).psbt.clone().extract_tx().txid();
        if !matching_tx {
            return Err(Error::InvalidPayout("invalid payout tx"))
        }
        let payout_address = payout.address()?;
        if payout_address != payout.contract.p1_payout_address &&
           payout_address != payout.contract.p2_payout_address {
            return Err(Error::InvalidPayout("invalid recipient"))
        }
        if payout.psbt.inputs[0].partial_sigs.len() != 1 { //|| 
// TODO: need to create psbt correctly.  options: 
//  manually set values following bdk::wallet::Wallet::create_tx
//  is it a foreign UTXO ? no but it's to an address not in the descriptor
//  need to test one-off escrow wallet using multisig desriptor
//  set wallet subaccount/address descriptor to something like m/7/0 instead of m/0/*
//  when creating payout tx psbt
//      this might work great
//
            return Err(Error::InvalidPayout("payout tx signed incorrectly"))
        };
        if payout.script_sig.is_none() {
            return Err(Error::InvalidPayout("no script sig"))
        }
        let mut env = TgScriptEnv::new(payout.clone());
        env.validate_payout()
    }
}

pub fn sign_contract<T>(wallet: &T, contract: &Contract, pw: Secret<String>) -> Result<Signature> 
where T: EscrowWallet + SigningWallet {
    Ok(wallet.sign_message(Message::from_slice(&contract.cxid()).unwrap(), 
                DerivationPath::from_str(&format!("m/{}/{}", ESCROW_SUBACCOUNT, ESCROW_KIX)).unwrap(), pw).unwrap())
}

//pub fn sign_payout_psbt<T>(wallet: &T, psbt: PartiallySignedTransaction, pw: Secret<String>) -> Result<PartiallySignedTransaction> 
//where T: EscrowWallet + SigningWallet {
//    wallet.sign_tx(psbt, Some(DerivationPath::from_str(&format!("m/{}/{}", ESCROW_SUBACCOUNT, ESCROW_KIX)).unwrap()), pw)
//}

pub fn create_escrow_address(p1_pubkey: &PublicKey, p2_pubkey: &PublicKey, arbiter_pubkey: &PublicKey, network: Network) -> Result<Address> {
// TODO: could use descriptor here
    let escrow_address = Address::p2wsh(
        &create_escrow_script(p1_pubkey, p2_pubkey, arbiter_pubkey),
        network,
    );
    Ok(escrow_address)
}

pub fn create_escrow_script(p1_pubkey: &PublicKey, p2_pubkey: &PublicKey, arbiter_pubkey: &PublicKey) -> Script {
// standard multisig transaction script
// https://en.bitcoin.it/wiki/BIP_0011
    let b = Builder::new()
        .push_opcode(Opcodes::OP_PUSHNUM_2)
        .push_slice(&p1_pubkey.to_bytes())
        .push_slice(&p2_pubkey.to_bytes())
        .push_slice(&arbiter_pubkey.to_bytes())
        .push_opcode(Opcodes::OP_PUSHNUM_3)
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
    let funding_tx = contract.clone().funding_tx.extract_tx();
    let mut escrow_txout = funding_tx.output.iter().filter(|txout| txout.script_pubkey == escrow_address.script_pubkey());
    let mut psbt: PartiallySignedTransaction = PartiallySignedTransaction::from_unsigned_tx(create_payout_tx(&funding_tx, &escrow_address, &payout_address).unwrap()).unwrap();
    if let Some(txout) = escrow_txout.next() {
        psbt.inputs[0].witness_utxo = Some(txout.clone());
        psbt.inputs[0].witness_script = Some(create_escrow_script(&contract.p1_pubkey, &contract.p2_pubkey, &contract.arbiter_pubkey));
    }
    Payout::new(contract.clone(), psbt)
}

pub fn create_payout_script(escrow_address: &Address, p1_payout_address: &Address, p2_payout_address: &Address, funding_tx: &Transaction) -> TgScript {

    let p1_payout_tx = create_payout_tx(&funding_tx, &escrow_address, &p1_payout_address).unwrap();
    let p2_payout_tx = create_payout_tx(&funding_tx, &escrow_address, &p2_payout_address).unwrap();

    let txid1: &[u8] = &p1_payout_tx.txid();
    let txid2: &[u8] = &p2_payout_tx.txid();

// TODO: should be a pubkeyhash instead of full pubkey, same reasons as bitcoin addresses
// that requires the pubkey to also be given as input as in standard pay to pubkey hash
    let pubkey_bytes = referee_pubkey().to_bytes();

    use crate::script::TgOpcode::*;
    TgScript(vec![         
        OP_PUSHDATA1(pubkey_bytes.len().try_into().unwrap(), pubkey_bytes.clone()),
// p1 payout
        OP_PUSHDATA1(txid1.len().try_into().unwrap(), Vec::from(txid1)),
        OP_PUSHTXID,
        OP_EQUAL,
        OP_IF(
            TgScript(vec![
// wasteful in this case (could use OP_DUP above) but this handles the case of the generic token in place of the payout txid
                OP_PUSHDATA1(txid1.len().try_into().unwrap(), Vec::from(txid1)),
            ]),
            Some(TgScript(vec![
// p2 payout
                OP_PUSHDATA1(txid2.len().try_into().unwrap(), Vec::from(txid2)),
                OP_PUSHTXID,
                OP_EQUAL,
                OP_IF(
                    TgScript(vec![
                        OP_PUSHDATA1(txid2.len().try_into().unwrap(), Vec::from(txid2)),
                    ]),
// unknown payout tx
                    Some(TgScript(vec![OP_0])),
                ),
            ]))
        ),
        OP_VERIFYSIG,
        OP_VALIDATE,
    ])
}

pub fn create_token_pair_script(oracle_pubkey: &PublicKey, pairs: Vec<(Txid, Vec<u8>)>) -> TgScript {
    let mut script = TgScript(vec!());
    let oracle_pubkey_bytes = oracle_pubkey.to_bytes();
    use crate::script::TgOpcode::*;
    script.0.push(OP_PUSHDATA1(oracle_pubkey_bytes.len().try_into().unwrap(), oracle_pubkey_bytes));
    let fragments = pairs.iter().enumerate().map(|(i, (txid, token))| { 
        let last = i == pairs.len() - 1;
        token_branch_fragment(txid, token, last)
    }).collect();
    script.0.extend(nest_fragments(fragments));
// TODO: move OP_VERIFYSIG to the end
    script.0.push(OP_VERIFYSIG);
    script.0.push(OP_VALIDATE);
    script
}

fn nest_fragments(fragments: Vec<Vec<TgOpcode>>) -> Vec<TgOpcode> {
    let (f, fs) = fragments.split_last().unwrap();
    if fs.is_empty() {
        f.to_owned()
    } else {
        let (f2, fs) = fs.split_last().unwrap();
        let (op, ops) = f2.split_last().unwrap();
        let nested = match op {
            TgOpcode::OP_IF(true_branch, None) => TgOpcode::OP_IF(true_branch.to_owned(), Some(TgScript(f.to_owned()))),
            _ => panic!("can't nest this script fragment: op is {:?}", op),
        };
        let mut ops = ops.to_owned();
        ops.push(nested);
        let mut fs = fs.to_owned();
        fs.push(ops);
        nest_fragments(fs)
    }
}

fn token_branch_fragment(txid: &Txid, token: &[u8], last: bool) -> Vec<crate::script::TgOpcode> {
    use crate::script::TgOpcode::*;
    vec![
        OP_PUSHDATA1(txid.len().try_into().unwrap(), Vec::from(txid.as_ref())),
        OP_PUSHTXID,
        OP_EQUAL,
        OP_IF(
            TgScript(vec![
                OP_PUSHDATA1(token.len().try_into().unwrap(), Vec::from(token)),
            ]),
            if last { Some(TgScript(vec!(OP_0))) } else { None },
        ),
    ]
}

pub fn create_payout_tx(funding_tx: &Transaction, escrow_address: &Address, payout_address: &Address) -> Result<Transaction> {

// TODO: need to standardize this with builder-implementation in player wallet
    let (input, amount) = funding_tx.output
        .iter()
        .enumerate()
        .find_map(|(i, txout)| {
            if txout.script_pubkey == escrow_address.script_pubkey() {
                Some((
                    vec!(TxIn {
                        previous_output: OutPoint {
                            txid: funding_tx.txid(),
                            vout: i as u32,
                        },
                        script_sig: Script::new(),
// see bdk::Wallet::create tx
// i believe this disables RBF which is appropriate for now
                        sequence: 0xFFFFFFFF,
                        witness: Vec::new()
                    }),
                    txout.value,
                ))
            } else { None }
    }).ok_or(Error::Adhoc("couldn't find escrow output in funding tx"))?;

    Ok(Transaction {
        version: 1,
        lock_time: 0,
        input,
        output: vec!(TxOut { 
            value: amount - TX_FEE,
            script_pubkey: payout_address.script_pubkey() 
        })
    })
}

pub fn derive_account_xprivkey(seed: Secret<Vec<u8>>, network: Network) -> ExtendedPrivKey {
        let root_key = ExtendedPrivKey::new_master(network, seed.expose_secret()).unwrap();
        let secp = Secp256k1::new();
        let path = DerivationPath::from_str(&String::from(format!("m/{}", BITCOIN_ACCOUNT_PATH))).unwrap();
        root_key.derive_priv(&secp, &path).unwrap()
}

pub fn derive_account_xpubkey(seed: Secret<Vec<u8>>, network: Network) -> ExtendedPubKey {
        let secp = Secp256k1::new();
        ExtendedPubKey::from_private(&secp, &derive_account_xprivkey(seed, network))
}

pub fn get_namecoin_address(pubkey: &PublicKey, network: Network) -> String {
    let mut sha256_engine = sha256::HashEngine::default();
    sha256_engine.input(&pubkey.key.serialize());
    let hash: &[u8] = &sha256::Hash::from_engine(sha256_engine);

    let mut ripemd160_engine = ripemd160::HashEngine::default();
    ripemd160_engine.input(hash);
    let hash = &ripemd160::Hash::from_engine(ripemd160_engine);

    let mut hash = hash.to_vec();
    match network {
        Network::Bitcoin => {
            panic!("nice try, sucker");
//            hash.insert(0,NAMECOIN_VERSION_BYTE);
        },
        Network::Regtest | Network::Testnet | Network::Signet => {
            hash.insert(0,NAMECOIN_TESTNET_VERSION_BYTE);
        }
    }

    base58::check_encode_slice(&hash)
}

#[cfg(test)]
mod tests {

    use super::*;
    use hex;
    use crate::{
        mock::{
            get_referee_signature,
            Trezor,
            ARBITER_MNEMONIC,
            ESCROW_KIX,
            ESCROW_SUBACCOUNT,
            NETWORK,
            PLAYER_1_MNEMONIC,
            PLAYER_2_MNEMONIC,
        },
    };

    const CONTRACT: &'static str = "";

    const PUBKEY: &'static str = "02123e6a7816f2149f90cca1ea1ba41b73e77db44cd71f01c184defd10961d03fc";
    const TESTNET_ADDRESS_FROM_NAMECOIND: &'static str = "mfuf8qvMsMJMgBqtEGBt8aCQPQi1qgANzo";

    #[test]
    fn test_get_namecoin_address() {
        let pubkey = PublicKey::from_slice(&hex::decode(PUBKEY).unwrap()).unwrap();
        let namecoin_address = get_namecoin_address(&pubkey, Network::Testnet).unwrap();
        assert_eq!(namecoin_address,TESTNET_ADDRESS_FROM_NAMECOIND)
    }

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
        payout.psbt = sign_payout_psbt(&wallet, &mut payout).unwrap();
        let arbiter_wallet = Trezor::new(Mnemonic::parse(ARBITER_MNEMONIC).unwrap());
        let msg = Message::from_slice(&payout.psbt.clone().extract_tx().txid()).unwrap();
        payout.script_sig = Some(get_referee_signature(msg));

        assert!(arbiter_wallet.validate_payout(&payout).is_ok())
    }

    #[test]
    fn pass_p2_payout() {
        let mut contract = Contract::from_bytes(hex::decode(CONTRACT).unwrap()).unwrap();
        all_sign(&mut contract);

        let address = Address::p2wpkh(&contract.p2_pubkey, NETWORK).unwrap();
        let mut payout = create_payout(&contract, &address);
        let wallet = Trezor::new(Mnemonic::parse(PLAYER_2_MNEMONIC).unwrap());
        payout.psbt = sign_payout_psbt(&wallet, &mut payout).unwrap();
        let arbiter_wallet = Trezor::new(Mnemonic::parse(ARBITER_MNEMONIC).unwrap());
        let msg = Message::from_slice(&payout.psbt.clone().extract_tx().txid()).unwrap();
        payout.script_sig = Some(get_referee_signature(msg));

        let r = arbiter_wallet.validate_payout(&payout);
        assert!(r.is_ok())
    }

    #[test]
    fn fail_unsigned_contract() {
        let contract = Contract::from_bytes(hex::decode(CONTRACT).unwrap()).unwrap();

        let address = Address::p2wpkh(&contract.p1_pubkey, NETWORK).unwrap();
        let mut payout = create_payout(&contract, &address);
        let wallet = Trezor::new(Mnemonic::parse(PLAYER_1_MNEMONIC).unwrap());
        payout.psbt = sign_payout_psbt(&wallet, &mut payout).unwrap();
        let arbiter_wallet = Trezor::new(Mnemonic::parse(ARBITER_MNEMONIC).unwrap());
        let msg = Message::from_slice(&payout.psbt.clone().extract_tx().txid()).unwrap();
        payout.script_sig = Some(get_referee_signature(msg));

        assert!(arbiter_wallet.validate_payout(&payout).is_err())
    }

    #[test]
    fn fail_no_script_sig() {
        let mut contract = Contract::from_bytes(hex::decode(CONTRACT).unwrap()).unwrap();
        all_sign(&mut contract);

        let address = Address::p2wpkh(&contract.p1_pubkey, NETWORK).unwrap();
        let mut payout = create_payout(&contract, &address);
        let wallet = Trezor::new(Mnemonic::parse(PLAYER_1_MNEMONIC).unwrap());
        payout.psbt = sign_payout_psbt(&wallet, &mut payout).unwrap();
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
        payout.psbt = sign_payout_psbt(&wallet, &mut payout).unwrap();
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
        let msg = Message::from_slice(&payout.psbt.clone().extract_tx().txid()).unwrap();
        payout.script_sig = Some(get_referee_signature(msg));
        assert!(arbiter_wallet.validate_payout(&payout).is_err())

    }

    #[test]
    fn fail_invalid_payout_tx() {
        let mut contract = Contract::from_bytes(hex::decode(CONTRACT).unwrap()).unwrap();
        all_sign(&mut contract);

        let wallet = Trezor::new(Mnemonic::parse(PLAYER_1_MNEMONIC).unwrap());
// giving a new address for the payout tx instead of the ones baked into the payout script
        let mut payout = create_payout(&contract, &wallet.wallet.get_new_address().unwrap());
        payout.psbt = sign_payout_psbt(&wallet, &mut payout).unwrap();
        let arbiter_wallet = Trezor::new(Mnemonic::parse(ARBITER_MNEMONIC).unwrap());
        let msg = Message::from_slice(&payout.psbt.clone().extract_tx().txid()).unwrap();
        payout.script_sig = Some(get_referee_signature(msg));
        assert!(arbiter_wallet.validate_payout(&payout).is_err())
    }
}
