use std::convert::TryInto;
use crate::{
    script::{
        lib::{
            TgOpcode,
            OpcodeOrData,
            TgScript,
            opcodes::*,
        },
        parser::{
            tg_script,
        }
    },
    lib::{
        PayoutScript,
        PayoutRequest,
        Result,
        TgError,
    }
};

struct TgScriptEnv {
//    payout_request: Option<PayoutRequest>,
    stack: Vec<Vec<u8>>,
// one level of recursion 
    in_if: bool,
    script: TgScript,
}

impl From<TgScript> for TgScriptEnv {
    fn from(script: TgScript) -> Self {
        TgScriptEnv {
//            payout_request: None,
            script,
            stack: Vec::new(),
            in_if: false,
        }
    }
}

trait TgScriptInterpreter {
    fn eval(&mut self) ->Result<()> { Err(TgError("")) }
//NOTE: opcode functions - in own trait?
    fn op_pushdata1(&mut self, n:u8, bytes: Vec<u8>) {}
    fn op_pushdata2(&mut self, n:u16, bytes: Vec<u8>) {}
    fn op_pushdata4(&mut self, n:u32, bytes: Vec<u8>) {}
    fn op_0(&mut self) {}
    fn op_1(&mut self) {}
    fn op_dup(&mut self) {}
    fn op_drop(&mut self) {}
    fn op_if(&mut self) {}
    fn op_else(&mut self) {}
    fn op_endif(&mut self) {}
    fn op_equal(&mut self) {}
    fn op_nequal(&mut self) {}
    fn op_and(&mut self) {}
    fn op_or(&mut self) {}
    fn op_xor(&mut self) {}
}

impl TgScriptInterpreter for TgScriptEnv {
    fn eval(&mut self) -> Result<()> {
        self.script.0.reverse();
        let mut next: OpcodeOrData; 
        while self.script.0.len() > 0 {
            next = self.script.0.pop().unwrap();    
            println!("{:?}", next);
            match next {
                OpcodeOrData::Opcode(op) => match op {
                    OP_0 => self.op_0(),
                    OP_1 => self.op_1(),
                    OP_DUP => self.op_dup(),
                    OP_2DUP => self.op_dup(),
                    OP_DROP => self.op_drop(),
                    OP_IF => self.op_if(),
                    OP_ELSE => self.op_else(),
                    OP_ENDIF => self.op_endif(),
                    _ => return Err(TgError("Bad Opcode")),
                },
                OpcodeOrData::Data(op, n, bytes) => match op {
                    OP_PUSHDATA1 => self.op_pushdata1(n.try_into().unwrap(), bytes),
                    OP_PUSHDATA2 => self.op_pushdata2(n.try_into().unwrap(), bytes),
                    OP_PUSHDATA4 => self.op_pushdata4(n.try_into().unwrap(), bytes),
                    _ => return Err(TgError("Bad Data")),
                }
            }
            println!("{:?}", self.stack);
        }
        Ok(())
    }

    fn op_0(&mut self) {
        self.stack.push(vec![0u8])
    }

    fn op_1(&mut self) {
        self.stack.push(vec![1u8])
    }

    fn op_if(&mut self) {
// true branch, continue executing
// everything except [0x00] is true
        if self.stack.pop().unwrap() != vec![0u8] {
            if !self.in_if {
                self.in_if = true;
            }
            else {
                panic!("op_if: tried to enter in_if state twice");
            }
        }
// false branch, ignore opcodes until OP_ELSE or OP_ENDIF
        else {
            loop {
                let next = self.script.0.last().unwrap();
                match next {
                    OpcodeOrData::Opcode(OP_ELSE) => break,
// set in_if = true to allow for check in op_endif()
// we set in_if at beginning of conditional code execution, i.e.
// if / else. however if condition is false and there is no else,
// we jump to endif without executing conditional code
                    OpcodeOrData::Opcode(OP_ENDIF) => { self.in_if = true; break; },
                    _ => { self.script.0.pop(); },
                }
            } 
        }
    }

    fn op_else(&mut self) {
// on the true branch, already executed conditional code, ignore everything until OP_ENDIF
        if self.in_if {
            loop {
                let next = self.script.0.last().unwrap();
                match next {
                  OpcodeOrData::Opcode(OP_ENDIF) => break,
                    _ => { self.script.0.pop(); },
                }
            }
        }
        else {
// on the false branch, execute the else code
            self.in_if = true;
        }
    }

    fn op_endif(&mut self) {
// want this check to disallow OP_ENDIF without a preceding OP_IF
        if self.in_if {
            self.in_if = false;
        }
        else {
            panic!("op_endif: tried leave in_if state but not in it");
        }
    }

    fn op_pushdata1(&mut self, n: u8, bytes: Vec<u8>) {
        self.pushdata(n as usize, bytes);
    }

    fn op_pushdata2(&mut self, n: u16, bytes: Vec<u8>) {
        self.pushdata(n as usize, bytes);
    }

    fn op_pushdata4(&mut self, n: u32, bytes: Vec<u8>) {
        self.pushdata(n as usize, bytes);
    }
}

impl TgScriptEnv {
    fn pushdata(&mut self, n: usize, bytes: Vec<u8>) {
        assert!(bytes.len() == n as usize);
        self.stack.push(bytes)
    }
}

#[cfg(test)]
mod tests {

    use super::*;

    const PUSHDATA_SCRIPT: &'static[u8] = &[0xD1,0x01,0xFF,0xD1,0x02,0x01,0x01];
    const CONDITIONAL_SCRIPT_TRUE: &'static[u8] = &[0x01,0xF1,0x01,0xF2,0x00,0xF3,0xF1,0x00,0xF3];
    const CONDITIONAL_SCRIPT_FALSE: &'static[u8] = &[0x00,0xF1,0x01,0xF2,0x00,0xF3];
    const ERROR_SCRIPT: &'static[u8] = &[0xA1];

//    #[test]
    fn pushdata() {
        let (input, script) = tg_script(&PUSHDATA_SCRIPT).unwrap(); 
        let mut env = TgScriptEnv::from(script);
        env.eval();
    }

    #[test]
    fn conditional_true() {
        let (input, script) = tg_script(&CONDITIONAL_SCRIPT_TRUE).unwrap(); 
        let mut env = TgScriptEnv::from(script);
        env.eval();
    }

 //   #[test]
    fn conditional_false() {
        let (input, script) = tg_script(&CONDITIONAL_SCRIPT_FALSE).unwrap(); 
        let mut env = TgScriptEnv::from(script);
        env.eval();
    }
}
