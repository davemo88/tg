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
    PublicKey,
    consensus::{
        self,
        Decodable,
    },
    util::psbt::PartiallySignedTransaction,
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
//    pub tx:              Transaction,
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
// payout length + contract
        let contract_bytes = self.contract.to_bytes();
        v.write_u32::<BigEndian>(contract_bytes.len() as u32).unwrap();
        v.extend(contract_bytes);
// payout tx length  + bytes
        let payout_tx = consensus::serialize(&self.psbt);
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
        let tx = self.psbt.clone().extract_tx();
        for txout in tx.output {
            if txout.value == amount.as_sat() {
                return Ok(Address::from_script(&txout.script_pubkey, NETWORK).unwrap())
            }
        };
        Err(TgError("couldn't determine payout address"))
    }

    pub fn recipient_pubkey(&self) -> Result<PublicKey> {
        let address = self.address()?;
        if address == Address::p2wpkh(&self.contract.p1_pubkey, address.network).unwrap() {
            Ok(self.contract.p1_pubkey.clone())
        } else if address == Address::p2wpkh(&self.contract.p2_pubkey, address.network).unwrap() {
            Ok(self.contract.p2_pubkey.clone())
        } else {
            Err(TgError("couldn't determine recipient pubkey"))
        }
    }
}

fn payout(input: &[u8]) ->IResult<&[u8], Payout> {
    let (input, (
        version,
        contract,
        psbt,
        script_sig,
    )) = tuple((version, contract, payout_psbt, opt(signature)))(input)?;

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
