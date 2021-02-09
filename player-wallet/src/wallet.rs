use std::str::FromStr;
use tglib::{
    bdk::{
        bitcoin::{
            Address,
            Amount,
            Network,
            PublicKey,
            Transaction,
            TxIn,
            TxOut,
            Script,
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
        blockchain::{
            noop_progress,
            ElectrumBlockchain,
        },
        database::MemoryDatabase,
        electrum_client::Client,
        signer::Signer,
        Wallet,
    },
    secrecy::Secret,
    Result as TgResult,
    TgError,
    arbiter::ArbiterService,
    contract::{
        Contract,
        PlayerContractInfo,
    },
    wallet::{
        create_escrow_address,
        create_payout_script,
        derive_account_xprivkey,
        EscrowWallet,
        NameWallet,
        SavedSeed,
        SigningWallet,
        BITCOIN_ACCOUNT_PATH,
        NAME_SUBACCOUNT,
        NAME_KIX,
    },
    mock::{
        ARBITER_PUBLIC_URL,
        ESCROW_SUBACCOUNT,
        ESCROW_KIX,
        NETWORK,
    },
};
use crate::{
    player::PlayerNameClient,
    arbiter::ArbiterClient,
    db::DB,
};
pub struct PlayerWallet {
//    xpubkey: ExtendedPubKey,
    seed: SavedSeed,
    network: Network,
// TODO: allow for offline wallet
    pub wallet: Wallet<ElectrumBlockchain, MemoryDatabase>,
    pub db: DB,
    pub name_client: PlayerNameClient,
    pub arbiter_client: ArbiterClient,
}

impl PlayerWallet {
    pub fn new(seed: SavedSeed, db: DB, network: Network, electrum_client: Client, name_client: PlayerNameClient, arbiter_client: ArbiterClient) ->  Self {
        let descriptor_key = format!("[{}/{}]{}", seed.fingerprint, BITCOIN_ACCOUNT_PATH, seed.xpubkey);
        let external_descriptor = format!("wpkh({}/0/*)", descriptor_key);
        let internal_descriptor = format!("wpkh({}/1/*)", descriptor_key);
//        let mut db_path = db_path.unwrap_or(current_dir().unwrap());
//        let mut db_path = current_dir().unwrap();
// TODO: need to use the right path depending on platform
//        if db_path == PathBuf::from("/") {
// we're probably on android
// TODO: append --db-path arg in java using android API instead
//            db_path.push("data/data/com.playerapp/files/");
//        }
//        db_path.push(DB_NAME);
//        let db = DB::new(&db_path).unwrap();
//        let _r = db.create_tables().unwrap();

        PlayerWallet {
//            xpubkey,
            seed,
            network,
            wallet: Wallet::new(
                &external_descriptor,
                Some(&internal_descriptor),
                network,
                MemoryDatabase::default(),
                ElectrumBlockchain::from(electrum_client)
            ).unwrap(),
            db,
            name_client,
            arbiter_client,
        }
    }

    pub fn balance(&self) -> Amount {
        self.wallet.sync(noop_progress(), None).unwrap();
        Amount::from_sat(self.wallet.get_balance().unwrap())
    }

    pub fn new_address(&self) -> Address {
        self.wallet.get_new_address().unwrap()
    }

    pub fn create_contract(&self, p2_contract_info: PlayerContractInfo, amount: Amount, arbiter_pubkey: PublicKey ) -> Contract {

        let p1_pubkey = self.get_escrow_pubkey();
        let escrow_address = create_escrow_address(&p1_pubkey, &p2_contract_info.escrow_pubkey, &arbiter_pubkey, self.network).unwrap();
        let funding_tx = self.create_funding_tx(&p2_contract_info, amount, &escrow_address);
        let payout_script = create_payout_script(&p1_pubkey, &p2_contract_info.escrow_pubkey, &arbiter_pubkey, &funding_tx, self.network);

        Contract::new(
            p1_pubkey,
            p2_contract_info.escrow_pubkey,
            arbiter_pubkey,
            PartiallySignedTransaction::from_unsigned_tx(funding_tx).unwrap(),
            payout_script,
        )
    }

    fn create_funding_tx(&self, p2_contract_info: &PlayerContractInfo, amount: Amount, escrow_address: &Address) -> Transaction {
        let mut input = Vec::new();
        let arbiter_fee = amount.as_sat()/100;
        let sats_per_player = (amount.as_sat() + arbiter_fee)/2;
        let mut total: u64 = 0;

        assert_ne!(p2_contract_info.utxos.len(), 0);
        for utxo in &p2_contract_info.utxos {
            if total > sats_per_player {
                break
            } else {
                total += utxo.txout.value;
                input.push(TxIn{
                    previous_output: utxo.outpoint, 
                    script_sig: Script::new(),
                    sequence: 0,
                    witness: Vec::new(),
                });
            }
        }
        assert!(total > sats_per_player);
        let p2_change = total - sats_per_player;
        assert!(self.wallet.sync(noop_progress(), None).is_ok());
        for utxo in self.wallet.list_unspent().unwrap() {
            if total > 2 * sats_per_player + p2_change {
                break
            }
            else {
                total += utxo.txout.value;
                input.push(TxIn{
                    previous_output: utxo.outpoint, 
                    script_sig: Script::new(),
                    sequence: 0,
                    witness: Vec::new(),
                });
            }
        }
        assert!(total > 2 * sats_per_player + p2_change);
        let p1_change = total - 2 * sats_per_player - p2_change;

        let arbiter_client = ArbiterClient::new(ARBITER_PUBLIC_URL);
        let fee_address = arbiter_client.get_fee_address().unwrap();

        let output = vec!(
            TxOut { 
                value: amount.as_sat(),
                script_pubkey: escrow_address.script_pubkey(),
            },
            TxOut { 
                value: arbiter_fee,
                script_pubkey: fee_address.script_pubkey(),
            },
            TxOut { 
                value: p2_change, 
                script_pubkey: p2_contract_info.change_address.script_pubkey(),
            },
            TxOut { 
                value: p1_change, 
                script_pubkey: self.new_address().script_pubkey(),
            },
        );

        Transaction {
            version: 1,
            lock_time: 0,
            input,
            output,
        }
    }
}

impl NameWallet for PlayerWallet {
    fn name_pubkey(&self) -> PublicKey {
        let secp = Secp256k1::new();
        let path = DerivationPath::from_str(&String::from(format!("m/{}/{}", NAME_SUBACCOUNT, NAME_KIX))).unwrap();
        let pubkey = self.seed.xpubkey.derive_pub(&secp, &path).unwrap();
        pubkey.public_key
    }
}

impl EscrowWallet for PlayerWallet {
    fn get_escrow_pubkey(&self) -> PublicKey {
        let secp = Secp256k1::new();
        let path = DerivationPath::from_str(&String::from(format!("m/{}/{}", ESCROW_SUBACCOUNT, ESCROW_KIX))).unwrap();
        let escrow_pubkey = self.seed.xpubkey.derive_pub(&secp, &path).unwrap();
        escrow_pubkey.public_key
    }

    fn validate_contract(&self, contract: &Contract) -> TgResult<()> {
        let player_pubkey = self.get_escrow_pubkey();
        if contract.p1_pubkey != player_pubkey && contract.p2_pubkey != player_pubkey {
            return Err(TgError("contract doesn't contain our pubkey".to_string()));
        }
        let arbiter_client = ArbiterClient::new(ARBITER_PUBLIC_URL);
        let arbiter_pubkey = arbiter_client.get_escrow_pubkey().unwrap();
        if contract.arbiter_pubkey != arbiter_pubkey {
            return Err(TgError("unexpected arbiter pubkey".to_string()));
        }
        contract.validate()
    }
}

impl SigningWallet for PlayerWallet {

    fn sign_tx(&self, psbt: PartiallySignedTransaction, _kdp: String, pw: Secret<String>) -> TgResult<PartiallySignedTransaction> {
        let secp = Secp256k1::new();
        let path = DerivationPath::from_str(&String::from(format!("m/{}/{}", ESCROW_SUBACCOUNT, ESCROW_KIX))).unwrap();
// TODO: decrypt seed with pw
        let account_key = derive_account_xprivkey(&self.seed.encrypted_seed, NETWORK);
        let escrow_key = account_key.derive_priv(&secp, &path).unwrap();
        let mut maybe_signed = psbt.clone();
//        println!("psbt to sign: {:?}", psbt);
//        match Signer::sign(&escrow_key.private_key, &mut maybe_signed, Some(0)) {
        match Signer::sign(&escrow_key.private_key, &mut maybe_signed, Some(0), &secp) {
            Ok(()) => {
                Ok(maybe_signed)
            }
            Err(e) => {
                println!("err: {:?}", e);
                Err(TgError("cannot sign transaction".to_string()))
            }
        }
    }

    fn sign_message(&self, msg: Message, path: DerivationPath, pw: Secret<String>) -> TgResult<Signature> {
        let account_key = derive_account_xprivkey(&self.seed.encrypted_seed, NETWORK);
        let secp = Secp256k1::new();
        let signing_key = account_key.derive_priv(&secp, &path).unwrap();
        Ok(secp.sign(&msg, &signing_key.private_key.key))
    }
}
