use std::{
    str::FromStr,
    path::PathBuf,
    fs::File,
};
use sled;
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
        database::AnyDatabase,
        electrum_client::Client as ElectrumClient,
        signer::Signer,
        Wallet,
    },
    secrecy::Secret,
    Result as TgResult,
    TgError,
    arbiter::ArbiterService,
    contract::{
        Contract,
        ContractRecord,
        PlayerContractInfo,
    },
    player::PlayerName,
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
        DB_NAME,
        ESCROW_SUBACCOUNT,
        ESCROW_KIX,
        SEED_NAME,
        WALLET_DB_NAME,
        WALLET_TREE_NAME,
    },
};
use crate::{
    player::PlayerNameClient,
    arbiter::ArbiterClient,
    db::DB,
    ui::PlayerUI,
};

pub struct PlayerWallet {
//    seed: SavedSeed,
    wallet_dir: PathBuf,
    network: Network,
//    pub db: DB,
    pub electrum_url: String,
    pub name_url: String,
    pub arbiter_url: String,
}

impl PlayerWallet {
//    pub fn new(seed: SavedSeed, wallet_db: sled::Tree, db: DB, network: Network, electrum_client: ElectrumClient, name_client: PlayerNameClient, arbiter_client: ArbiterClient) ->  Self {
    pub fn new(wallet_dir: PathBuf, network: Network, electrum_url: String, name_url: String, arbiter_url: String) ->  Self {
//        let descriptor_key = format!("[{}/{}]{}", seed.fingerprint, BITCOIN_ACCOUNT_PATH, seed.xpubkey);
//        let external_descriptor = format!("wpkh({}/0/*)", descriptor_key);
//        let internal_descriptor = format!("wpkh({}/1/*)", descriptor_key);
        PlayerWallet {
//            seed,
//            network,
//            wallet: Wallet::new(
//                &external_descriptor,
//                Some(&internal_descriptor),
//                network,
//                AnyDatabase::Sled(wallet_db),
//                ElectrumBlockchain::from(electrum_client)
//            ).unwrap(),
            wallet_dir,
            network,
            electrum_url,
            name_url,
            arbiter_url,
        }
    }

    fn saved_seed(&self) -> TgResult<SavedSeed> {
        let mut seed_path = self.wallet_dir.clone();
        seed_path.push(SEED_NAME);
        match File::open(&seed_path) {
            Ok(reader) => Ok(serde_json::from_reader(reader).unwrap()),
            Err(e) => Err(TgError(format!("{:?}", e))),
        }
    }

    pub fn wallet(&self) -> Wallet<ElectrumBlockchain, AnyDatabase> {
        let saved_seed = self.saved_seed().unwrap();
        let descriptor_key = format!("[{}/{}]{}", saved_seed.fingerprint, BITCOIN_ACCOUNT_PATH, saved_seed.xpubkey);

        Wallet::new(
            &self.external_descriptor(&descriptor_key),
            Some(&self.internal_descriptor(&descriptor_key)),
            self.network,
            AnyDatabase::Sled(self.wallet_db()),
            ElectrumBlockchain::from(ElectrumClient::new(&self.electrum_url).unwrap())
        ).unwrap()
    }

    fn signing_wallet(&self, pw: Secret<String>) -> Wallet<ElectrumBlockchain, AnyDatabase> {
        let saved_seed = self.saved_seed().unwrap();
        let seed = saved_seed.get_seed(pw).unwrap();
        let account_key = derive_account_xprivkey(seed, self.network);
        let descriptor_key = format!("[{}/{}]{}", saved_seed.fingerprint, BITCOIN_ACCOUNT_PATH, account_key);

        Wallet::new(
            &self.external_descriptor(&descriptor_key),
            Some(&self.internal_descriptor(&descriptor_key)),
            self.network,
            AnyDatabase::Sled(self.wallet_db()),
            ElectrumBlockchain::from(ElectrumClient::new(&self.electrum_url).unwrap())
        ).unwrap()
    }

    fn wallet_db(&self) -> sled::Tree {
        let mut wallet_db_path = self.wallet_dir.clone();  
        wallet_db_path.push(WALLET_DB_NAME);
        sled::open(wallet_db_path).unwrap().open_tree(WALLET_TREE_NAME).unwrap()
    }

    fn external_descriptor(&self, descriptor_key: &str) -> String {
        format!("wpkh({}/0/*)", descriptor_key)
    }

    fn internal_descriptor(&self, descriptor_key: &str) -> String {
        format!("wpkh({}/1/*)", descriptor_key)
    }

    pub fn name_client(&self) -> PlayerNameClient {
        PlayerNameClient::new(&self.name_url)
    }

    pub fn arbiter_client(&self) -> ArbiterClient {
        ArbiterClient::new(&self.arbiter_url)
    }

    pub fn db(&self) -> DB {
        let mut db_path = self.wallet_dir.clone();
        db_path.push(DB_NAME);
        let db = DB::new(&db_path).unwrap();
        let _r = db.create_tables().unwrap();
        db
    }

    pub fn balance(&self) -> Amount {
        self.wallet().sync(noop_progress(), None).unwrap();
        Amount::from_sat(self.wallet().get_balance().unwrap())
    }

    pub fn new_address(&self) -> Address {
        self.wallet().get_new_address().unwrap()
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
        assert!(self.wallet().sync(noop_progress(), None).is_ok());
        for utxo in self.wallet().list_unspent().unwrap() {
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

    pub fn get_other_player_name(&self, contract_record: &ContractRecord) -> TgResult<PlayerName> {
        let my_players = self.mine();
        if my_players.contains(&contract_record.p1_name) {
            Ok(contract_record.p2_name.clone())
        } else if my_players.contains(&contract_record.p2_name) {
            Ok(contract_record.p1_name.clone())
        } else {
            Err(TgError("not party to this contract".to_string()))
        }
    }
}

impl NameWallet for PlayerWallet {
    fn name_pubkey(&self) -> PublicKey {
        let secp = Secp256k1::new();
        let path = DerivationPath::from_str(&String::from(format!("m/{}/{}", NAME_SUBACCOUNT, NAME_KIX))).unwrap();
        let pubkey = self.saved_seed().unwrap().xpubkey.derive_pub(&secp, &path).unwrap();
        pubkey.public_key
    }
}

impl EscrowWallet for PlayerWallet {
    fn get_escrow_pubkey(&self) -> PublicKey {
        let secp = Secp256k1::new();
        let path = DerivationPath::from_str(&String::from(format!("m/{}/{}", ESCROW_SUBACCOUNT, ESCROW_KIX))).unwrap();
        let escrow_pubkey = self.saved_seed().unwrap().xpubkey.derive_pub(&secp, &path).unwrap();
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

    fn sign_tx(&self, psbt: PartiallySignedTransaction, path: Option<DerivationPath>, pw: Secret<String>) -> TgResult<PartiallySignedTransaction> {
        let seed = match self.saved_seed().unwrap().get_seed(pw.clone()) {
            Ok(seed) => seed,
            Err(e) => return Err(TgError(format!("{:?}", e))),
        };
        let account_key = derive_account_xprivkey(seed, self.network);
        match path {
            Some(path) => {
                let secp = Secp256k1::new();
                let signing_key = account_key.derive_priv(&secp, &path).unwrap();
                let mut maybe_signed = psbt.clone();
//                println!("psbt to sign: {:?}", psbt);
                match Signer::sign(&signing_key.private_key, &mut maybe_signed, None, &secp) {
                    Ok(()) => {
                        Ok(maybe_signed)
                    }
                    Err(e) => {
                        println!("err: {:?}", e);
                        Err(TgError("cannot sign transaction".to_string()))
                    }
                }
            }
            None => {
                let signing_wallet = self.signing_wallet(pw);
                let (psbt, _b) = signing_wallet.sign(psbt.clone(),None).unwrap();
                Ok(psbt)
            },
        }
    }

    fn sign_message(&self, msg: Message, path: DerivationPath, pw: Secret<String>) -> TgResult<Signature> {
        let seed = match self.saved_seed().unwrap().get_seed(pw) {
            Ok(seed) => seed,
            Err(e) => return Err(TgError(format!("{:?}", e))),
        };
        let account_key = derive_account_xprivkey(seed, self.network);
        let secp = Secp256k1::new();
        let signing_key = account_key.derive_priv(&secp, &path).unwrap();
        Ok(secp.sign(&msg, &signing_key.private_key.key))
    }
}
