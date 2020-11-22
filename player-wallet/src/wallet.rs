use std::{
    str::FromStr,
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
    TgScriptSig,
    arbiter::{
        ArbiterId,
    },
    contract::{
        Contract,
        ContractSignature,
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
        Creation,
        Signing,
    }
};
use crate::{
    mock::{
        PlayerInfoService,
        ArbiterService,
        ARBITER_ID,
        ELECTRS_SERVER,
        BITCOIN_DERIVATION_PATH,
        ESCROW_SUBACCOUNT,
        ESCROW_KIX,
    },
};


pub struct PlayerWallet {
    fingerprint: Fingerprint,
    xpubkey: ExtendedPubKey,
    network: Network,
    pub wallet: Wallet<ElectrumBlockchain, MemoryDatabase>
}

impl PlayerWallet {
    pub fn new(fingerprint: Fingerprint, xpubkey: ExtendedPubKey, network: Network) -> Self {
        let descriptor_key = format!("[{}/44'/0'/0']{}", fingerprint, xpubkey);
        let external_descriptor = format!("wpkh({}/0/*)", descriptor_key);
        let internal_descriptor = format!("wpkh({}/1/*)", descriptor_key);
        let client = Client::new(ELECTRS_SERVER, None).unwrap();
        PlayerWallet {
            fingerprint,
            xpubkey,
            network,
            wallet: Wallet::new(
                &external_descriptor,
                None,//Some(&internal_descriptor),
                network,
                MemoryDatabase::default(),
                ElectrumBlockchain::from(client)
            ).unwrap()
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

    fn create_funding_tx(&self, p2_contract_info: PlayerContractInfo, amount: Amount) -> Transaction {

        let escrow_address = self.create_escrow_address(p2_contract_info.escrow_pubkey).unwrap();

        let mut input = Vec::new();
        let arbiter_fee = amount.as_sat()/100;
        let sats_per_player = (amount.as_sat() + arbiter_fee)/2;
        let mut total: u64 = 0;

// p2's inputs
        for utxo in p2_contract_info.utxos {
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
        let p2_change = total - sats_per_player;
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
        let p1_change = total - 2 * sats_per_player - p2_change;

        let output = vec!(
            TxOut { 
                value: amount.as_sat(),
                script_pubkey: escrow_address.script_pubkey(),
            },
            TxOut { 
                value: arbiter_fee,
                script_pubkey: ArbiterService::get_fee_address().script_pubkey(),
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

    fn create_escrow_address(&self, p2_pubkey: PublicKey) -> TgResult<Address> {

        let p1_pubkey = self.get_new_escrow_pubkey();
        let arbiter_pubkey = ArbiterService::get_escrow_pubkey();

        let escrow_address = Address::p2wsh(
            &self.create_escrow_script(p1_pubkey, p2_pubkey, arbiter_pubkey),
            self.network,
        );

        Ok(escrow_address)

    }

    pub fn get_new_escrow_pubkey(&self) -> PublicKey {
// TODO: need to store escrow_kix somewhere and increment for new contracts
        let secp = Secp256k1::new();
        let path = DerivationPath::from_str(&String::from(format!("m/{}/{}/{}", BITCOIN_DERIVATION_PATH, ESCROW_SUBACCOUNT, ESCROW_KIX))).unwrap();
        println!("{:?}",path);
        let escrow_pubkey = self.xpubkey.derive_pub(&secp, &path).unwrap();
        escrow_pubkey.public_key
    }

    fn create_escrow_script(&self, p1_pubkey: PublicKey, p2_pubkey: PublicKey, arbiter_pubkey: PublicKey) -> Script {
// standard multisig transaction script
// https://en.bitcoin.it/wiki/BIP_0011
        let b = Builder::new()
            .push_opcode(Opcodes::OP_PUSHBYTES_2)
            .push_slice(&p1_pubkey.to_bytes())
            .push_slice(&p2_pubkey.to_bytes())
            .push_slice(&arbiter_pubkey.to_bytes())
            .push_opcode(Opcodes::OP_PUSHBYTES_3)
            .push_opcode(Opcodes::OP_CHECKMULTISIG);

        b.into_script()
    }

    fn create_payout_script(&self, p2_id: PlayerId, amount: Amount, funding_tx: Transaction) -> TgScript {
        TgScript::default()
    }
    
}

impl Creation for PlayerWallet {
    fn create_contract(&self,
        p2_id:          PlayerId,
        amount:         Amount,
    ) -> Contract {

        let p2_contract_info = PlayerInfoService::get_contract_info(&p2_id);

        let funding_tx = self.create_funding_tx(p2_contract_info, amount);

        Contract::new(
            self.player_id(),
            p2_id.clone(),
            ArbiterId(String::from(ARBITER_ID)),
            amount,
            funding_tx.clone(),
            self.create_payout_script(p2_id, amount, funding_tx),
        )
    }

    fn create_payout(&self, contract: &Contract, payout_tx: Transaction, payout_script_sig: TgScriptSig) -> Payout {
        Payout::new(
            &contract,
            payout_tx,
            payout_script_sig,
        )
    }
}

impl Signing for PlayerWallet {

    fn sign_contract(&self, _contract: Contract) -> TgResult<Contract> {
// delegate to SigningWallet e.g. trezor
        Err(TgError("cannot sign contract"))
    }

    fn sign_payout(&self, _payout: Payout) -> TgResult<Payout>{
// delegate to SigningWallet e.g. trezor
        Err(TgError("cannot sign payout request"))
    }
}

// TODO: need to clarify. this is signing in the normal bitcoin / crypto sense
// and the Signing trait is for signing our contracts and payouts only
// this is here because we will delegate from the app wallet to e.g.
// a hardware wallet for key storage and signing

// we'll only do pubkey-related tasks in our application wallet
// and delegate key storage and signing to a better tested wallet 
// e.g. trezor
pub trait SigningWallet {
    fn fingerprint(&self) -> Fingerprint;
    fn xpubkey(&self) -> ExtendedPubKey;
    fn descriptor_xpubkey(&self) -> String;
    fn sign_tx(&self, pstx: PartiallySignedTransaction, kdp: String) -> TgResult<Transaction>;
    fn sign_message(&self, msg: Message, kdp: String) -> TgResult<Signature>;
}
