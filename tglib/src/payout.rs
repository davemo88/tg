use serde::{
    Serialize,
    Deserialize,
};
use byteorder::{BigEndian, WriteBytesExt};
use nom::{
    IResult,
    combinator::opt,
    multi::{
        length_data,
        length_value,
    },
    number::complete::be_u32,
    sequence::tuple,
};
use bdk::bitcoin::{
    Address,
    consensus::{
        self,
        Decodable,
    },
    util::psbt::PartiallySignedTransaction,
    secp256k1::Signature,
};
use crate::{
    Result,
    Error,
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

#[derive(Clone, Deserialize, Serialize, Debug)]
pub struct Payout {
    pub version:         u8,
    pub contract:        Contract,
    pub psbt:            PartiallySignedTransaction,
    pub script_sig:      Option<Signature>,
}

impl Payout {
//    pub fn new(contract: Contract, tx: Transaction) -> Self {
    pub fn new(contract: Contract, psbt: PartiallySignedTransaction) -> Self {
        Payout {
            version: PAYOUT_VERSION,
            contract,
            psbt,
            script_sig: None,
        }
    }

    pub fn to_bytes(&self) -> Vec<u8> {
        let mut v = Vec::new();
        let _ = v.write_u8(self.version);
// contract length + contract
        let contract_bytes = self.contract.to_bytes();
        v.write_u32::<BigEndian>(contract_bytes.len() as u32).unwrap();
        v.extend(contract_bytes);
// payout tx length  + bytes
        let payout_tx = consensus::serialize(&self.psbt);
        v.write_u32::<BigEndian>(payout_tx.len() as u32).unwrap();
        v.extend(payout_tx);
// payout script sig
        if let Some(sig) = self.script_sig {
            let sig_bytes = sig.serialize_der().to_vec();
            v.write_u8(sig_bytes.len() as u8).unwrap();
            v.extend(sig_bytes);
        }
        v

    }

    pub fn from_bytes(bytes: Vec<u8>) -> Result<Payout> {
        let (i, p) = payout(&bytes).unwrap();
        if i.len() == 0 {
            return Ok(p)
        }
        else {
            return Err(Error::Adhoc("couldn't parse payout"))
        }
    }

    pub fn address(&self) -> Result<Address> {
        let amount = self.contract.amount()?.as_sat() - crate::wallet::TX_FEE;
        let tx = self.psbt.clone().extract_tx();
        for txout in tx.output {
            if txout.value == amount {
                return Ok(Address::from_script(&txout.script_pubkey, NETWORK).unwrap())
            }
        };
        Err(Error::Adhoc("couldn't determine payout address"))
    }
}

fn payout(input: &[u8]) ->IResult<&[u8], Payout> {
    let (input, (
        version,
        contract,
        psbt,
        script_sig,
    )) = tuple((
        version, 
        length_value(be_u32, contract), 
        payout_psbt, 
        opt(signature)
    ))(input)?;
//    let (input, version) = version(input).unwrap();
//    let (input, contract) = length_value(be_u32, contract)(input).unwrap();
//    let (input, psbt) = length_value(be_u32, payout_psbt)(input).unwrap();
//    let (input, script_sig) = opt(signature)(input).unwrap();
//        contract,
//        psbt,
//        script_sig,
//    )) = tuple((
//        version, 
//        length_value(be_u32, contract), 
//        length_value(be_u32, payout_psbt), 
//        opt(signature)
//    ))(input)?;

    let p = Payout {
        version,
        contract,
        psbt,
        script_sig,
    };

    Ok((input, p))
}

fn payout_psbt(input: &[u8]) ->IResult<&[u8], PartiallySignedTransaction> {
    let (input, b) = length_data(be_u32)(input)?;
    let psbt = PartiallySignedTransaction::consensus_decode(b).unwrap();
    Ok((input, psbt))
}

pub enum PayoutState {
    Unsigned,
    PlayerSigned,
    ArbiterSigned,
    Live,
    Resolved,
    Invalid,
}
