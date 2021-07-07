use std::{
    str::FromStr,
    path::PathBuf,
    fs::File,
};
use libexchange::PlayerContractInfo;
use tglib::{
    bdk::{
        bitcoin::{
            Address,
            Amount,
            Network,
            PublicKey,
            Script,
            Transaction,
            TxIn,
            TxOut,
            blockdata::transaction::OutPoint,
            secp256k1::{
                Message,
                Secp256k1,
                Signature,
            },
            util::{
                bip32::DerivationPath,
                psbt::{
                    Input,
                    PartiallySignedTransaction,
                },
            },
        },
        blockchain::{
            noop_progress,
            ElectrumBlockchain,
        },
        electrum_client::Client as ElectrumClient,
        signer::TransactionSigner,
        Wallet,
    },
    secrecy::Secret,
    Result as TgResult,
    Error as TgError,
    arbiter::ArbiterService,
    contract::Contract,
    payout::Payout,
    player::PlayerName,
    wallet::{
        create_escrow_address,
        create_escrow_script,
        create_payout_script,
        create_payout_tx,
        create_token_pair_script,
        derive_account_xprivkey,
        EscrowWallet,
        NameWallet,
        SavedSeed,
        SigningWallet,
        BITCOIN_ACCOUNT_PATH,
        NAME_SUBACCOUNT,
        NAME_KIX,
        TX_FEE,
    },
    mock::{
        referee_pubkey,
        DB_NAME,
        ESCROW_SUBACCOUNT,
        ESCROW_KIX,
        NETWORK,
        SEED_NAME,
        WALLET_DB_NAME,
        WALLET_TREE_NAME,
    },
};
use crate::{
    Result,
    Error,
    Event,
    player::PlayerNameClient,
    arbiter::ArbiterClient,
    exchange::ExchangeClient,
    db::{
        ContractRecord,
        DB,
        TokenRecord,
    },
    ui::PlayerUI,
};

pub struct PlayerWallet {
    wallet_dir: PathBuf,
    network: Network,
    pub electrum_url: String,
    pub name_url: String,
    pub arbiter_url: String,
    pub exchange_url: String,
}

impl PlayerWallet {
    pub fn new(wallet_dir: PathBuf, network: Network, electrum_url: String, name_url: String, arbiter_url: String, exchange_url: String) ->  Self {
        PlayerWallet {
            wallet_dir,
            network,
            electrum_url,
            name_url,
            arbiter_url,
            exchange_url,
        }
    }

    fn saved_seed(&self) -> Result<SavedSeed> {
        let mut seed_path = self.wallet_dir.clone();
        seed_path.push(SEED_NAME);
        let reader = File::open(&seed_path)?;
        let mut seed: SavedSeed = serde_json::from_reader(reader).unwrap();
// serde json bug ? where regtest keys load as testnet
// maybe because they have the same string form
        if seed.xpubkey.network == Network::Testnet 
        && self.network == Network::Regtest {
            seed.xpubkey.network = Network::Regtest;
        }
        Ok(seed)
    }

    pub fn wallet(&self) -> Result<Wallet<ElectrumBlockchain, sled::Tree>> {
        let saved_seed = self.saved_seed().unwrap();
        let descriptor_key = format!("[{}/{}]{}", saved_seed.fingerprint, BITCOIN_ACCOUNT_PATH, saved_seed.xpubkey);

        let w = Wallet::new(
            &self.external_descriptor(&descriptor_key),
            Some(&self.internal_descriptor(&descriptor_key)),
            self.network,
            self.wallet_db(),
            ElectrumBlockchain::from(ElectrumClient::new(&self.electrum_url)?)
        ).unwrap();
        w.sync(noop_progress(), None).unwrap();
        Ok(w)
    }

    pub fn offline_wallet(&self) -> Wallet<(), sled::Tree> {
        let saved_seed = self.saved_seed().unwrap();
        let descriptor_key = format!("[{}/{}]{}", saved_seed.fingerprint, BITCOIN_ACCOUNT_PATH, saved_seed.xpubkey);

        Wallet::new_offline(
            &self.external_descriptor(&descriptor_key),
            Some(&self.internal_descriptor(&descriptor_key)),
            self.network,
            self.wallet_db(),
        ).unwrap()
    }

    fn signing_wallet(&self, pw: Secret<String>) -> TgResult<Wallet<ElectrumBlockchain, sled::Tree>> {
        let saved_seed = self.saved_seed().unwrap();
// can fail due to incorrect password
        let seed = saved_seed.get_seed(pw)?;
        let account_key = derive_account_xprivkey(seed, self.network);
        let descriptor_key = format!("[{}/{}]{}", saved_seed.fingerprint, BITCOIN_ACCOUNT_PATH, account_key);

        Ok(Wallet::new(
            &self.external_descriptor(&descriptor_key),
            Some(&self.internal_descriptor(&descriptor_key)),
            self.network,
            self.wallet_db(),
            ElectrumBlockchain::from(ElectrumClient::new(&self.electrum_url).unwrap())
        ).unwrap())
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

    pub fn exchange_client(&self) -> ExchangeClient {
        ExchangeClient::new(&self.exchange_url)
    }

    pub fn db(&self) -> DB {
        let mut db_path = self.wallet_dir.clone();
        db_path.push(DB_NAME);
        let db = DB::new(&db_path).unwrap();
        let _r = db.create_tables().unwrap();
        db
    }

    pub fn create_event_contract(&self, p1_name: &PlayerName, p2_name: &PlayerName, p2_contract_info: PlayerContractInfo, amount: Amount, arbiter_pubkey: PublicKey, event: &Event, event_payouts: &Vec<PlayerName>) -> Result<(Contract, Vec<TokenRecord>)> {
        if event.outcomes.len() != event_payouts.len() {
            return Err(Error::Adhoc("not enough payouts specified for event"))
        }
        let p1_pubkey = self.get_escrow_pubkey();
        let escrow_address = create_escrow_address(&p1_pubkey, &p2_contract_info.escrow_pubkey, &arbiter_pubkey, self.network).unwrap();
        let funding_tx = self.create_funding_tx(&p2_contract_info, amount, &escrow_address)?;
        let p1_payout_address = self.offline_wallet().get_new_address()?;
        let payout_txs: std::collections::HashMap<&PlayerName, Transaction> = 
            [
                (p1_name, create_payout_tx(&funding_tx.clone().extract_tx(), &escrow_address, &p1_payout_address).unwrap()),
                (p2_name, create_payout_tx(&funding_tx.clone().extract_tx(), &escrow_address, &p2_contract_info.payout_address).unwrap()),
            ].iter().cloned().collect();

        let tx_token_pairs: Vec<(tglib::bdk::bitcoin::Txid, Vec<u8>)> = event.outcomes.iter().enumerate().map(|(i, outcome)| {
            let txid = payout_txs.get(event_payouts.get(i).unwrap()).unwrap().txid();
            let token_bytes = tglib::hex::decode(&outcome.token).unwrap();
            (txid, token_bytes)
        }).collect();
        let tx_token_script = create_token_pair_script(&event.oracle_pubkey, tx_token_pairs);
        let contract = Contract::new(
            p1_pubkey,
            p2_contract_info.escrow_pubkey,
            arbiter_pubkey,
            event.oracle_pubkey,
            p1_payout_address,
            p2_contract_info.payout_address,
            funding_tx,
            tx_token_script,
        );

        let cxid = tglib::hex::encode(contract.cxid());

        let token_records = event_payouts.iter().cloned().enumerate().map(|(i, player)| {
            let outcome = event.outcomes.get(i).unwrap();
            
            TokenRecord {
                cxid: cxid.clone(),
                player,
                token: outcome.token.clone(),
                desc: outcome.desc.clone(),
            }
        }).collect();

        Ok((contract, token_records))
    }

    pub fn create_contract(&self, p2_contract_info: PlayerContractInfo, amount: Amount, arbiter_pubkey: PublicKey ) -> Result<Contract> {

        let p1_pubkey = self.get_escrow_pubkey();
        let escrow_address = create_escrow_address(&p1_pubkey, &p2_contract_info.escrow_pubkey, &arbiter_pubkey, self.network).unwrap();
        let funding_tx = self.create_funding_tx(&p2_contract_info, amount, &escrow_address)?;
        let p1_payout_address = self.offline_wallet().get_new_address()?;
// need the oracle tokens here
        let p1_payout_tx = create_payout_tx(&funding_tx.clone().extract_tx(), &escrow_address, &p1_payout_address).unwrap();
        let p2_payout_tx = create_payout_tx(&funding_tx.clone().extract_tx(), &escrow_address, &p2_contract_info.payout_address).unwrap();
        let tx_token_pairs = vec![
            (p1_payout_tx.txid(), p1_payout_tx.txid().to_vec()),
            (p2_payout_tx.txid(), p2_payout_tx.txid().to_vec()),
        ];
        let oracle_pubkey = referee_pubkey();
        let tx_token_script = create_token_pair_script(&oracle_pubkey, tx_token_pairs);
        let payout_script = create_payout_script(&escrow_address, &p1_payout_address, &p2_contract_info.payout_address, &funding_tx.clone().extract_tx());

        assert_eq!(tx_token_script, payout_script);

        Ok(Contract::new(
            p1_pubkey,
            p2_contract_info.escrow_pubkey,
            arbiter_pubkey,
            oracle_pubkey,
            p1_payout_address,
            p2_contract_info.payout_address,
            funding_tx,
            payout_script,
        ))
    }

    fn create_funding_tx(&self, p2_contract_info: &PlayerContractInfo, amount: Amount, escrow_address: &Address) -> Result<PartiallySignedTransaction> {
        let mut input = Vec::new();
        let mut psbt_inputs = Vec::new();
        let arbiter_fee = amount.as_sat()/100;
        let sats_per_player = (amount.as_sat() + arbiter_fee + TX_FEE)/2;
// electrum default is 20000 sats
        let mut total: u64 = 0;

        for (outpoint, value, psbt_input) in &p2_contract_info.utxos {
            if total > sats_per_player {
                break
            } else {
                total += value;
                input.push(TxIn{
                    previous_output: *outpoint,
                    script_sig: Script::new(),
                    sequence: 0,
                    witness: Vec::new(),
                });
                psbt_inputs.push(psbt_input.clone());
            }
        }
        if total < sats_per_player {
            return Err(Error::Adhoc("p2 has insufficient funds"))
        }
        let p2_change = total - sats_per_player;
        let wallet = self.wallet()?;
        assert!(wallet.sync(noop_progress(), None).is_ok());
        for utxo in wallet.list_unspent().unwrap() {
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
                psbt_inputs.push(wallet.get_psbt_input(utxo, None, false).unwrap());
            }
        }
        assert!(total > 2 * sats_per_player + p2_change);
        let p1_change = total - 2 * sats_per_player - p2_change;

        let arbiter_client = self.arbiter_client();
        let fee_address = arbiter_client.get_fee_address().map_err(|_| Error::Adhoc("couldn't get fee address"))?;

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
                script_pubkey: wallet.get_new_address().unwrap().script_pubkey(),
            },
        );

        let mut psbt = PartiallySignedTransaction::from_unsigned_tx(Transaction {
            version: 1,
            lock_time: 0,
            input,
            output,
        }).unwrap();

        psbt.inputs = psbt_inputs;

        Ok(psbt)
    }

    pub fn get_other_player_name(&self, contract_record: &ContractRecord) -> Result<PlayerName> {
        let my_players = self.mine();
        if my_players.contains(&contract_record.p1_name) {
            Ok(contract_record.p2_name.clone())
        } else if my_players.contains(&contract_record.p2_name) {
            Ok(contract_record.p1_name.clone())
        } else {
            Err(Error::Adhoc("not party to this contract"))
        }
    }

    // TODO: refactor this to take p1_amount instead of payout address
    // and p2 gets the difference between p1_amount and the contract amount
    pub fn create_payout(&self, contract: &Contract, p1_amount: Amount, p2_amount: Amount) -> Result<Payout> {
        let contract_amount = contract.amount()?;
        if p1_amount + p2_amount != contract_amount {
            return Err(Error::Adhoc("payout amounts don't sum to contract amount"));
        }
        let escrow_address = create_escrow_address(
            &contract.p1_pubkey,
            &contract.p2_pubkey,
            &contract.arbiter_pubkey,
            NETWORK,
            ).unwrap();
        let funding_tx = contract.clone().funding_tx.extract_tx();
        let (escrow_vout, escrow_txout) = funding_tx.output.iter().enumerate().find(|(_vout, txout)| txout.script_pubkey == escrow_address.script_pubkey()).unwrap();
        let wallet = self.offline_wallet();
        let mut builder = wallet.build_tx();
        let psbt_input = Input {
            witness_utxo: Some(escrow_txout.clone()),
            witness_script: Some(create_escrow_script(&contract.p1_pubkey, &contract.p2_pubkey, &contract.arbiter_pubkey)),
            ..Default::default()
        };
// TODO: set satisfaction weight correctly and include tx fee
        let p1_tx_fee: u64 = Amount::from_btc(Amount::from_sat(TX_FEE).as_btc() * (p1_amount.as_btc() as f64 / contract_amount.as_btc())).unwrap().as_sat();
        let p2_tx_fee: u64 = TX_FEE - p1_tx_fee;
        builder.add_foreign_utxo(OutPoint { vout: escrow_vout as u32, txid: funding_tx.txid()}, psbt_input, 0).unwrap();
        if p1_amount.as_sat() != 0 {
//            builder.add_recipient(Address::p2wpkh(&contract.p1_pubkey, NETWORK).unwrap().script_pubkey(), p1_amount.as_sat() - p1_tx_fee);
            builder.add_recipient(contract.p1_payout_address.script_pubkey(), p1_amount.as_sat() - p1_tx_fee);
        }
        if p2_amount.as_sat() != 0 {
//            builder.add_recipient(Address::p2wpkh(&contract.p2_pubkey, NETWORK).unwrap().script_pubkey(), p2_amount.as_sat() - p2_tx_fee);
            builder.add_recipient(contract.p2_payout_address.script_pubkey(), p2_amount.as_sat() - p2_tx_fee);
        }
        builder.manually_selected_only();
        builder.fee_absolute(TX_FEE);
        let (psbt, _details) = builder.finish().unwrap();
        Ok(Payout::new(contract.clone(), psbt))
    }

    pub fn sign_payout(&self, payout: Payout, pw: Secret<String>) -> Result<PartiallySignedTransaction> {
// derive escrow private key
        let path = DerivationPath::from_str(&format!("m/{}/{}", ESCROW_SUBACCOUNT, ESCROW_KIX)).unwrap();
        let seed = self.saved_seed().unwrap().get_seed(pw)?;
        let account_key = derive_account_xprivkey(seed, self.network);
        let secp = Secp256k1::new();
        let escrow_privkey = account_key.derive_priv(&secp, &path).unwrap().private_key;
// check if we are p1 or p2
        let escrow_pubkey = PublicKey::from_private_key(&secp, &escrow_privkey);
// create escrow descriptor wallet
        let desc = if escrow_pubkey == payout.contract.p1_pubkey {
            format!("wsh(multi(2,{},{},{}))", 
                escrow_privkey.to_wif(), 
                payout.contract.p2_pubkey.to_string(), 
                payout.contract.arbiter_pubkey.to_string())
        } else if escrow_pubkey == payout.contract.p2_pubkey {
            format!("wsh(multi(2,{},{},{}))", 
                payout.contract.p1_pubkey.to_string(), 
                escrow_privkey.to_wif(), 
                payout.contract.arbiter_pubkey.to_string())
        } else {
            return Err(Error::Adhoc("can't sign payout - not party to this payout"))
        };
        
        let wallet = tglib::bdk::Wallet::new_offline(&desc, None, NETWORK, tglib::bdk::database::MemoryDatabase::default()).unwrap();

        let (psbt, _finalized) = wallet.sign(payout.psbt, None).unwrap();
        
        Ok(psbt)
    }
}

impl NameWallet for PlayerWallet {
    fn name_pubkey(&self) -> PublicKey {
        let secp = Secp256k1::new();
        let path = DerivationPath::from_str(&format!("m/{}/{}", NAME_SUBACCOUNT, NAME_KIX)).unwrap();
        let pubkey = self.saved_seed().unwrap().xpubkey.derive_pub(&secp, &path).unwrap();
        pubkey.public_key
    }
}

impl EscrowWallet for PlayerWallet {
    fn get_escrow_pubkey(&self) -> PublicKey {
        let secp = Secp256k1::new();
        let path = DerivationPath::from_str(&format!("m/{}/{}", ESCROW_SUBACCOUNT, ESCROW_KIX)).unwrap();
        let escrow_pubkey = self.saved_seed().unwrap().xpubkey.derive_pub(&secp, &path).unwrap();
        escrow_pubkey.public_key
    }

    fn validate_contract(&self, contract: &Contract) -> TgResult<()> {
        let player_pubkey = self.get_escrow_pubkey();
        if contract.p1_pubkey != player_pubkey && contract.p2_pubkey != player_pubkey {
            return Err(TgError::Adhoc("contract doesn't contain our pubkey"));
        }
        let arbiter_client = self.arbiter_client();
        let arbiter_pubkey = arbiter_client.get_escrow_pubkey().unwrap();
        if contract.arbiter_pubkey != arbiter_pubkey {
            return Err(TgError::Adhoc("unexpected arbiter pubkey"));
        }
        contract.validate()
    }
}

impl SigningWallet for PlayerWallet {

    fn sign_tx(&self, mut psbt: PartiallySignedTransaction, path: Option<DerivationPath>, pw: Secret<String>) -> TgResult<PartiallySignedTransaction> {
        match path {
            Some(path) => {
                let seed = self.saved_seed().unwrap().get_seed(pw)?;
                let account_key = derive_account_xprivkey(seed, self.network);
                let secp = Secp256k1::new();
                let signing_key = account_key.derive_priv(&secp, &path).unwrap();
                signing_key.private_key.sign_tx(&mut psbt, &secp)?;
                Ok(psbt)
            }
            None => {
                let signing_wallet = self.signing_wallet(pw)?;
                signing_wallet.sync(noop_progress(), None).unwrap();
                let (signed_psbt, _finished) = signing_wallet.sign(psbt, None)?;

                Ok(signed_psbt)
            },
        }
    }

    fn sign_message(&self, msg: Message, path: DerivationPath, pw: Secret<String>) -> TgResult<Signature> {
        let seed = self.saved_seed().unwrap().get_seed(pw)?;
        let account_key = derive_account_xprivkey(seed, self.network);
        let secp = Secp256k1::new();
        let signing_key = account_key.derive_priv(&secp, &path).unwrap();
        Ok(secp.sign(&msg, &signing_key.private_key.key))
    }
}
