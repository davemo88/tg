use std::convert::TryInto;
use crate::{
    script::{
        lib::{
//            TgOpcode,
            TgScript,
            TgOpcode::*,
        },
    },
    Result,
    TgError,
};

struct TgScriptEnv {
//    payout_request: Option<PayoutRequest>,
    stack: Vec<Vec<u8>>,
    eval_depth: u8,
}

const EVAL_DEPTH_LIMIT : u8 = 2;

impl Default for TgScriptEnv {
    fn default() -> Self {
        TgScriptEnv {
            stack: Vec::new(),
            eval_depth: 0,
        }
    }
}

trait TgScriptInterpreter {
    fn eval(&mut self, _script: TgScript) -> Result<()> { Err(TgError("")) }
// NOTE: opcode functions - in own trait?
    fn op_pushdata1(&mut self, _n:u8, _bytes: Vec<u8>) {}
    fn op_pushdata2(&mut self, _n:u16, _bytes: Vec<u8>) {}
    fn op_pushdata4(&mut self, _n:u32, _bytes: Vec<u8>) {}
    fn op_0(&mut self) {}
    fn op_1(&mut self) {}
    fn op_dup(&mut self) {}
    fn op_drop(&mut self) {}
    fn op_if(&mut self, _true_branch: TgScript, _false_branch: Option<TgScript>) {}
    fn op_else(&mut self) {}
    fn op_endif(&mut self) {}
    fn op_equal(&mut self) {}
    fn op_nequal(&mut self) {}
    fn op_verifysig(&mut self) {}
    fn op_sha256(&mut self) {}
}

impl TgScriptInterpreter for TgScriptEnv {
    fn eval(&mut self, mut script: TgScript) -> Result<()> {

        if self.eval_depth == EVAL_DEPTH_LIMIT {
            panic!("eval depth limit reached");
        }
        else
        {
            println!("eval: {:?}", script);
            self.eval_depth += 1;
        }

        while script.0.len() > 0 {
            let next = script.0.remove(0);    
            println!("next op: {:?}", next);
            println!("preop stack: {:?}", self.stack);
            match next {
                OP_0 => self.op_0(),
                OP_1 => self.op_1(),
                OP_DUP => self.op_dup(),
                OP_DROP => self.op_drop(),
                OP_EQUAL => self.op_equal(),
                OP_VERIFYSIG => self.op_verifysig(),
                OP_SHA256 => self.op_sha256(),
                OP_PUSHDATA1(n, bytes) => self.op_pushdata1(n.try_into().unwrap(), bytes),
                OP_PUSHDATA2(n, bytes) => self.op_pushdata2(n.try_into().unwrap(), bytes),
                OP_PUSHDATA4(n, bytes) => self.op_pushdata4(n.try_into().unwrap(), bytes),
                OP_IF(true_branch, false_branch) => self.op_if(true_branch, false_branch),
// we shouldn't directly encounter these so I guess they should cause panic
// OP_ELSE and OP_ENDIF get consumed while parsing OP_IF to avoid keeping track of conditional
// state during evaluation
// e.g. when the next op is OP_ELSE but shouldn't be executed, need to remember conditional state
// instead, an else block is stored as an optional second field for OP_IF to evaluate directly
                OP_ELSE(_) => self.op_else(),
                OP_ENDIF => self.op_endif(),
            }
            println!("postop stack: {:?}", self.stack);
        }
        self.eval_depth -= 1;

        println!("eval returning");

        Ok(())
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

    fn op_0(&mut self) {
        self.stack.push(vec![OP_0.bytecode()])
    }

    fn op_1(&mut self) {
        self.stack.push(vec![OP_1.bytecode()])
    }

    fn op_dup(&mut self) {
        self.stack.push(self.stack.last().unwrap().clone());
    }

    fn op_drop(&mut self) {
        self.stack.pop().unwrap();
    }

    fn op_if(&mut self, true_branch: TgScript, false_branch: Option<TgScript>) {
        if self.stack.pop().unwrap() != vec![OP_0.bytecode()] {
            self.eval(true_branch).unwrap();
        }
        else if let Some(false_branch) = false_branch {
            self.eval(false_branch).unwrap();
        }
    }

    fn op_else(&mut self) {
        panic!("unexpected OP_ELSE")
    }

    fn op_endif(&mut self) {
        panic!("unexpected OP_ENDIF")
    }

    fn op_equal(&mut self) {
        if self.stack.pop().unwrap() == self.stack.pop().unwrap() {
        }
        else {
            self.op_0();
        }
    }

    fn op_nequal(&mut self) {
        if self.stack.pop().unwrap() == self.stack.pop().unwrap() {
            self.op_0();
        }
        else {
            self.op_1();
        }
    }

    fn op_verifysig(&mut self) {

    }

    fn op_sha256(&mut self) {

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
    use crate::script::parser::tg_script;

    const PUSHDATA_SCRIPT: &'static[u8] = &[0xD1,0x01,0xFF,0xD1,0x02,0x01,0x01];
    const CONDITIONAL_SCRIPT_TRUE: &'static[u8] = &[0x01,0xF1,0x01,0xF2,0x00,0xF3,0xF1,0x00,0xF3];
    const CONDITIONAL_SCRIPT_FALSE: &'static[u8] = &[0x00,0xF1,0x01,0xF2,0x00,0xF3,0xF1,0x00,0xF3];

    #[test]
    fn pushdata() {
        let (_, script) = tg_script(&PUSHDATA_SCRIPT).unwrap(); 
        let mut env = TgScriptEnv::default();
        env.eval(script).unwrap();
    }

    #[test]
    fn conditional_true() {
        let (_, script) = tg_script(&CONDITIONAL_SCRIPT_TRUE).unwrap(); 
        let mut env = TgScriptEnv::default();
        env.eval(script).unwrap();
    }

    #[test]
    fn conditional_false() {
        let (_, script) = tg_script(&CONDITIONAL_SCRIPT_FALSE).unwrap(); 
        let mut env = TgScriptEnv::default();
        env.eval(script).unwrap();
    }
}
