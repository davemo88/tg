use std::convert::AsRef;
use byteorder::{BigEndian, WriteBytesExt};
use serde::{Serialize, Deserialize,};
use bdk::{
    bitcoin::{
        Address,
        Amount,
        Transaction,
        PublicKey,
        consensus::{
            self,
            encode::Decodable,
        },
        hashes::{
            Hash,
            HashEngine,
            sha256::Hash as ShaHash,
            sha256::HashEngine as ShaHashEngine,
        },
        secp256k1::{
            Signature,
        },
    },
    UTXO,
};
use nom::{
    self,
    IResult,
    bytes::complete::take,
    number::complete::{be_u8, be_u16, be_u32},
    branch::alt,
    multi::{many1, length_data},
    combinator::opt,
    sequence::{tuple, preceded, terminated},
};

use crate::{
    Result,
    TgError,
    arbiter::ArbiterId,
    player::PlayerId,
    script::{
        parser::tg_script,
        TgScript,
    }
};

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Contract {
    pub p1_pubkey:          PublicKey,
    pub p2_pubkey:          PublicKey,
    pub arbiter_pubkey:     PublicKey,
    pub funding_tx:         Transaction,
    pub payout_script:      TgScript,
    pub sigs:               Vec<Signature>, 
}

impl Contract {
    pub fn new(p1_pubkey: PublicKey, p2_pubkey: PublicKey, arbiter_pubkey: PublicKey, funding_tx: Transaction, payout_script: TgScript) -> Self {
        Contract {
            p1_pubkey,
            p2_pubkey,
            arbiter_pubkey,
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

// TODO: should these functions be more explicit and specific
    pub fn to_bytes(&self) -> Vec<u8> {
        let mut v = Vec::new();
// 3 fixed-length pubkeys
        v.extend(self.p1_pubkey.to_bytes());
        v.extend(self.p2_pubkey.to_bytes());
        v.extend(self.arbiter_pubkey.to_bytes());
// funding tx length  + bytes
        let funding_tx = consensus::serialize(&self.funding_tx);
        v.write_u32::<BigEndian>(funding_tx.len() as u32).unwrap();
        v.extend(funding_tx);
// payout script length  + bytes
        println!("payout script going to bytes: {:?}", self.payout_script);
        let payout_script = Vec::from(self.payout_script.clone());
        println!("payout script as bytes({}): {:?}", payout_script.len(), payout_script);
        v.write_u32::<BigEndian>(payout_script.len() as u32).unwrap();
        v.extend(payout_script);
        for sig in &self.sigs {
            v.extend(sig.to_string().into_bytes());
        }
        v
    }

    pub fn from_bytes(bytes: Vec<u8>) ->Contract {
        let (i, c) = contract(&bytes).unwrap();
        assert_eq!(i.len(),0);
        c
    }
//
}
fn contract(input: &[u8]) ->IResult<&[u8], Contract> {
    let (input, (
        p1_pubkey, 
        p2_pubkey, 
        arbiter_pubkey, 
        funding_tx, 
        payout_script, 
        sigs
    )) = tuple((pubkey, pubkey, pubkey, funding_tx, payout_script, sigs))(input)?; 

    let c = Contract {
        p1_pubkey,
        p2_pubkey,
        arbiter_pubkey,
        funding_tx,
        payout_script,
        sigs,
    };

    Ok((input, c))
    
}

fn pubkey(input: &[u8]) -> IResult<&[u8], PublicKey> {
    let (input, b) = take(33u8)(input)?;
    let key = PublicKey::from_slice(&b).unwrap();
    Ok((input, key))
}

fn funding_tx(input: &[u8]) -> IResult<&[u8], Transaction> {
    let (input, b) = length_data(be_u32)(input)?;
    let tx = Transaction::consensus_decode(b).unwrap();
    Ok((input, tx))
}

fn payout_script(input: &[u8]) -> IResult<&[u8], TgScript> {
    let (input, b) = length_data(be_u32)(input)?;
    let (input, script) = tg_script(&b).unwrap();
    println!("input: {:?} script{:?}", input, script);
    Ok((input, script))
}

fn sigs(input: &[u8]) -> IResult<&[u8], Vec<Signature>> {
// TODO: oops
    Ok((input, Vec::new()))
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

pub struct PlayerContractInfo {
    pub escrow_pubkey: PublicKey,
    pub change_address: Address,
    pub utxos: Vec<UTXO>,
}
