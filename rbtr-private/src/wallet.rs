use std::{
    str::FromStr,
};
use bdk::{
    Wallet as BdkWallet,
    database::{
        BatchDatabase,
        MemoryDatabase,
    },
    electrum_client::Client,
    bitcoin::{
        Address,
        Network,
        PublicKey,
        Transaction,
        consensus,
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
    blockchain::{
        noop_progress,
        Blockchain,
        BlockchainMarker,
        ElectrumBlockchain,
    },
};
use bip39::Mnemonic;
use tglib::{
    Result as TgResult,
    TgError,
    contract::Contract,
    payout::Payout,
    script::TgScriptEnv,
    wallet::{
        create_payout,
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

    pub fn validate_payout(&self, payout: &Payout) -> TgResult<()> {
        if self.validate_contract(&payout.contract).is_ok() {
            if payout.tx.txid() != create_payout(&payout.contract, &payout.address().unwrap()).tx.txid() {
                return Err(TgError("invalid payout"));
            }
            let mut env = TgScriptEnv::new(payout.clone());
            return env.validate_payout()
        }
        Err(TgError("invalid payout"))
    }

    pub fn sign_contract(&self, contract: &Contract) -> TgResult<Signature> {
        Ok(self.sign_message(Message::from_slice(&contract.cxid()).unwrap(), 
                    DerivationPath::from_str(&format!("m/{}/{}", ESCROW_SUBACCOUNT, ESCROW_KIX)).unwrap()).unwrap())
    }

    pub fn sign_payout(&self, payout: &Payout) -> TgResult<Transaction> {
        let psbt: PartiallySignedTransaction = consensus::deserialize(&consensus::serialize(&payout.tx)).unwrap();
        self.sign_tx(psbt, "".to_string())
    }
}

impl SigningWallet for Wallet {
    fn fingerprint(&self) -> Fingerprint {
        self.trezor.fingerprint()
    }

    fn xpubkey(&self) -> ExtendedPubKey {
        self.trezor.xpubkey()
    }

    fn sign_tx(&self, pstx: PartiallySignedTransaction, descriptor: String) -> TgResult<Transaction> {
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
