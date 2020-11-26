use std::{
    str::FromStr,
    env::current_dir,
};
use bdk::{
    Wallet as BdkWallet,
    database::MemoryDatabase,
    electrum_client::Client,
    bitcoin::{
        Address,
        Network,
        PublicKey,
        Transaction,
        secp256k1::{
            Secp256k1,
            Signature,
        },
        util::{
            bip32::{
                ExtendedPubKey,
                DerivationPath,
                Fingerprint,
            },
        }
    },
    blockchain::{
        ElectrumBlockchain,
    },
};
use tglib::{
    Result,
    TgError,
    arbiter::ArbiterService,
    contract::Contract,
    payout::Payout,
    wallet::{
        EscrowWallet,
        BITCOIN_ACCOUNT_PATH,
        ESCROW_SUBACCOUNT,
    },
    mock::{
        ELECTRS_SERVER,
    }
};

const ESCROW_KIX: u64 = 0;

pub struct Wallet {
    fingerprint: Fingerprint,
    xpubkey: ExtendedPubKey,
    network: Network,
    escrow_kix: u64,
    pub wallet: BdkWallet<ElectrumBlockchain, MemoryDatabase>,
}

impl Wallet {
    pub fn new(fingerprint: Fingerprint, xpubkey: ExtendedPubKey, network: Network) -> Self {
        let descriptor_key = format!("[{}/{}]{}", fingerprint, BITCOIN_ACCOUNT_PATH, xpubkey);
        let external_descriptor = format!("wpkh({}/0/*)", descriptor_key);
        let internal_descriptor = format!("wpkh({}/1/*)", descriptor_key);
        let client = Client::new(ELECTRS_SERVER, None).unwrap();
        let mut db_path = current_dir().unwrap();

        Wallet {
            fingerprint,
            xpubkey,
            network,
            wallet: BdkWallet::new(
                &external_descriptor,
                Some(&internal_descriptor),
                network,
                MemoryDatabase::default(),
                ElectrumBlockchain::from(client)
            ).unwrap(),
            escrow_kix: ESCROW_KIX,
        }
    }
}

impl EscrowWallet for Wallet {
    fn get_escrow_pubkey(&self) -> PublicKey {
        let secp = Secp256k1::new();
        let path = DerivationPath::from_str(&String::from(format!("m/{}/{}", ESCROW_SUBACCOUNT, self.escrow_kix))).unwrap();
        let escrow_pubkey = self.xpubkey.derive_pub(&secp, &path).unwrap();
        escrow_pubkey.public_key
    }
}

impl ArbiterService for Wallet {
    fn get_escrow_pubkey(&self) -> Result<PublicKey> {
        Ok(EscrowWallet::get_escrow_pubkey(self))
    }

    fn get_fee_address(&self) -> Result<Address> {
        Ok((self.wallet.get_new_address().unwrap()))
    }

    fn submit_contract(&self, contract: &Contract) -> Result<Signature> {
        Err(TgError("invalid contract"))
    }

    fn submit_payout(&self, payout: &Payout) -> Result<Transaction> {
        Err(TgError("invalid payout"))
    }
}
