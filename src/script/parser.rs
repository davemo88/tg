use std::convert::TryInto;
use nom::{
    self,
    IResult,
    bytes::complete::{take, take_until, tag},
    number::complete::{le_u8, be_u8, be_u16, be_u32},
    branch::alt,
    multi::{many1, length_data},
    combinator::opt,
    sequence::{tuple, preceded},
    InputIter,
    InputTake,
    InputLength,
    ToUsize,
    error::ParseError,
};
use crate::{
    script::lib::{
        TgOpcode,
        TgScript,
    },
};

fn op_bytecode(op: TgOpcode) -> impl Fn(&[u8]) -> IResult<&[u8], TgOpcode> {
    move |input: &[u8]| {
        let (input, b) = take(1u8)(input)?;
        if b[0] == op.bytecode() {
            Ok((input, op.clone()))
        }
        else{
            Err(nom::Err::Error((input, nom::error::ErrorKind::IsNot)))
        }
    }
}

fn op_0(input: &[u8]) -> IResult<&[u8], TgOpcode> {
    op_bytecode(TgOpcode::OP_0)(input)
}

fn op_1(input: &[u8]) -> IResult<&[u8], TgOpcode> {
    op_bytecode(TgOpcode::OP_1)(input)
}

fn op_drop(input: &[u8]) -> IResult<&[u8], TgOpcode> {
    op_bytecode(TgOpcode::OP_DROP)(input)
}

fn op_dup(input: &[u8]) -> IResult<&[u8], TgOpcode> {
    op_bytecode(TgOpcode::OP_DUP)(input)
}

fn op_equal(input: &[u8]) -> IResult<&[u8], TgOpcode> {
    op_bytecode(TgOpcode::OP_EQUAL)(input)
}

fn op_if(input: &[u8]) -> IResult<&[u8], TgOpcode> {
    let (input, (op_if, true_branch, else_block, op_endif)) = tuple((
            op_bytecode(TgOpcode::OP_IF(TgScript::default(),None)),
            tg_script,
            opt(op_else),
            op_bytecode(TgOpcode::OP_ENDIF),
            ))(input)?;
    let mut false_branch = None;
    if let Some(else_op) = else_block {
        if let TgOpcode::OP_ELSE(script) = else_op {
           false_branch = Some(script); 
        }
    }
    Ok((input, TgOpcode::OP_IF(true_branch,false_branch)))
}

fn op_else(input: &[u8]) -> IResult<&[u8], TgOpcode> {
    let (input, else_script) = preceded(
        op_bytecode(TgOpcode::OP_ELSE(TgScript::default())),
        tg_script,
    )(input)?;
    Ok((input, TgOpcode::OP_ELSE(else_script)))
}

fn op_endif(input: &[u8]) -> IResult<&[u8], TgOpcode> {
    op_bytecode(TgOpcode::OP_ENDIF)(input)
}

fn op_pushdata1(input: &[u8]) -> IResult<&[u8], TgOpcode> {
    let (input, data) = preceded(
        op_bytecode(TgOpcode::OP_PUSHDATA1(0,Vec::new())),
        length_data(be_u8),
    )(input)?;
    Ok((input, TgOpcode::OP_PUSHDATA1(data.len().try_into().unwrap(),Vec::from(data))))
}

fn op_pushdata2(input: &[u8]) -> IResult<&[u8], TgOpcode> {
    let (input, data) = preceded(
        op_bytecode(TgOpcode::OP_PUSHDATA2(0,Vec::new())),
        length_data(be_u16),
    )(input)?;
    Ok((input, TgOpcode::OP_PUSHDATA2(data.len().try_into().unwrap(),Vec::from(data))))
}

fn op_pushdata4(input: &[u8]) -> IResult<&[u8], TgOpcode> {
    let (input, data) = preceded(
        op_bytecode(TgOpcode::OP_PUSHDATA4(0,Vec::new())),
        length_data(be_u32),
    )(input)?;
    Ok((input, TgOpcode::OP_PUSHDATA4(data.len().try_into().unwrap(),Vec::from(data))))
}

//fn pushdata_curry<'a, I, N, E, F>(f: F) -> impl Fn(I) -> IResult<I, I, E> 
//where
//I: Clone + InputLength + InputTake,
//N: Copy + ToUsize,
//F: Fn(I) -> IResult<I, N, E>,
//E: ParseError<I>,
//{
//    move|input: I| {
//        let (input, num_bytes) = f(input)?;
//        let (input, (op, data)) = tuple((
//            op_bytecode(TgOpcode::OP_PUSHDATA4(0,Vec::new())),
//            length_data(f),
//        ))(input)?;
//        op_bytecode(TgOpcode::OP_PUSHDATA4(0,Vec::from(data)))(input)
//    }
//}

pub fn tg_script(input: &[u8]) -> IResult<&[u8], TgScript> {
    let (input, ops) = many1(
        alt((
            op_0,
            op_1,
            op_if,
//            op_drop,
//            op_dup,
//            op_equal,
            op_pushdata1,
//            op_pushdata2,
//            op_pushdata4,
        ))
    )(input)?; 

    Ok((input, TgScript(ops)))
}

#[cfg(test)]
mod tests {

    use super::*;

    const PUSHDATA_SCRIPT: &'static[u8] = &[0xD1,0x01,0xFF];//,0xD1,0x02,0x01,0x01];
    const CONDITIONAL_SCRIPT_TRUE: &'static[u8] = &[0x01,0xF1,0x01,0xF2,0x00,0xF3,0xF1,0x00,0xF3];
    const ERROR_SCRIPT: &'static[u8] = &[0xA1];

    #[test]
    fn parser () {
        let (input, script) = op_pushdata1(&PUSHDATA_SCRIPT).unwrap(); 
        println!("{:?}", script);

//        let script = tg_script(&CONDITIONAL_SCRIPT_TRUE); 
//        println!("{:?}", script);
//        assert!(script.is_ok());
    }
}
