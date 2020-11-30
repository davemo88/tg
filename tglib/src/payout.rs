use byteorder::{BigEndian, WriteBytesExt};
use nom::{
    IResult,
    combinator::opt,
    multi::length_data,
    number::complete::be_u32,
    sequence::tuple,
};
use bdk::bitcoin::{
    Address,
    Transaction,
    consensus::{
        self,
        encode::Decodable,
    },
    secp256k1::Signature,
};
use crate::{
    Result,
    TgError,
    contract::{
        Contract,
        contract,
        signature,
        version,
    },
    mock::{
        NETWORK,
        PAYOUT_VERSION,
    }
};

#[derive(Clone, Debug)]
pub struct Payout {
    pub version:         u8,
    pub contract:        Contract,
    pub tx:              Transaction,
    pub script_sig:      Option<Signature>,
}

impl Payout {
    pub fn new(contract: Contract, tx: Transaction) -> Self {
        Payout {
            version: PAYOUT_VERSION,
            contract,
            tx,
            script_sig: None,
        }
    }

    pub fn to_bytes(&self) -> Vec<u8> {
        let mut v = Vec::new();
        let _ = v.write_u8(self.version);
// payout length + contract
        let contract_bytes = self.contract.to_bytes();
        v.write_u32::<BigEndian>(contract_bytes.len() as u32).unwrap();
        v.extend(contract_bytes);
// payout tx length  + bytes
        let payout_tx = consensus::serialize(&self.tx);
        v.write_u32::<BigEndian>(payout_tx.len() as u32).unwrap();
        v.extend(payout_tx);
// payout script sig
        if let Some(sig) = self.script_sig {
            v.extend(sig.serialize_compact().to_vec());
        }
        v

    }

    pub fn from_bytes(bytes: Vec<u8>) -> Result<Payout> {
        let (i, p) = payout(&bytes).unwrap();
        if i.len() == 0 {
            Ok(p)
        }
        else {
            Err(TgError("couldn't parse contract"))
        }
    }

    pub fn address(&self) -> Result<Address> {
        let amount = self.contract.amount()?;
        for txout in self.tx.output.clone() {
            if txout.value == amount.as_sat() {
                return Ok(Address::from_script(&txout.script_pubkey, NETWORK).unwrap())
            }
        };
        Err(TgError("couldn't determine payout address"))
    }
}

fn payout(input: &[u8]) ->IResult<&[u8], Payout> {
    let (input, (
        version,
        contract,
        tx,
        script_sig,
    )) = tuple((version, contract, payout_tx, opt(signature)))(input)?;

    let p = Payout {
        version,
        contract,
        tx,
        script_sig,
    };

    Ok((input, p))
}

fn payout_tx(input: &[u8]) ->IResult<&[u8], Transaction> {
    let (input, b) = length_data(be_u32)(input)?;
    let tx = Transaction::consensus_decode(b).unwrap();
    Ok((input, tx))
    
}

pub enum PayoutState {
    Unsigned,
    PlayerSigned,
    ArbiterSigned,
    Live,
    Resolved,
    Invalid,
}

