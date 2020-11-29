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
    script::{
        TgScriptEnv,
    },
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
        MockWallet,
        Trezor,
        ELECTRS_SERVER,
        PLAYER_2_MNEMONIC,
        NETWORK,
    }
};

const ESCROW_KIX: u64 = 0;

pub struct Wallet<B, D> where B: BlockchainMarker, D: BatchDatabase {
    fingerprint: Fingerprint,
    xpubkey: ExtendedPubKey,
    network: Network,
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

    pub fn validate_contract(&self, contract: &Contract) -> Result<()> {
        if contract.arbiter_pubkey != EscrowWallet::get_escrow_pubkey(self) {
            return Err(TgError("unexpected arbiter pubkey"));
        }
        let escrow_address = create_escrow_address(
            &contract.p1_pubkey,
            &contract.p2_pubkey,
            &contract.arbiter_pubkey,
            NETWORK,
        ).unwrap();
        let escrow_script_pubkey = escrow_address.script_pubkey();
        let amount = contract.amount()?;
        let fee_address = self.get_fee_address()?;
        let fee_script_pubkey = fee_address.script_pubkey();
        let fee_amount = amount/100;

        let mut correct_fee = false;

        for txout in contract.funding_tx.clone().output {
            correct_fee = (txout.script_pubkey == fee_script_pubkey && txout.value == fee_amount.as_sat()) || correct_fee; 
        }

        if !correct_fee {
            return Err(TgError("funding transaction does not contain correct fee"));
        }

        let payout_script = create_payout_script(
            &contract.p1_pubkey,
            &contract.p2_pubkey,
            &contract.arbiter_pubkey,
            &contract.funding_tx,
            NETWORK,
        );

        if contract.payout_script != payout_script {
            return Err(TgError("invalid payout script"));
        }

        let secp = Secp256k1::new();
        let msg = Message::from_slice(&contract.cxid()).unwrap();

        for (i, sig) in contract.sigs.iter().enumerate() {
            let pubkey = match i {
                0 => &contract.p1_pubkey.key,
                1 => &contract.p2_pubkey.key,
                2 => &contract.arbiter_pubkey.key,
                _ => return Err(TgError("too many signatures")),
            };
            if secp.verify(&msg, &sig, &pubkey).is_err() {
                return Err(TgError("invalid signature"))
            }
        }
        Ok(())
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
}

impl<B, D> ArbiterService for Wallet<B, D> 
where 
    B: BlockchainMarker,
    D: BatchDatabase + Default,
{
    fn get_escrow_pubkey(&self) -> Result<PublicKey> {
        Ok(EscrowWallet::get_escrow_pubkey(self))
    }

    fn get_fee_address(&self) -> Result<Address> {
        let a = self.xpubkey.derive_pub(&Secp256k1::new(), &DerivationPath::from_str("m/0/0").unwrap()).unwrap();
        Ok(Address::p2wpkh(&a.public_key, self.network).unwrap())
    }

    fn submit_contract(&self, _contract: &Contract) -> Result<Signature> {
        Err(TgError("invalid contract"))
    }

    fn submit_payout(&self, _payout: &Payout) -> Result<Transaction> {
        Err(TgError("invalid payout"))
    }

    fn get_player_info(&self, playerId: PlayerId) -> Result<PlayerContractInfo> {
        let signing_wallet = Trezor::new(Mnemonic::parse(PLAYER_2_MNEMONIC).unwrap());
        let player_wallet = MockWallet::new(signing_wallet.fingerprint(), signing_wallet.xpubkey(), NETWORK);
        player_wallet.wallet.sync(noop_progress(), None).unwrap();
        let escrow_pubkey = player_wallet.get_escrow_pubkey();
        Ok(PlayerContractInfo {
            escrow_pubkey,
// TODO: send to internal descriptor, no immediate way to do so atm
            change_address: player_wallet.wallet.get_new_address().unwrap(),
            utxos: player_wallet.wallet.list_unspent().unwrap(),
        })
    }
}