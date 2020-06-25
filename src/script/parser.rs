use std::convert::TryInto;
use nom::{
    self,
    IResult,
    bytes::complete::{take},
//    number::complete::{be_u8, be_u16, be_u32},
    branch::alt,
    multi::many1,
    sequence::tuple,
    InputIter,
    InputTake,
};
use crate::{
    script::lib::{
        TgOpcode,
        TgStatement,
        TgScript,
        opcodes::*,
    },
};

fn opcode(op: TgOpcode) -> impl Fn(&[u8]) -> IResult<&[u8], TgOpcode> {
    move |input: &[u8]| {
        let (input, b) = take(1u8)(input)?;
        let potential_op = TgOpcode(b[0]);
        if !VALID_OPCODES.contains(&potential_op) {
//TODO: how to add error message to nom error?
            println!("script contains invalid opcode: {:?}", potential_op);
            return Err(nom::Err::Failure((input, nom::error::ErrorKind::IsNot)))
        }
        if potential_op == op  {
            Ok((input, op))
        }
        else {
            Err(nom::Err::Error((input, nom::error::ErrorKind::IsNot)))
        }
    }
}

fn wrapped_op(op: TgOpcode) -> impl Fn(&[u8]) -> IResult<&[u8], TgStatement> {
    move |input: &[u8]| {
        let (input, op) = opcode(op)(input)?;
        Ok((input, TgStatement::from(op)))
    }
}

fn pushdata(op: TgOpcode) -> impl Fn(&[u8]) -> IResult<&[u8], TgStatement> {
    move |input: &[u8]| {
        let n = match op {
            OP_PUSHDATA1 => 1,
            OP_PUSHDATA2 => 2,
            OP_PUSHDATA4 => 4,
            _ => return Err(nom::Err::Error((input, nom::error::ErrorKind::IsNot))),
        };
        let (input, (op, num_bytes)) = tuple((opcode(op), pushdata_num_bytes(n)))(input)?;
        let (input, bytes) = take(num_bytes)(input)?;
        Ok((input, TgStatement::Data(op, num_bytes, bytes.to_vec())))
    }
}

fn pushdata_num_bytes(n: u8) -> impl Fn(&[u8]) -> IResult<&[u8], u64> {
    move |input: &[u8]| {
// could be replaced with nom::number::be_uX combinators
// but that also creates type woes similar to those below
        let (input, num_bytes) = take(n)(input)?;
        let num_bytes: u64 = match n {
            1 => u8::from_be_bytes(num_bytes.try_into().unwrap()).into(),
            2 => u16::from_be_bytes(num_bytes.try_into().unwrap()).into(),
            4 => u32::from_be_bytes(num_bytes.try_into().unwrap()).into(),
            _ => return Err(nom::Err::Error((input, nom::error::ErrorKind::IsNot))),
        };
        Ok((input, num_bytes))
    }
}

fn if_statement(input: &[u8]) -> IResult<&[u8], TgStatement> {
    let (input, (_, true_branch_script, op)) = tuple((
            opcode(OP_IF),
            tg_script,
            alt((
                opcode(OP_ELSE),
                opcode(OP_ENDIF)
            )),
    ))(input)?;
    match op {
        OP_ELSE => {
            let (input, false_branch_script) = if_statement_false_branch(input)?;
            Ok((input, TgStatement::IfStatement(true_branch_script, Some(false_branch_script))))
        }, 
        OP_ENDIF => Ok((input, TgStatement::IfStatement(true_branch_script, None))),
        _ => return Err(nom::Err::Error((input, nom::error::ErrorKind::IsNot))),
    }
}

fn if_statement_false_branch(input: &[u8]) -> IResult<&[u8], TgScript> {
    let (input, (false_branch_script, _)) = tuple((
            tg_script,
            opcode(OP_ENDIF)
    ))(input)?;
    Ok((input, false_branch_script))
}

fn data_opcode(input: &[u8]) -> IResult<&[u8], TgStatement> {
    alt(
        (
            pushdata(OP_PUSHDATA1),
            pushdata(OP_PUSHDATA2),
            pushdata(OP_PUSHDATA4),
        )
    )(input)
}

fn constant_opcode(input: &[u8]) -> IResult<&[u8], TgStatement> {
    alt(
        (
            wrapped_op(OP_0),
            wrapped_op(OP_1),
        )
    )(input)
}

fn normal_opcode(input: &[u8]) -> IResult<&[u8], TgStatement> {
    alt(
        (
            wrapped_op(OP_DROP),
            wrapped_op(OP_DUP),
        )
    )(input)
}

pub fn tg_script(input: &[u8]) -> IResult<&[u8], TgScript> {
    let (input, ops) = many1(alt((data_opcode, constant_opcode, normal_opcode, if_statement)))(input)?; 
    Ok((input, TgScript(ops)))
}

#[cfg(test)]
mod tests {

    use super::*;

    const PUSHDATA_SCRIPT: &'static[u8] = &[0xD1,0x01,0xFF,0xD1,0x02,0x01,0x01];
    const CONDITIONAL_SCRIPT_TRUE: &'static[u8] = &[0x01,0xF1,0x01,0xF2,0x00,0xF3,0xF1,0x00,0xF3];
    const ERROR_SCRIPT: &'static[u8] = &[0xA1];

    #[test]
    fn parser () {
//        let script = tg_script(&PUSHDATA_SCRIPT); 
//        assert!(script.is_ok());
//        println!("{:?}", script);

        let script = tg_script(&CONDITIONAL_SCRIPT_TRUE); 
        assert!(script.is_ok());
//        println!("{:?}", script);

//        let script = tg_script(&ERROR_SCRIPT); 
//        assert!(script.is_err());
//        println!("{:?}", script);
    }
}
