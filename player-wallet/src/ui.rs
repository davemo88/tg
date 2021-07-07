use std::str::FromStr;
use libexchange::{
    AuthTokenSig,
    ExchangeService,
    PlayerContractInfo,
};
use tglib::{
    bdk::{
        blockchain::Blockchain,
        bitcoin::{
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
    },
    hex,
    secrecy::Secret,
    arbiter::ArbiterService,
    contract::Contract,
    payout::{
        Payout,
        PayoutRecord,
    },
    player::{
        PlayerName,
        PlayerNameService,
    },
    wallet::{
        sign_contract,
        EscrowWallet,
        NameWallet,
        SigningWallet,
        NAME_SUBACCOUNT,
        NAME_KIX,
    },
    mock::PAYOUT_VERSION,
};
use crate::{
    Event,
    Error,
    db::{
        ContractRecord,
        PlayerRecord,
        TokenRecord,
    },
    wallet::PlayerWallet,
};

type Result<T> = std::result::Result<T, Box<dyn std::error::Error>>;

// basic crypto wallet
pub trait WalletUI {
    fn deposit(&self) -> Address;
    fn balance(&self) -> Result<Amount>;
//    fn withdraw(&self, address: Address, amount: Amount) -> Transaction;
    fn fund(&self) -> Result<Txid>;
    fn get_tx(&self, txid: &str) -> Result<bool>;
}

impl PlayerWallet {
    fn get_auth(&self, player_name: &PlayerName, pw: Secret<String>) -> Result<AuthTokenSig> {
        let token = self.exchange_client().get_auth_token(&player_name)?;
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

#[derive(Debug)]
pub enum NewDocumentParams {
    NewContractParams { 
        p1_name: PlayerName, 
        p2_name: PlayerName, 
        amount: Amount, 
        event: Option<Event>,
        event_payouts: Option<Vec<PlayerName>>,
        desc: Option<String>,  
    },
    NewPayoutParams { 
        cxid: String, 
        p1_amount: Amount,
        p2_amount: Amount 
    },
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

    fn get_tx(&self, txid: &str) -> Result<bool> {
        let wallet = self.wallet()?;
        let blockchain_client = wallet.client();
        match blockchain_client.get_tx(&Txid::from_str(txid)?) {
            Ok(maybe_transaction) => Ok(maybe_transaction.is_some()),
// electrum client returns an error for a missing / unindexed tx so the option within the result seems redundant sometimes
            Err(tglib::bdk::Error::Electrum(tglib::bdk::electrum_client::Error::Protocol(_e))) => Ok(false),
            Err(e) => Err(Box::new(e))
        }
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
        self.name_client().register_name(name.clone(), self.name_pubkey(), sig)?;
        PlayerUI::add(self, name)
    }

    fn add(&self, name: PlayerName) -> Result<()> {
        let player = PlayerRecord { name };
        self.db().insert_player(player)?; 
        Ok(())
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
        for utxo in wallet.list_unspent()? {
            if total >= amount.as_sat() {
                break
            } else {
                total += utxo.txout.value;
                let psbt_input = wallet.get_psbt_input(utxo.clone(), None, false)?;
                utxos.push((utxo.outpoint, utxo.txout.value, psbt_input));
            }
        }
        if total < amount.as_sat()  {
            return Err(Error::Adhoc("insufficient funds").into())
        }
        let info = PlayerContractInfo {
            name,
            escrow_pubkey: self.get_escrow_pubkey(),
            change_address: wallet.get_new_address()?,
            payout_address: wallet.get_new_address()?,
            utxos,
        };

        let sig = self.sign_message(Message::from_slice(&info.hash()).unwrap(), DerivationPath::from_str(&format!("m/{}/{}", NAME_SUBACCOUNT, NAME_KIX)).unwrap(), pw)?;

        let _r = self.exchange_client().set_contract_info(info, self.name_pubkey(), sig);
        Ok(())
    }

}

impl DocumentUI<ContractRecord> for PlayerWallet {
    fn new(&self, params: NewDocumentParams) -> Result<ContractRecord> {

        println!("params: {:?}", params);
        let (p1_name, p2_name, amount, event, event_payouts, desc) = match params {
            NewDocumentParams::NewContractParams { p1_name, p2_name, amount, event, event_payouts, desc } => (p1_name, p2_name, amount, event, event_payouts, desc),
            _ => return Err(Error::Adhoc("invalid params").into()),
        };

        if !self.mine().contains(&p1_name) {
            return Err(Error::Adhoc("can't create contract: p1 name not controlled by local wallet").into())
        }

// TODO: check if p2_name is registered, or just let the future contract info check fail
// if p2 isn't registered, they couldn't have posted contract info
        let p2_contract_info = match self.exchange_client().get_contract_info(p2_name.clone())? {
            Some(info) => info,
            None => {
                return Err(Error::Adhoc("can't create contract: couldn't fetch p2 contract info").into())
            }
        };

        let arbiter_pubkey = self.arbiter_client().get_escrow_pubkey()?;
//        let arbiter_pubkey = match self.arbiter_client().get_escrow_pubkey() {
//            Ok(pubkey) => pubkey,
//            Err(_) => return Err(Error::Adhoc("can't create contract: couldn't get arbiter pubkey").into())
//        };

// TODO: could fail if amount is too large
        let (contract, maybe_token_records): (Contract, Option<Vec<TokenRecord>>) = match &event {
            Some(event) => {
                let (contract, token_records) = self.create_event_contract(&p1_name, &p2_name, p2_contract_info, amount, arbiter_pubkey, event, &event_payouts.unwrap())?;
                (contract, Some(token_records))
            }
            None => (self.create_contract(p2_contract_info, amount, arbiter_pubkey)?, None),
        };

        let contract_record = ContractRecord {
            cxid: hex::encode(contract.cxid()),
            p1_name,
            p2_name,
            hex: hex::encode(contract.to_bytes()),
            desc: match &event {
                Some(event) => event.desc.clone(),
                None => desc.unwrap_or_default(),
            }
        };

        self.db().insert_contract(contract_record.clone())?;

        if let Some(token_records) = maybe_token_records {
            for record in token_records {
                self.db().insert_token(record)?;
            }
        }

        Ok(contract_record)
    }

    fn import(&self, hex: &str) -> Result<()> {
// accept both contracts and contract records in binary
// contract record binary encoding can be defined in player-wallet libs
// is not a necessarily standardized encoding like contract
// try to parse contract record first, since it contains more info
        if let Ok(contract_record) = serde_json::from_str::<ContractRecord>(&hex) {
            match self.db().insert_contract(contract_record) {
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
        self.exchange_client().send_contract(
            contract_record.clone(),
            self.get_other_player_name(&contract_record)?,
        )
    }

    fn receive(&self, player_name: PlayerName, pw: Secret<String>) -> Result<Option<String>> {
        let auth = self.get_auth(&player_name, pw)?;
        let received = self.exchange_client().receive_contract(auth)?;
        if let Some(contract_record) = received {
            self.db().insert_contract(contract_record.clone())?;
            Ok(Some(contract_record.cxid))
        } else {
            Ok(None)
        }
    }

    fn submit(&self, cxid: &str) -> Result<()> {
        if let Some(cr) = self.db().get_contract(&cxid) {
            let mut contract = Contract::from_bytes(hex::decode(cr.hex)?)?;
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
            let contract = Contract::from_bytes(hex::decode(cr.hex)?)?;
            self.wallet()?.broadcast(contract.funding_tx.extract_tx())?;
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
        let (cxid, p1_amount, p2_amount): (String, Amount, Amount) = match params {
            NewDocumentParams::NewPayoutParams { cxid, p1_amount, p2_amount } => (cxid, p1_amount, p2_amount),
            _ => return Err(Error::Adhoc("invalid params").into()),
        };
        let contract_record = self.db().get_contract(&cxid).unwrap();
        let contract = Contract::from_bytes(hex::decode(contract_record.hex).unwrap()).unwrap();
        let payout = self.create_payout(&contract, p1_amount, p2_amount)?;
        let payout_record = PayoutRecord::from(payout);
        let _r = self.db().insert_payout(payout_record.clone());
        Ok(payout_record)
    }

    fn import(&self, hex: &str) -> Result<()> {
        if let Ok(payout_record) = serde_json::from_str::<PayoutRecord>(&hex) {
            let _r = self.db().insert_payout(payout_record);
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
        let pr = self.db().get_payout(&cxid).ok_or(Error::Adhoc("unknown payout"))?;
        let cr = DocumentUI::<ContractRecord>::get(self, &cxid).ok_or(Error::Adhoc("unknown contract"))?;
        let contract = Contract::from_bytes(hex::decode(cr.hex)?)?;
        let psbt: PartiallySignedTransaction = consensus::deserialize(&hex::decode(pr.psbt)?)?;
        let payout = Payout::new(contract, psbt);
        let psbt = self.sign_payout(payout, pw)?;
        let psbt = hex::encode(consensus::serialize(&psbt));
        self.db().insert_payout(PayoutRecord {
            cxid: pr.cxid, 
            psbt,
            sig: match script_sig {
                Some(sig) => hex::encode(sig.serialize_der()),
                None => String::default(),
            }
        })?;
        Ok(())
    }

    fn send(&self, cxid: &str) -> Result<()> {
        let payout_record = DocumentUI::<PayoutRecord>::get(self, cxid).ok_or(Error::Adhoc("unknown payout"))?;
        let contract_record = DocumentUI::<ContractRecord>::get(self, cxid).ok_or(Error::Adhoc("unknown contract"))?;
        self.exchange_client().send_payout(
            payout_record,
            self.get_other_player_name(&contract_record)?
        )
    }

    fn receive(&self, player_name: PlayerName, pw: Secret<String>) -> Result<Option<String>> {
        let auth = self.get_auth(&player_name, pw)?;
        let received = self.exchange_client().receive_payout(auth)?;
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
        let psbt = self.arbiter_client().submit_payout(&p)?; 
        p.psbt = psbt; 
        self.db().insert_payout(PayoutRecord::from(p)).unwrap();
        Ok(())
    }

    fn broadcast(&self, cxid: &str) -> Result<()> {
         if let Some(pr) = self.db().get_payout(&cxid) {
             let tx: PartiallySignedTransaction = consensus::deserialize(&hex::decode(pr.psbt)?)?;
             self.wallet()?.broadcast(tx.extract_tx())?;
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
