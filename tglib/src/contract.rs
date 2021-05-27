use std::str::FromStr;
use byteorder::{BigEndian, WriteBytesExt};
use serde::{Serialize, Deserialize,};
use bdk::{
    bitcoin::{
        Address,
        Amount,
        PublicKey,
        blockdata::transaction::OutPoint,
        consensus::{
            self,
            encode::Decodable,
        },
        hashes::{
            Hash as BitcoinHash,
            HashEngine,
            sha256::Hash as ShaHash,
            sha256::HashEngine as ShaHashEngine,
        },
        secp256k1::{
            Message,
            Secp256k1,
            Signature,
        },
        util::psbt::{
            Input,
            PartiallySignedTransaction,
        }
    },
};
use nom::{
    self,
    IResult,
    bytes::complete::take,
    number::complete::{be_u8, be_u32},
//    branch::alt,
    multi::{many0, length_data, length_value},
//    combinator::opt,
    sequence::tuple,
};

use crate::{
    Result,
    Error,
    player::PlayerName,
    script::{
        parser::tg_script,
        TgScript,
    },
    wallet::create_escrow_address,
    mock::{
        NETWORK,
        CONTRACT_VERSION,
    }
};

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Contract {
    pub p1_pubkey:          PublicKey,
    pub p2_pubkey:          PublicKey,
    pub arbiter_pubkey:     PublicKey,
    pub p1_payout_address:  Address,
    pub p2_payout_address:  Address,
    pub funding_tx:         PartiallySignedTransaction,
    pub payout_script:      TgScript,
    pub sigs:               Vec<Signature>, 
    pub version:            u8,
}

impl Contract {
    pub fn new(p1_pubkey: PublicKey, p2_pubkey: PublicKey, arbiter_pubkey: PublicKey, p1_payout_address: Address, p2_payout_address: Address, funding_tx: PartiallySignedTransaction, payout_script: TgScript) -> Self {
        Contract {
            version: CONTRACT_VERSION,
            p1_pubkey,
            p2_pubkey,
            arbiter_pubkey,
            p1_payout_address,
            p2_payout_address,
            funding_tx,
            payout_script,
            sigs: Vec::new(),
        }
    }

    pub fn cxid(&self) -> Vec<u8> {
        let mut engine = ShaHashEngine::default();
        engine.input(&Vec::from(self.payout_script.clone()));
        let hash: &[u8] = &ShaHash::from_engine(engine);
        hash.to_vec()
    }

    pub fn state(&self) -> ContractState {
        return ContractState::Invalid
    }

    pub fn to_bytes(&self) -> Vec<u8> {
        let mut v = Vec::new();
// version
        let _ = v.write_u8(self.version);
// 3 fixed-length pubkeys
        v.extend(self.p1_pubkey.to_bytes());
        v.extend(self.p2_pubkey.to_bytes());
        v.extend(self.arbiter_pubkey.to_bytes());
// 2 payout addresses
        let p1_address_string = self.p1_payout_address.to_string();
        let p1_address_bytes = p1_address_string.as_bytes();
        v.write_u32::<BigEndian>(p1_address_bytes.len() as u32).unwrap();
        v.extend(p1_address_bytes);
        let p2_address_string = self.p2_payout_address.to_string();
        let p2_address_bytes = p2_address_string.as_bytes();
        v.write_u32::<BigEndian>(p2_address_bytes.len() as u32).unwrap();
        v.extend(p2_address_bytes);
// funding tx length  + bytes
        let funding_tx = consensus::serialize(&self.funding_tx);
        v.write_u32::<BigEndian>(funding_tx.len() as u32).unwrap();
        v.extend(funding_tx);
// payout script length  + bytes
        let payout_script = Vec::from(self.payout_script.clone());
        v.write_u32::<BigEndian>(payout_script.len() as u32).unwrap();
        v.extend(payout_script);
        for sig in &self.sigs {
// der-encoded signatures with their lengths
            let sig_bytes = sig.serialize_der().to_vec();
// this is guaranteed to fit, right?
            v.write_u8(sig_bytes.len() as u8).unwrap();
            v.extend(sig_bytes);
        }
        v
    }

    pub fn from_bytes(bytes: Vec<u8>) -> Result<Contract> {
        let (i, c) = contract(&bytes).unwrap();
        if i.len() == 0 {
            Ok(c)
        }
        else {
            Err(Error::Adhoc("couldn't parse contract"))
        }
    }

    pub fn amount(&self) -> Result<Amount> {
        let escrow_address = create_escrow_address(&self.p1_pubkey, &self.p2_pubkey, &self.arbiter_pubkey, NETWORK).unwrap();
        let funding_tx = self.funding_tx.clone().extract_tx();
        let escrow_txout = funding_tx.output.iter().find(|txout| txout.script_pubkey == escrow_address.script_pubkey()).ok_or(Error::Adhoc("couldn't determine amount"))?;
        Ok(Amount::from_sat(escrow_txout.value))
    }

    pub fn fee_address(&self) -> Result<Address> {
        let fee_amount = self.amount().unwrap().as_sat()/100;
        let funding_tx = self.funding_tx.clone().extract_tx();
        let fee_txout = funding_tx.output.iter().find(|txout| txout.value == fee_amount).ok_or(Error::Adhoc("fee not found"))?;
        Ok(Address::from_script(&fee_txout.script_pubkey, NETWORK).unwrap())
    }

    pub fn validate(&self) -> Result<()> {
        self.validate_funding_tx()?;
// TODO: the validation here is that the script is the expected one,
// e.g. the correct script for a specific contract or contract type
// the script might depend on data not made explicit in the contract,
// e.g. a list of tokens coinciding with payout transactions
// therefore validation in this sense is not always possible without
// data from another party and such validation should occur elsewhere
//        self.validate_payout_script()?;
        self.validate_sigs()?;
        Ok(())
    }

    fn validate_funding_tx(&self) -> Result<()> {
        self.amount()?;
        self.fee_address()?;
        Ok(())
    }

// TODO: payout addresses would need to be included in contract for this validation to carry
// through with flexible payout addresses, e.g. not derived from player pubkey
//    fn validate_payout_script(&self) -> Result<()> {
//        let payout_script = create_payout_script(
//            &create_escrow_address(&self.p1_pubkey, &self.p2_pubkey, &self.arbiter_pubkey, NETWORK).unwrap(),
//            &self.p1_payout_address,
//            &self.p2_payout_address,
//            &self.funding_tx.clone().extract_tx(),
//        );
//        if self.payout_script != payout_script {
//            Err(Error::Adhoc("invalid payout script"))
//        } else {
//            Ok(())
//        }
//    }

    fn validate_sigs(&self) -> Result<()> {
        let secp = Secp256k1::new();
        let msg = Message::from_slice(&self.cxid()).unwrap();
        for (i, sig) in self.sigs.iter().enumerate() {
            let pubkey = match i {
                0 => self.p1_pubkey.key,
                1 => self.p2_pubkey.key,
                2 => self.arbiter_pubkey.key,
                _ => return Err(Error::Adhoc("too many signatures")),
            };
            if secp.verify(&msg, &sig, &pubkey).is_err() {
                return Err(Error::Adhoc("invalid signature"))
            }
        };
        Ok(())
    }
//
}
pub fn contract(input: &[u8]) ->IResult<&[u8], Contract> {
    let (input, (
        version,
        p1_pubkey, 
        p2_pubkey, 
        arbiter_pubkey, 
        p1_payout_address,
        p2_payout_address,
        funding_tx, 
        payout_script, 
        sigs
    )) = tuple((version, pubkey, pubkey, pubkey, address, address, funding_tx, payout_script, sigs))(input)?; 

    let c = Contract {
        version,
        p1_pubkey,
        p2_pubkey,
        arbiter_pubkey,
        p1_payout_address,
        p2_payout_address,
        funding_tx,
        payout_script,
        sigs,
    };

    Ok((input, c))
}

pub fn version(input: &[u8]) -> IResult<&[u8], u8> {
    be_u8(input)
}

fn pubkey(input: &[u8]) -> IResult<&[u8], PublicKey> {
    let (input, b) = take(33u8)(input)?;
    let key = PublicKey::from_slice(&b).unwrap();
    Ok((input, key))
}

fn address(input: &[u8]) -> IResult<&[u8], Address> {
    let (input, b) = length_data(be_u32)(input)?;
    let address = Address::from_str(&String::from_utf8(b.to_vec()).unwrap()).unwrap();
    Ok((input, address))
}

fn funding_tx(input: &[u8]) -> IResult<&[u8], PartiallySignedTransaction> {
    let (input, b) = length_data(be_u32)(input)?;
    let tx = PartiallySignedTransaction::consensus_decode(b).unwrap();
    Ok((input, tx))
}

fn payout_script(input: &[u8]) -> IResult<&[u8], TgScript> {
    length_value(be_u32, tg_script)(input)
}

fn sigs(input: &[u8]) -> IResult<&[u8], Vec<Signature>> {
    many0(signature)(input)
}

pub fn signature(input: &[u8]) -> IResult<&[u8], Signature> {
    let (input, b) = length_data(be_u8)(input)?;
    let sig = Signature::from_der(b).unwrap();
    Ok((input, sig))
}

// TODO: add version
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ContractRecord {
    pub cxid:           String,
    pub p1_name:        PlayerName,
    pub p2_name:        PlayerName,
    pub hex:            String,
    pub desc:           String,
}

#[derive(Debug, PartialEq)]
pub enum ContractState {
    Unsigned,
    P1Signed,
    P2Signed,
    ArbiterSigned,
    Live,
    Resolved,
    Invalid,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct PlayerContractInfo {
    pub name: PlayerName,
    pub escrow_pubkey: PublicKey,
    pub change_address: Address,
    pub payout_address: Address,
    pub utxos: Vec<(OutPoint, u64, Input)>,
}

impl PlayerContractInfo {
    pub fn hash(&self) -> Vec<u8> {
        let mut engine = ShaHashEngine::default();
        engine.input(self.name.0.as_bytes());
        engine.input(&self.escrow_pubkey.to_bytes());
        engine.input(&self.change_address.to_string().as_bytes());
        for (outpoint, _, _) in self.utxos.clone() {
            engine.input(outpoint.txid.as_inner());
            engine.input(&Vec::from(outpoint.vout.to_be_bytes()));
        }

        let hash: &[u8] = &ShaHash::from_engine(engine);
        Vec::from(hash)
    }
}
