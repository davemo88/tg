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
use sled;
use tglib::{
    Result,
    TgError,
    arbiter::ArbiterService,
    contract::{
        Contract,
        PlayerContractInfo,
    },
    payout::Payout,
    player::PlayerId,
    script::TgScriptEnv,
    wallet::{
        create_escrow_address,
        create_payout_script,
        create_payout,
        EscrowWallet,
        SigningWallet,
        BITCOIN_ACCOUNT_PATH,
        ESCROW_SUBACCOUNT,
    },
    mock::{
        Trezor,
        ELECTRS_SERVER,
        PLAYER_2_MNEMONIC,
        NETWORK,
    }
};

const ESCROW_KIX: u64 = 0;

pub struct Wallet<B, D> where B: BlockchainMarker, D: BatchDatabase {
    fingerprint: Fingerprint,
    pub xpubkey: ExtendedPubKey,
    pub network: Network,
    escrow_kix: u64,
    pub wallet: BdkWallet<B, D>,
}

impl<B, D> Wallet<B, D> 
where 
    B: BlockchainMarker + Blockchain,
    D: BatchDatabase + Default,
{
    pub fn new(fingerprint: Fingerprint, xpubkey: ExtendedPubKey, client: B, network: Network) -> Result<Self> {
        let descriptor_key = format!("[{}/{}]{}", fingerprint, BITCOIN_ACCOUNT_PATH, xpubkey);
        let external_descriptor = format!("wpkh({}/0/*)", descriptor_key);
        let internal_descriptor = format!("wpkh({}/1/*)", descriptor_key);

        Ok(Wallet {
            fingerprint,
            xpubkey,
            network,
            wallet: BdkWallet::new(
                &external_descriptor,
                Some(&internal_descriptor),
                network,
//                sled::open("wallet-db").unwrap().open_tree("wallet-db").unwrap(),
                D::default(),
                client,
            ).unwrap(),
            escrow_kix: ESCROW_KIX,
        })
    }
    
    pub fn new_offline(fingerprint: Fingerprint, xpubkey: ExtendedPubKey, network: Network) -> Self {
        let descriptor_key = format!("[{}/{}]{}", fingerprint, BITCOIN_ACCOUNT_PATH, xpubkey);
        let external_descriptor = format!("wpkh({}/0/*)", descriptor_key);
        let internal_descriptor = format!("wpkh({}/1/*)", descriptor_key);
        let client = Client::new(ELECTRS_SERVER, None).unwrap();

        Wallet {
            fingerprint,
            xpubkey,
            network,
            wallet: BdkWallet::new_offline(
                &external_descriptor,
                Some(&internal_descriptor),
                network,
                D::default(),
            ).unwrap(),
            escrow_kix: ESCROW_KIX,
        }
    }

    pub fn validate_payout(&self, payout: &Payout) -> Result<()> {
        if self.validate_contract(&payout.contract).is_ok() {
            if payout.tx.txid() != create_payout(&payout.contract, &payout.address().unwrap()).tx.txid() {
                return Err(TgError("invalid payout"));
            }
            let mut env = TgScriptEnv::new(payout.clone());
            return env.validate_payout()
        }
        Err(TgError("invalid payout"))
    }
}

impl<B, D> EscrowWallet for Wallet<B, D>
where 
    B: BlockchainMarker,
    D: BatchDatabase + Default,
{
    fn get_escrow_pubkey(&self) -> PublicKey {
        let path = DerivationPath::from_str(&String::from(format!("m/{}/{}", ESCROW_SUBACCOUNT, self.escrow_kix))).unwrap();
        let escrow_pubkey = self.xpubkey.derive_pub(&Secp256k1::new(), &path).unwrap();
        escrow_pubkey.public_key
    }

    fn validate_contract(&self, contract: &Contract) -> Result<()> {
        if contract.arbiter_pubkey != EscrowWallet::get_escrow_pubkey(self) {
            return Err(TgError("unexpected arbiter pubkey"));
        }
        contract.validate()
    }
}
