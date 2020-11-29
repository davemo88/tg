use std::{
    env::current_dir,
    str::FromStr,
    convert::TryInto,
};
use bdk::{
    bitcoin::{
        Address,
        Amount,
        Network,
        PublicKey,
        Transaction,
        TxIn,
        TxOut,
        Script,
        blockdata::{
            script::Builder,
            opcodes::all as Opcodes,
            transaction::OutPoint,
        },
        secp256k1::{
            Secp256k1,
            Message,
            Signature,
            All,
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
        ElectrumBlockchain,
    },
    database::MemoryDatabase,
    electrum_client::Client,
    Error,
    ScriptType,
    Wallet,
};
use tglib::{
    Result as TgResult,
    TgError,
    arbiter::{
        ArbiterId,
        ArbiterService,
    },
    contract::{
        Contract,
        PlayerContractInfo,
    },
    payout::{
        Payout,
    },
    player::{
        PlayerId,
    },
    script::TgScript,
    wallet::{
        create_escrow_address,
        create_payout_script,
        EscrowWallet,
        BITCOIN_ACCOUNT_PATH,
    },
    mock::{
        ARBITER_PUBLIC_URL,
        DB_NAME,
        ELECTRS_SERVER,
        ESCROW_SUBACCOUNT,
        ESCROW_KIX,
    },
};
use crate::{
    arbiter::ArbiterClient,
    db::{
        ContractRecord,
        DB,
    },
};

pub struct PlayerWallet {
    fingerprint: Fingerprint,
    xpubkey: ExtendedPubKey,
    network: Network,
    pub wallet: Wallet<ElectrumBlockchain, MemoryDatabase>,
    pub db: DB,
}

impl PlayerWallet {
    pub fn new(fingerprint: Fingerprint, xpubkey: ExtendedPubKey, network: Network) -> Self {
        let descriptor_key = format!("[{}/{}]{}", fingerprint, BITCOIN_ACCOUNT_PATH, xpubkey);
        let external_descriptor = format!("wpkh({}/0/*)", descriptor_key);
        let internal_descriptor = format!("wpkh({}/1/*)", descriptor_key);
//        let client = Client::new(ELECTRS_SERVER, None).unwrap();
        let client = Client::new("tcp://localhost:60401", None).unwrap();
        let mut db_path = current_dir().unwrap();
        db_path.push(DB_NAME);
        let db = DB::new(&db_path).unwrap();
        db.create_tables();

        PlayerWallet {
            fingerprint,
            xpubkey,
            network,
            wallet: Wallet::new(
                &external_descriptor,
                Some(&internal_descriptor),
                network,
                MemoryDatabase::default(),
                ElectrumBlockchain::from(client)
            ).unwrap(),
            db,

        }
    }

    pub fn player_id(&self) -> PlayerId {
        PlayerId::from(self.xpubkey)
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
            funding_tx,
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

impl EscrowWallet for PlayerWallet {
    fn get_escrow_pubkey(&self) -> PublicKey {
        let secp = Secp256k1::new();
        let path = DerivationPath::from_str(&String::from(format!("m/{}/{}", ESCROW_SUBACCOUNT, ESCROW_KIX))).unwrap();
        let escrow_pubkey = self.xpubkey.derive_pub(&secp, &path).unwrap();
        escrow_pubkey.public_key
    }
}
