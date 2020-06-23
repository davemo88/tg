use std::convert::TryInto;
use nom::{
    self,
    IResult,
    Err,
    switch,
    bytes::complete::{tag, take, take_while_m_n},
    number::complete::be_u16,
    branch::alt,
    multi::many1,
    combinator::map_parser,
    sequence::tuple,
};
use hex;
use crate::{
    script::lib::{
        TgOpcode,
        opcodes::*,
    },
};

#[derive(Debug)]
pub enum OpcodeOrData<'a> {
    Opcode(TgOpcode),
    Data(TgOpcode, &'a [u8], &'a [u8]),
}

impl From<TgOpcode> for OpcodeOrData<'_> {
    fn from(opcode: TgOpcode) -> Self {
        OpcodeOrData::Opcode(opcode)
    }
}

#[derive(Debug)]
struct TgScript<'a>(pub Vec<OpcodeOrData<'a>>);

struct TgScriptParser; 

impl TgScriptParser {


}

fn tg_script(input: &[u8]) -> IResult<&[u8], TgScript> {
   many1(pushdata1)(input); 
//    let (input, v: Vec<OpcodeOrData>) = multi::opcode(input, None);
//    while script.0.len() < 10 {
//        println!("{:?}", hex::encode(i));
//        let (i, op) = opcode(i, None)?; 
//        println!("{:?}", hex::encode(i));
//        script.0.push(OpcodeOrData::from(op.clone()));
//        match op {
//            OP_PUSHDATA1 => {
//                let (i, data) = pushdata1_data(i)?;
//                script.0.push(data);
//            },
//            OP_PUSHDATA2 => {
//                let (i, data) = pushdata1_data(i)?;
//                script.0.push(data);
//            },
//            OP_PUSHDATA4 => {
//                let (i, data) = pushdata1_data(i)?;
//                script.0.push(data);
//            },
//            _ => (),
//        }
//        println!("{:?}", hex::encode(i));
//        println!("{:?}", script);
//    }
    Ok((input, TgScript(Vec::new())))
}

fn take1(input: &[u8]) -> IResult<&[u8], u8> {
    let (input, b) = take(1u8)(input)?;
    Ok((input, b[0]))
}

fn op_pushdata1(input: &[u8]) -> IResult<&[u8], TgOpcode> {
    opcode(input, Some(OP_PUSHDATA1))
}

fn pushdata1_num_bytes(input: &[u8]) -> IResult<&[u8], u64> {
    let (input, num_bytes) = take(1u8)(input)?;
    let num_bytes = u8::from_be_bytes(num_bytes.try_into().unwrap());
    Ok((input, num_bytes.into()))
}

fn pushdata1(input: &[u8]) -> IResult<&[u8], OpcodeOrData> {
    let (input, (op, num_bytes)) = tuple((op_pushdata1, pushdata1_num_bytes))(input)?;
    let (input, bytes) = take(num_bytes)(input)?;
    println!("op: {:?} num bytes: {:?} bytes: {:?}", op, num_bytes, hex::encode(bytes));

    Ok((input, OpcodeOrData::Opcode(TgOpcode(0x00))))
}

//fn pushdata2_data(input: &[u8]) -> IResult<&[u8], OpcodeOrData> {
//    let (i, num_bytes) = take(2u8)(input)?;
//    let num_bytes = u16::from_be_bytes(num_bytes.try_into().unwrap());
//    let (i, bytes) = take(num_bytes)(i)?;
//    Ok((i, OpcodeOrData::Data(bytes)))
//}
//
//fn pushdata4_data(input: &[u8]) -> IResult<&[u8], OpcodeOrData> {
//    let (i, num_bytes) = take(4u8)(input)?;
//    let num_bytes = u64::from_be_bytes(num_bytes.try_into().unwrap());
//    let (i, bytes) = take(num_bytes)(i)?;
//    Ok((i, OpcodeOrData::Data(bytes)))
//}

fn opcode(input: &[u8], opcode: Option<TgOpcode>) -> IResult<&[u8], TgOpcode> {
    let (input, b) = take1(input)?;
    if let Some(opcode) = opcode {
        if opcode.is_valid() && opcode.0 == b {
            return Ok((input,opcode));
        }
    } 
    else {
        let opcode = TgOpcode(b);
        if opcode.is_valid() {
            return Ok((input, opcode));
        }
    }
    Err(nom::Err::Error((input, nom::error::ErrorKind::IsNot)))
}

//fn pushdata_num_bytes_2(input: &[u8]) -> IResult<&[u8], u64> {
//    take(2u8)(input)
//}

//fn pushdata_num_bytes_4(input: &[u8]) -> IResult<&[u8], u64> {
//    take(4u8)(input)
//}

fn pushdata_opcode(input: &[u8], opcode: TgOpcode) -> IResult<&[u8], &[u8]> {
    match opcode {
        OP_PUSHDATA1 => {
            Ok((input, input))
        },
        OP_PUSHDATA1 => {
            Ok((input, input))
        },
        OP_PUSHDATA1 => {
            Ok((input, input))
        },
        _ => Err(nom::Err::Error((input, nom::error::ErrorKind::IsNot)))
    }
}

#[cfg(test)]
mod tests {

    use super::*;

    const PUSHDATA_SCRIPT: &'static[u8] = &[0xD1,0x01,0xFF,0xD1,0x02,0x01,0x01];

    #[test]
    fn pushdata() {
        tg_script(&PUSHDATA_SCRIPT).unwrap(); 
    }
    

}


