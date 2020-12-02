use std::str::FromStr;
use tglib::{
    bdk::bitcoin::{
        PublicKey,
        secp256k1::{
            Message,
            Secp256k1,
            Signature,
        },
        util::{
            bip32::{
                ExtendedPubKey,
                DerivationPath,
                Fingerprint,
            },
            psbt::PartiallySignedTransaction,
        }
    },
    bip39::Mnemonic,
    Result as TgResult,
    TgError,
    contract::Contract,
    wallet::{
        SigningWallet,
        EscrowWallet,
    },
    mock::{
        Trezor,
        ARBITER_MNEMONIC,
        ESCROW_KIX,
        ESCROW_SUBACCOUNT,
    }
};

pub struct Wallet {
    trezor: Trezor,    
}

impl Wallet {
    pub fn new() -> Self {
       Wallet {
           trezor: Trezor::new(Mnemonic::parse(ARBITER_MNEMONIC).unwrap())
       } 
    }
}

impl SigningWallet for Wallet {
    fn fingerprint(&self) -> Fingerprint {
        self.trezor.fingerprint()
    }

    fn xpubkey(&self) -> ExtendedPubKey {
        self.trezor.xpubkey()
    }

    fn sign_tx(&self, pstx: PartiallySignedTransaction, descriptor: String) -> TgResult<PartiallySignedTransaction> {
        self.trezor.sign_tx(pstx, descriptor)
    }

    fn sign_message(&self, msg: Message, path: DerivationPath) -> TgResult<Signature> {
    self.trezor.sign_message(msg, path)
    }
}

impl EscrowWallet for Wallet {
    fn get_escrow_pubkey(&self) -> PublicKey {
        let path = DerivationPath::from_str(&String::from(format!("m/{}/{}", ESCROW_SUBACCOUNT, ESCROW_KIX))).unwrap();
        let escrow_pubkey = self.xpubkey().derive_pub(&Secp256k1::new(), &path).unwrap();
        escrow_pubkey.public_key
    }

    fn validate_contract(&self, contract: &Contract) -> TgResult<()> {
// TODO: better fee validation
        if contract.arbiter_pubkey != self.get_escrow_pubkey() {
            return Err(TgError("unexpected arbiter pubkey"));
        }
        contract.validate()
    }
}
