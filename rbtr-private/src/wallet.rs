use std::str::FromStr;
use tglib::{
    bdk::{
        bitcoin::{
            PublicKey,
            secp256k1::{
                Message,
                Secp256k1,
                Signature,
            },
            util::{
                bip32::DerivationPath,
                psbt::PartiallySignedTransaction,
            },
        },
        signer::TransactionSigner,
    },
    log::error,
    secrecy::Secret,
    Result,
    Error,
    contract::Contract,
    payout::Payout,
    wallet::{
        derive_account_xprivkey,
        EscrowWallet,
        SavedSeed,
        SigningWallet,
    },
    mock::{
        ARBITER_MNEMONIC,
        ESCROW_KIX,
        ESCROW_SUBACCOUNT,
        NETWORK,
    }
};

pub struct Wallet {
    saved_seed: SavedSeed,
}

impl Wallet {
    pub fn new(pw: Secret<String>) -> Self {
       Wallet {
           saved_seed: SavedSeed::new(pw, Some(Secret::new(ARBITER_MNEMONIC.to_owned()))).unwrap()
       } 
    }

    pub fn sign_payout(&self, payout: Payout, pw: Secret<String>) -> Result<PartiallySignedTransaction> {
// derive escrow private key
        let path = DerivationPath::from_str(&format!("m/{}/{}", ESCROW_SUBACCOUNT, ESCROW_KIX)).unwrap();
        let seed = self.saved_seed.get_seed(pw)?;
        let account_key = derive_account_xprivkey(seed, NETWORK);
        let secp = Secp256k1::new();
        let escrow_privkey = account_key.derive_priv(&secp, &path).unwrap().private_key;
// create escrow descriptor wallet
        let desc = format!("wsh(multi(2,{},{},{}))", 
                payout.contract.p1_pubkey.to_string(), 
                payout.contract.p2_pubkey.to_string(), 
                escrow_privkey.to_wif());
        let wallet = tglib::bdk::Wallet::new_offline(&desc, None, NETWORK, tglib::bdk::database::MemoryDatabase::default()).unwrap();

        let (psbt, _finalized) = wallet.sign(payout.psbt, None).unwrap();
        
        Ok(psbt)
    }
}

impl SigningWallet for Wallet {
    fn sign_tx(&self, psbt: PartiallySignedTransaction, _path: Option<DerivationPath>, pw: Secret<String>) -> Result<PartiallySignedTransaction> {
        let secp = Secp256k1::new();
        let path = DerivationPath::from_str(&String::from(format!("m/{}/{}", ESCROW_SUBACCOUNT, ESCROW_KIX))).unwrap();
        let account_key = derive_account_xprivkey(self.saved_seed.get_seed(pw)?, NETWORK);
        let escrow_key = account_key.derive_priv(&secp, &path).unwrap();
        let mut maybe_signed = psbt.clone();
        match &escrow_key.private_key.sign_tx(&mut maybe_signed, &secp) {
            Ok(()) => {
                Ok(maybe_signed)
            }
            Err(e) => {
                error!("{}", e);
                Err(Error::Adhoc("cannot sign transaction"))
            }
        }
    }

    fn sign_message(&self, msg: Message, path: DerivationPath, pw: Secret<String>) -> Result<Signature> {
        let account_key = derive_account_xprivkey(self.saved_seed.get_seed(pw)?, NETWORK);
        let secp = Secp256k1::new();
        let signing_key = account_key.derive_priv(&secp, &path).unwrap();
        Ok(secp.sign(&msg, &signing_key.private_key.key))
    }
}

impl EscrowWallet for Wallet {
    fn get_escrow_pubkey(&self) -> PublicKey {
        let path = DerivationPath::from_str(&String::from(format!("m/{}/{}", ESCROW_SUBACCOUNT, ESCROW_KIX))).unwrap();
        let escrow_pubkey = self.saved_seed.xpubkey.derive_pub(&Secp256k1::new(), &path).unwrap();
        escrow_pubkey.public_key
    }

    fn validate_contract(&self, contract: &Contract) -> Result<()> {
// TODO: better fee validation
        if contract.arbiter_pubkey != self.get_escrow_pubkey() {
            error!("incorrect arbiter pubkey");
            return Err(Error::Adhoc("incorrect arbiter pubkey"));
        }
        contract.validate()
    }
}
