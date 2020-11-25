
use std::convert::TryInto;
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

#[allow(dead_code)]
fn op_0(input: &[u8]) -> IResult<&[u8], TgOpcode> {
    op_bytecode(TgOpcode::OP_0)(input)
}

#[allow(dead_code)]
fn op_1(input: &[u8]) -> IResult<&[u8], TgOpcode> {
    op_bytecode(TgOpcode::OP_1)(input)
}

#[allow(dead_code)]
fn op_drop(input: &[u8]) -> IResult<&[u8], TgOpcode> {
    op_bytecode(TgOpcode::OP_DROP)(input)
}

#[allow(dead_code)]
fn op_dup(input: &[u8]) -> IResult<&[u8], TgOpcode> {
    op_bytecode(TgOpcode::OP_DUP)(input)
}

#[allow(dead_code)]
fn op_2dup(input: &[u8]) -> IResult<&[u8], TgOpcode> {
    op_bytecode(TgOpcode::OP_2DUP)(input)
}

#[allow(dead_code)]
fn op_equal(input: &[u8]) -> IResult<&[u8], TgOpcode> {
    op_bytecode(TgOpcode::OP_EQUAL)(input)
}

#[allow(dead_code)]
fn op_verifysig(input: &[u8]) -> IResult<&[u8], TgOpcode> {
    op_bytecode(TgOpcode::OP_VERIFYSIG)(input)
}

#[allow(dead_code)]
fn op_validate(input: &[u8]) -> IResult<&[u8], TgOpcode> {
    op_bytecode(TgOpcode::OP_VALIDATE)(input)
}

#[allow(dead_code)]
fn op_if(input: &[u8]) -> IResult<&[u8], TgOpcode> {
    let (input, (true_branch, else_block)) = tuple((
        preceded(
// TODO: this is a bit of a wart, creating a script to get the bytecode
            op_bytecode(TgOpcode::OP_IF(TgScript::default(),None)),
            tg_script,
        ),
        terminated(
            opt(op_else),
            op_bytecode(TgOpcode::OP_ENDIF),
        ),
    ))(input)?;
    let false_branch = if let Some(else_op) = else_block {
        match else_op {
            TgOpcode::OP_ELSE(script) => Some(script),
            _ => None,
        }
    }
    else {
        None
    };
    Ok((input, TgOpcode::OP_IF(true_branch,false_branch)))
}

fn op_else(input: &[u8]) -> IResult<&[u8], TgOpcode> {
    let (input, else_script) = preceded(
        op_bytecode(TgOpcode::OP_ELSE(TgScript::default())),
        tg_script,
    )(input)?;
    Ok((input, TgOpcode::OP_ELSE(else_script)))
}

#[allow(dead_code)]
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

// make a pushdata function that can be curried with a number parser to get the right size data
// e.g. be_u8 to grab a slice with length given by a big endian u8, be_u16 for u16, be_u32, etc
// fn pushdata_curry<'a, I, N, E, F>(f: F) -> impl Fn(I) -> IResult<I, I, E> 
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
            op_drop,
            op_dup,
            op_2dup,
            op_equal,
            op_verifysig,
            op_validate,
            op_pushdata1,
            op_pushdata2,
            op_pushdata4,
        ))
    )(input)?; 

    Ok((input, TgScript(ops)))
}

#[cfg(test)]
mod tests {

    use super::*;

    const PUSHDATA_SCRIPT: &'static[u8] = &[0xD1,0x01,0xFF];//,0xD1,0x02,0x01,0x01];
    const CONDITIONAL_SCRIPT_TRUE: &'static[u8] = &[0x01,0xF1,0x01,0xF2,0x00,0xF3,0xF1,0x00,0xF3];
    const _ERROR_SCRIPT: &'static[u8] = &[0xA1];

    #[test]
    fn parser () {
        let (_input, script) = op_pushdata1(&PUSHDATA_SCRIPT).unwrap(); 
        println!("{:?}", script);

        let script = tg_script(&CONDITIONAL_SCRIPT_TRUE); 
        println!("{:?}", script);
        assert!(script.is_ok());
    }
}
