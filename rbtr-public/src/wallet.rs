use std::str::FromStr;
use tglib::{
    bdk::{
        Wallet as BdkWallet,
        database::BatchDatabase,
        bitcoin::{
            Network,
            PublicKey,
            secp256k1::Secp256k1,
            util::bip32::{
                ExtendedPubKey,
                DerivationPath,
                Fingerprint,
            }
        },
        blockchain::{
            Blockchain,
            BlockchainMarker,
        },
    },
    Result,
    TgError,
    contract::Contract,
    wallet::{
        EscrowWallet,
        BITCOIN_ACCOUNT_PATH,
        ESCROW_SUBACCOUNT,
    },
};

const ESCROW_KIX: u64 = 0;

pub struct Wallet<B, D> where B: BlockchainMarker, D: BatchDatabase {
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
    
    #[allow(dead_code)]
    pub fn new_offline(fingerprint: Fingerprint, xpubkey: ExtendedPubKey, network: Network) -> Self {
        let descriptor_key = format!("[{}/{}]{}", fingerprint, BITCOIN_ACCOUNT_PATH, xpubkey);
        let external_descriptor = format!("wpkh({}/0/*)", descriptor_key);
        let internal_descriptor = format!("wpkh({}/1/*)", descriptor_key);

        Wallet {
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
            return Err(TgError("unexpected arbiter pubkey".to_string()));
        }
        contract.validate()
    }
}
