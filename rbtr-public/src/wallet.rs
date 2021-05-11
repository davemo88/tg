use std::str::FromStr;
use tglib::{
    bdk::{
        Wallet as BdkWallet,
        database::BatchDatabase,
        bitcoin::{
            Address,
            Network,
            PublicKey,
            secp256k1::Secp256k1,
            util::bip32::{
                ExtendedPubKey,
                DerivationPath,
                Fingerprint,
            }
        },
        blockchain::Blockchain,
    },
    log::error,
    Result,
    Error,
    contract::Contract,
    wallet::{
        EscrowWallet,
        BITCOIN_ACCOUNT_PATH,
        ESCROW_SUBACCOUNT,
    },
};

const ESCROW_KIX: u64 = 0;

pub struct Wallet<B, D> where D: BatchDatabase {
    pub xpubkey: ExtendedPubKey,
    pub network: Network,
    pub wallet: BdkWallet<B, D>,
}

impl<B, D> Wallet<B, D> 
where 
    B: Blockchain,
    D: BatchDatabase + Default,
{
    pub fn new(fingerprint: Fingerprint, xpubkey: ExtendedPubKey, client: B, network: Network) -> Self {
        let descriptor_key = format!("[{}/{}]{}", fingerprint, BITCOIN_ACCOUNT_PATH, xpubkey);
        let external_descriptor = format!("wpkh({}/0/*)", descriptor_key);
        let internal_descriptor = format!("wpkh({}/1/*)", descriptor_key);

        Wallet {
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
        }
    }
}

impl<B, D> Wallet<B, D>
where
    D: BatchDatabase + Default, 
{
    pub fn get_fee_address(&self) -> Address {
// TODO: this should come from a separate wallet and the address hosted statically
        let a = self.xpubkey.derive_pub(&Secp256k1::new(), &DerivationPath::from_str("m/0/0").unwrap()).unwrap();
        Address::p2wpkh(&a.public_key, self.network).unwrap()
    }
}

impl<D> Wallet<(),D>
where
    D: BatchDatabase + Default 
{
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
        }
    }
}

impl<B, D> EscrowWallet for Wallet<B, D>
where 
    D: BatchDatabase + Default,
{
    fn get_escrow_pubkey(&self) -> PublicKey {
        let path = DerivationPath::from_str(&String::from(format!("m/{}/{}", ESCROW_SUBACCOUNT, ESCROW_KIX))).unwrap();
        let escrow_pubkey = self.xpubkey.derive_pub(&Secp256k1::new(), &path).unwrap();
        escrow_pubkey.public_key
    }

    fn validate_contract(&self, contract: &Contract) -> Result<()> {
        if contract.arbiter_pubkey != self.get_escrow_pubkey() {
            let e = Error::Adhoc("unexpected arbiter pubkey");
            error!("{}", e);
            return Err(e);
        }
        if contract.fee_address()? != self.get_fee_address() {
            let e = Error::Adhoc("incorrect fee address");
            error!("{}", e);
            return Err(e);
        }
        contract.validate()
    }
}
