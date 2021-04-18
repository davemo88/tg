use std::{
    str::FromStr,
    sync::Arc,
};
use tglib::{
    bdk::bitcoin::{
        Address,
        Amount,
        consensus,
        hash_types::Txid,
        hashes::{
            sha256,
            Hash,
            HashEngine,
        },
        secp256k1::{
            Message,
            Signature,
        },
        util::{
            bip32::DerivationPath,
            psbt::PartiallySignedTransaction,
        },
    },
    hex,
    secrecy::Secret,
    arbiter::{
        ArbiterService,
        AuthTokenSig,
    },
    contract::{
        Contract,
        ContractRecord,
        PlayerContractInfo,
    },
    payout::{
        Payout,
        PayoutRecord,
    },
    player::{
        PlayerName,
        PlayerNameService,
    },
    wallet::{
        create_payout,
        sign_contract,
        sign_payout_psbt,
        EscrowWallet,
        NameWallet,
        SigningWallet,
        NAME_SUBACCOUNT,
        NAME_KIX,
    },
    mock::{
        NETWORK,
        PAYOUT_VERSION,
    },
};
use crate::{
//    Result as PwResult,
    Error,
    db::PlayerRecord,
    wallet::PlayerWallet,
};

// basic crypto wallet
pub trait WalletUI {
    fn deposit(&self) -> Address;
    fn balance(&self) -> Result<Amount>;
//    fn withdraw(&self, address: Address, amount: Amount) -> Transaction;
    fn fund(&self) -> Result<Txid>;
}

type Result<T> = std::result::Result<T, Box<dyn std::error::Error>>;

impl PlayerWallet {
    fn get_auth(&self, player_name: &PlayerName, pw: Secret<String>) -> Result<AuthTokenSig> {
        let token = self.arbiter_client().get_auth_token(&player_name).unwrap();
        let sig = self.sign_message(
            Message::from_slice(&token).unwrap(), 
            DerivationPath::from_str(&format!("m/{}/{}", NAME_SUBACCOUNT, NAME_KIX)).unwrap(),
            pw
        ).unwrap();
        let sig_hex = hex::encode(sig.serialize_der());
        Ok(AuthTokenSig {
            player_name: player_name.clone(),
            pubkey: self.name_pubkey(),
            sig_hex,
        })
    }
}

// players
pub trait PlayerUI {
    fn register(&self, name: PlayerName, pw: Secret<String>) -> Result<()>;
    fn add(&self, name: PlayerName) -> Result<()>;
    fn remove(&self, name: PlayerName) -> Result<()>;
    fn list(&self) -> Vec<PlayerRecord>;
    fn mine(&self) -> Vec<PlayerName>;
    fn post(&self, name: PlayerName, amount: Amount, pw: Secret<String>) -> Result<()>;
}

// contracts and payouts
pub trait DocumentUI<T> {
    fn new(&self, params: NewDocumentParams) -> Result<T>;
    fn import(&self, hex: &str) -> Result<()>;
    fn export(&self, cxid: &str) -> Option<String>;
    fn get(&self, cxid: &str) -> Option<T>;
    fn sign(&self, params: SignDocumentParams, pw: Secret<String>) -> Result<()>;
    fn send(&self, cxid: &str) -> Result<()>;
    fn receive(&self, player_name: PlayerName, pw: Secret<String>) -> Result<Option<String>>;
    fn submit(&self, cxid: &str) -> Result<()>;
    fn broadcast(&self, cxid: &str) -> Result<()>;
    fn list(&self) -> Vec<T>;
    fn delete(&self, cxid: &str) -> Result<()>;
}

pub enum NewDocumentParams {
    NewContractParams { p1_name: PlayerName, p2_name: PlayerName, amount: Amount, desc: Option<String> },
    NewPayoutParams { cxid: String, name: PlayerName, amount: Amount },
}

pub enum SignDocumentParams {
    SignContractParams { cxid: String, sign_funding_tx: bool },
    SignPayoutParams { cxid: String, script_sig: Option<Signature> },
}

impl WalletUI for PlayerWallet {
    fn deposit(&self) -> Address {
        self.offline_wallet().get_new_address().unwrap()
    }

    fn balance(&self) -> Result<Amount> {
        Ok(Amount::from_sat(self.wallet()?.get_balance()?))
    }

//    fn withdraw(&self, address: Address, amount: Amount) -> Transaction {
//        self.wallet().create_tx(...)
//    }

    fn fund(&self) -> Result<Txid> {
        let txid = self.arbiter_client().fund_address(self.offline_wallet().get_new_address().unwrap())?;
        Ok(txid)
    }
}

impl PlayerUI for PlayerWallet {
    fn register(&self, name: PlayerName, pw: Secret<String>) -> Result<()> {
        let mut engine = sha256::HashEngine::default();
        engine.input(name.0.as_bytes());
        let hash: &[u8] = &sha256::Hash::from_engine(engine);
        let sig = self.sign_message(
            Message::from_slice(hash).unwrap(),
            DerivationPath::from_str(&format!("m/{}/{}", NAME_SUBACCOUNT, NAME_KIX)).unwrap(),
            pw,
        ).unwrap();
        match self.name_client().register_name(name.clone(), self.name_pubkey(), sig) {
            Ok(()) => PlayerUI::add(self, name),
// TODO: better error message here
            Err(_) => Err(Error::Adhoc("register name failed").into()),
        }
    }

    fn add(&self, name: PlayerName) -> Result<()> {
        let player = PlayerRecord { name };
        match self.db().insert_player(player.clone()) {
            Ok(_) => Ok(()),
            Err(e) => Err(Box::new(Error::Database(Arc::new(e)))),
        }
    }

    fn remove(&self, name: PlayerName) -> Result<()> {
        match self.db().delete_player(name) {
            Ok(num_deleted) => match num_deleted {
                0 => Err(Error::Adhoc("no player with that name").into()),
                1 => Ok(()),
                n => panic!("{} removed, should be impossible", n),//this is impossible
            }
            Err(e) => Err(e.into()),
        }
    }

    fn list(&self) -> Vec<PlayerRecord> {
        self.db().all_players().unwrap()
    }

    fn mine(&self) -> Vec<PlayerName> {
        self.name_client().get_player_names(&self.name_pubkey())
    }

    fn post(&self, name: PlayerName, amount: Amount, pw: Secret<String>) -> Result<()> {
        let mut utxos = vec!();
        let mut total: u64 = 0;
        let wallet = self.offline_wallet();
        for utxo in wallet.list_unspent().unwrap() {
            if total >= amount.as_sat() {
                break
            } else {
                total += utxo.txout.value;
                let psbt_input = wallet.get_psbt_input(utxo.clone(), None, false).unwrap();
                utxos.push((utxo.outpoint, utxo.txout.value, psbt_input));
            }
        }
        if total < amount.as_sat()  {
            return Err(Error::Adhoc("insufficient funds").into())
        }
        let info = PlayerContractInfo {
            name,
            escrow_pubkey: self.get_escrow_pubkey(),
            change_address: wallet.get_new_address().unwrap(),
            utxos,
        };

        let sig = self.sign_message(Message::from_slice(&info.hash()).unwrap(), DerivationPath::from_str(&format!("m/{}/{}", NAME_SUBACCOUNT, NAME_KIX)).unwrap(), pw).unwrap();

        let _r = self.arbiter_client().set_contract_info(info, self.name_pubkey(), sig);
        Ok(())
    }

}

impl DocumentUI<ContractRecord> for PlayerWallet {
    fn new(&self, params: NewDocumentParams) -> Result<ContractRecord> {
        let (p1_name, p2_name, amount, desc) = match params {
            NewDocumentParams::NewContractParams { p1_name, p2_name, amount, desc } => (p1_name, p2_name, amount, desc),
            _ => return Err(Error::Adhoc("invalid params").into()),
        };

        if !self.mine().contains(&p1_name) {
            return Err(Error::Adhoc("can't create contract: p1 name not controlled by local wallet").into())
        }

// TODO: check if p2_name is registered, or just let the future contract info check fail
// if p2 isn't registered, they couldn't have posted contract info
        let p2_contract_info = match self.arbiter_client().get_contract_info(p2_name.clone())? {
            Some(info) => info,
            None => {
                return Err(Error::Adhoc("can't create contract: couldn't fetch p2 contract info").into())
            }
        };

        let arbiter_pubkey = match self.arbiter_client().get_escrow_pubkey() {
            Ok(pubkey) => pubkey,
            Err(_) => return Err(Error::Adhoc("can't create contract: couldn't get arbiter pubkey").into())
        };

// TODO: could fail if amount is too large
        let contract = self.create_contract(p2_contract_info, amount, arbiter_pubkey)?;

        let contract_record = ContractRecord {
            cxid: hex::encode(contract.cxid()),
            p1_name,
            p2_name,
            hex: hex::encode(contract.to_bytes()),
            desc: desc.unwrap_or_default(),
        };

        match self.db().insert_contract(contract_record.clone()) {
            Ok(_) => Ok(contract_record),
            Err(_) => Err(Error::Adhoc("couldn't create contract: db insertion failed").into()),
        }
    }

    fn import(&self, hex: &str) -> Result<()> {
// accept both contracts and contract records in binary
// contract record binary encoding can be defined in player-wallet libs
// is not a necessarily standardized encoding like contract
// try to parse contract record first, since it contains more info
        if let Ok(contract_record) = serde_json::from_str::<ContractRecord>(&hex) {
            match self.db().insert_contract(contract_record.clone()) {
                Ok(_) => Ok(()),
                Err(_) => Err(Error::Adhoc("couldn't import contract: db insertion failed").into()),
            }
        }
// disable this for now until we decide how to address missing names
//           else if let Ok(contract) = Contract::from_bytes(hex::decode(a.value_of("contract-value").unwrap()).unwrap()) {
//// contract alone doesn't have player names
//           let contract_record = db::ContractRecord {
//               cxid: hex::encode(contract.cxid()),
//               p1_name: PlayerName::from(contract.p1_pubkey),
//               p2_name: PlayerName::from(contract.p2_pubkey),
//               hex: hex::encode(contract.to_bytes()),
//               desc: &str::default(),
//           };
//           match wallet.db.insert_contract(contract_record.clone()) {
//               Ok(_) => println!("imported contract {}", hex::encode(contract.cxid())),
//               Err(e) => println!("{:?}", e),
//           }
//       } 
       else {
           Err(Error::Adhoc("invalid contract").into())
       }

    }

    fn export(&self, cxid: &str) -> Option<String> {
        if let Some(contract_record) = self.db().get_contract(&cxid) {
            Some(contract_record.hex)
        } else { 
            None
        }
    }

    fn get(&self, cxid: &str) -> Option<ContractRecord> {
        self.db().get_contract(&cxid)
    }

    fn sign(&self, params: SignDocumentParams, pw: Secret<String>) -> Result<()> {
        let (cxid, sign_funding_tx) = match params {
            SignDocumentParams::SignContractParams { cxid, sign_funding_tx } => (cxid, sign_funding_tx),
            _ => return Err(Error::Adhoc("invalid params").into()),
        };
        if let Some(contract_record) = self.db().get_contract(&cxid) {
            let mut contract = Contract::from_bytes(hex::decode(contract_record.hex.clone()).unwrap()).unwrap();
            let sig = sign_contract(self, &contract, pw.clone()).unwrap();
            if !contract.sigs.contains(&sig) {
                contract.sigs.push(sig);
            }
            if sign_funding_tx {
                contract.funding_tx = self.sign_tx(contract.funding_tx.clone(), None, pw).unwrap();
            }
            let _r = self.db().add_signature(contract_record.cxid, hex::encode(contract.to_bytes()));
            Ok(())
        } else {
            Err(Error::Adhoc("unknown contract").into())
        }

    }

    fn send(&self, cxid: &str) -> Result<()> {
        let contract_record = DocumentUI::<ContractRecord>::get(self, cxid).ok_or(Error::Adhoc("unknown contract"))?;
        Ok(self.arbiter_client().send_contract(
            contract_record.clone(),
            self.get_other_player_name(&contract_record).unwrap(),
        )?)
    }

    fn receive(&self, player_name: PlayerName, pw: Secret<String>) -> Result<Option<String>> {
        let auth = self.get_auth(&player_name, pw).unwrap();
        let received = self.arbiter_client().receive_contract(auth)?;
        if let Some(contract_record) = received {
            let _changes = self.db().insert_contract(contract_record.clone())?;
            Ok(Some(contract_record.cxid))
        } else {
            Ok(None)
        }
    }

    fn submit(&self, cxid: &str) -> Result<()> {
        if let Some(cr) = self.db().get_contract(&cxid) {
            let mut contract = Contract::from_bytes(hex::decode(cr.hex.clone()).unwrap()).unwrap();
            if let Ok(sig) = self.arbiter_client().submit_contract(&contract) {
               contract.sigs.push(sig);
               let _r = self.db().add_signature(cr.cxid, hex::encode(contract.to_bytes()));
               Ok(())
            }
            else {
                Err(Error::Adhoc("contract rejected").into())
            }
        }
        else {
            Err(Error::Adhoc("unknown contract").into())
        }
    }

    fn broadcast(&self, cxid: &str) -> Result<()> {
        if let Some(cr) = self.db().get_contract(&cxid) {
            let contract = Contract::from_bytes(hex::decode(cr.hex.clone()).unwrap()).unwrap();
            let _r = self.wallet()?.broadcast(contract.funding_tx.extract_tx());
            Ok(())
        } else {
            Err(Error::Adhoc("unknown contract").into())
        }
    }

    fn list(&self) -> Vec<ContractRecord> {
        self.db().all_contracts().unwrap()
    }

    fn delete(&self, cxid: &str) -> Result<()> {
        match self.db().delete_contract(cxid) {
            Ok(_) => Ok(()),
            Err(_) => Err(Error::Adhoc("delete contract failed").into()),
        }
    }
}

impl DocumentUI<PayoutRecord> for PlayerWallet {
    fn new(&self, params: NewDocumentParams) -> Result<PayoutRecord> {
        let (cxid, _name, _amount): (String, PlayerName, Amount) = match params {
            NewDocumentParams::NewPayoutParams { cxid, name, amount } => (cxid, name, amount),
            _ => return Err(Error::Adhoc("invalid params").into()),
        };
        let contract_record = self.db().get_contract(&cxid).unwrap();
        let contract = Contract::from_bytes(hex::decode(contract_record.hex).unwrap()).unwrap();
        let escrow_pubkey = self.get_escrow_pubkey();
        let payout = create_payout(&contract, &Address::p2wpkh(&escrow_pubkey, NETWORK).unwrap());
        let payout_record = PayoutRecord::from(payout);
        let _r = self.db().insert_payout(payout_record.clone());
        Ok(payout_record)
    }

    fn import(&self, hex: &str) -> Result<()> {
        if let Ok(payout_record) = serde_json::from_str::<PayoutRecord>(&hex) {
            let _r = self.db().insert_payout(payout_record.clone());
            Ok(())
        }
        else {
            Err(Error::Adhoc("invalid payout").into())
        }
    }

    fn export(&self, cxid: &str) -> Option<String> {
        let cr = self.db().get_contract(&cxid).unwrap();
        let pr = self.db().get_payout(&cxid).unwrap();
        let p = Payout {
            version: PAYOUT_VERSION,
            contract: Contract::from_bytes(hex::decode(cr.hex).unwrap()).unwrap(),
            psbt: consensus::deserialize(&hex::decode(pr.psbt).unwrap()).unwrap(),
            script_sig: Signature::from_der(&hex::decode(pr.sig).unwrap()).ok()
        };
        Some(hex::encode(p.to_bytes()))
    }

    fn get(&self, cxid: &str) -> Option<PayoutRecord> {
        self.db().get_payout(&cxid)
    }

    fn sign(&self, params: SignDocumentParams, pw: Secret<String>) -> Result<()> {
        let (cxid, script_sig) = match params {
            SignDocumentParams::SignPayoutParams { cxid, script_sig } => (cxid, script_sig),
            _ => return Err(Error::Adhoc("invalid params").into()),
        };
        if let Some(pr) = self.db().get_payout(&cxid) {
            let psbt: PartiallySignedTransaction = consensus::deserialize(&hex::decode(pr.psbt).unwrap()).unwrap();
            let psbt = sign_payout_psbt(self, psbt, pw).unwrap();
            let psbt = hex::encode(consensus::serialize(&psbt));
            self.db().insert_payout(PayoutRecord {
                cxid: pr.cxid, 
                psbt,
                sig: match script_sig {
                    Some(sig) => hex::encode(sig.serialize_der()),
                    None => String::default(),
                }
            }).unwrap();
            Ok(())
        }
        else {
            Err(Error::Adhoc("unknown payout").into())
        }
    }

    fn send(&self, cxid: &str) -> Result<()> {
        let payout_record = DocumentUI::<PayoutRecord>::get(self, cxid).ok_or(Error::Adhoc("unknown payout"))?;
        let contract_record = DocumentUI::<ContractRecord>::get(self, cxid).ok_or(Error::Adhoc("unknown contract"))?;
        Ok(self.arbiter_client().send_payout(
            payout_record,
            self.get_other_player_name(&contract_record)?
        )?)
    }

    fn receive(&self, player_name: PlayerName, pw: Secret<String>) -> Result<Option<String>> {
        let auth = self.get_auth(&player_name, pw).unwrap();
        let received = self.arbiter_client().receive_payout(auth)?;
        if let Some(payout_record) = received {
            let _changes = self.db().insert_payout(payout_record.clone())?;
            Ok(Some(payout_record.cxid))
        } else {
            Ok(None)
        }
    }

    fn submit(&self, cxid: &str) -> Result<()> {
        let cr = self.db().get_contract(&cxid).unwrap();
        let pr = self.db().get_payout(&cxid).unwrap();
        let mut p = Payout {
            version: PAYOUT_VERSION,
// TODO: poster child for serde hell
            contract: Contract::from_bytes(hex::decode(cr.hex).unwrap()).unwrap(),
            psbt: consensus::deserialize(&hex::decode(pr.psbt).unwrap()).unwrap(),
            script_sig: Signature::from_der(&hex::decode(pr.sig).unwrap()).ok()
        };
        if let Ok(psbt) = self.arbiter_client().submit_payout(&p) {
            p.psbt = psbt; 
            self.db().insert_payout(PayoutRecord::from(p)).unwrap();
            Ok(())
        }
        else {
            Err(Error::Adhoc("payout rejected").into())
        }
    }

    fn broadcast(&self, cxid: &str) -> Result<()> {
         if let Some(pr) = self.db().get_payout(&cxid) {
             let tx: PartiallySignedTransaction = consensus::deserialize(&hex::decode(pr.psbt).unwrap()).unwrap();
             let _r = self.wallet()?.broadcast(tx.extract_tx());
             Ok(())
         }
         else {
             Err(Error::Adhoc("unknown payout").into())
         }
    }

    fn list(&self) -> Vec<PayoutRecord> {
        match self.db().all_payouts() {
            Ok(prs) => prs,
// TODO: this doesn't seem like good behavior if there is a db error
            Err(_) => vec!(),
        }

    }

    fn delete(&self, cxid: &str) -> Result<()> {
        match self.db().delete_payout(cxid) {
            Ok(_) => Ok(()),
            Err(_) => Err(Error::Adhoc("delete payout failed").into()),
        }
    }
}
