use std::convert::TryInto;
use nom::{
    self,
    IResult,
    Err,
    switch,
    bytes::complete::{tag, take},
    number::complete::be_u16,
    branch::alt,
    multi::many1,
    combinator::map_parser,
    sequence::tuple,
    InputIter,
    InputTake,
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
    Data(TgOpcode, u64, &'a [u8]),
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
    let (input, ops) = many1(alt((data_opcode, constant_opcode, normal_opcode)))(input)?; 
    Ok((input, TgScript(ops)))
}

fn opcode(op: TgOpcode) -> impl Fn(&[u8]) -> IResult<&[u8], TgOpcode> {
    move |input: &[u8]| {
        let (input, b) = take(1u8)(input)?;
        if TgOpcode(b[0]) == op  {
            return Ok((input, op));
        }
        Err(nom::Err::Error((input, nom::error::ErrorKind::IsNot)))
    }
}

fn wrapped_op(op: TgOpcode) -> impl Fn(&[u8]) -> IResult<&[u8], OpcodeOrData> {
    move |input: &[u8]| {
        let (input, op) = opcode(op)(input)?;
        Ok((input, OpcodeOrData::from(op)))
    }
}

fn pushdata(op: TgOpcode) -> impl Fn(&[u8]) -> IResult<&[u8], OpcodeOrData> {
    move |input: &[u8]| {
        let n = match op {
            OP_PUSHDATA1 => 1,
            OP_PUSHDATA2 => 2,
            OP_PUSHDATA4 => 4,
            _ => return Err(nom::Err::Error((input, nom::error::ErrorKind::IsNot))),
        };
        let (input, (op, num_bytes)) = tuple((opcode(op), pushdata_num_bytes(n)))(input)?;
        let (input, bytes) = take(num_bytes)(input)?;
        Ok((input, OpcodeOrData::Data(op, num_bytes, bytes)))
    }
}

fn pushdata_num_bytes(bytes: u8) -> impl Fn(&[u8]) -> IResult<&[u8], u64> {
    move |input: &[u8]| {
        let (input, num_bytes) = take(bytes)(input)?;
        let num_bytes: u64 = match bytes {
            1 => u8::from_be_bytes(num_bytes.try_into().unwrap()).into(),
            2 => u16::from_be_bytes(num_bytes.try_into().unwrap()).into(),
            4 => u32::from_be_bytes(num_bytes.try_into().unwrap()).into(),
            _ => return Err(nom::Err::Error((input, nom::error::ErrorKind::IsNot))),
        };
        Ok((input, num_bytes))
    }
}

fn data_opcode(input: &[u8]) -> IResult<&[u8], OpcodeOrData> {
    alt(
        (
            pushdata(OP_PUSHDATA1),
            pushdata(OP_PUSHDATA2),
            pushdata(OP_PUSHDATA4),
        )
    )(input)
}

fn constant_opcode(input: &[u8]) -> IResult<&[u8], OpcodeOrData> {
    alt(
        (
            wrapped_op(OP_0),
            wrapped_op(OP_1),
        )
    )(input)
}

fn normal_opcode(input: &[u8]) -> IResult<&[u8], OpcodeOrData> {
    alt(
        (
            wrapped_op(OP_DROP),
            wrapped_op(OP_DUP),
            wrapped_op(OP_IF),
            wrapped_op(OP_ELSE),
            wrapped_op(OP_ENDIF),
        )
    )(input)
}

#[cfg(test)]
mod tests {

    use super::*;

    const PUSHDATA_SCRIPT: &'static[u8] = &[0xD1,0x01,0xFF,0xD1,0x02,0x01,0x01];
    const CONDITIONAL_SCRIPT: &'static[u8] = &[0xD1,0x01,0x01,0xF1,0x01,0xF2,0x00,0xF3];
    const ERROR_SCRIPT: &'static[u8] = &[0xA1];

    #[test]
    fn pushdata() {
        let script = tg_script(&PUSHDATA_SCRIPT).unwrap(); 
        println!("{:?}", script);
        let script = tg_script(&CONDITIONAL_SCRIPT).unwrap(); 
        println!("{:?}", script);
//      let script = tg_script(&ERROR_SCRIPT).unwrap(); 
//        println!("{:?}", script);
    }
    

}


