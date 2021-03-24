use std::convert::TryInto;
use bdk::bitcoin::{
    util::key::PublicKey,
    secp256k1::{
        Secp256k1,
        Message,
        Signature,
        All,
    },
};
use crate::{
    Result,
    Error,
//    contract::ContractState,
    payout::{
        Payout,
    },
    script::{
        lib::{
//            TgOpcode,
            TgScript,
            TgOpcode::*,
        },
    },
};

const EVAL_DEPTH_LIMIT : u8 = 2;

pub struct TgScriptEnv {
    pub payout: Option<Payout>,
    stack: Vec<Vec<u8>>,
    eval_depth: u8,
    validity: Option<bool>,
    secp: Secp256k1<All>,
}

#[allow(dead_code)]
impl TgScriptEnv {
    pub fn new(payout: Payout) -> Self {
        TgScriptEnv {
            payout: Some(payout),
            stack: Vec::new(),
            eval_depth: 0,
            validity: None,
            secp: Secp256k1::new(),
        }
    }

//todo: probably move this out of the interpreter
    pub fn validate_payout(&mut self) -> Result<()> {

        if self.payout.is_none() {
            return Err(Error::Adhoc("cannot validate payout request - payout request is None"));
        }

// TODO: ensure payout_tx is signed by the same player making the request
// TODO: ensure payout_tx is not already in the blockchain (but then who cares?)

        let payout = self.payout.as_ref().unwrap().clone();
//confirm payout script hash sigs on contract
// TODO: check funding_tx is signed by the correct parties and in the blockchain
//        if payout.contract.state() != ContractState::Live {
//        if payout.contract.state() != ContractState::ArbiterSigned {
//            return Err(Error::Adhoc("invalid payout request - contract is uncertified"))
//        }
//push script sig to stack then evaluate the payout script
        if payout.script_sig.is_some() {
            self.stack.push(payout.script_sig.unwrap().serialize_der().to_vec());
        } else {
            return Err(Error::Adhoc("no script sig"));
        };

        let _result = self.eval(payout.contract.payout_script.clone());

// TODO: this is weird
        match self.validity {
            None | Some(false)  => Err(Error::Adhoc("invalid payout request")),
            Some(true) => Ok(())
        }
    }

    pub fn push_input(&mut self, bytes: Vec<u8>) {
        self.pushdata(bytes.len(), bytes);
    }

    fn pushdata(&mut self, n: usize, bytes: Vec<u8>) {
        assert!(bytes.len() == n as usize);
        self.stack.push(bytes)
    }
}

impl Default for TgScriptEnv {
    fn default() -> Self {
        TgScriptEnv {
            payout: None,
            stack: Vec::new(),
            eval_depth: 0,
            validity: None,
            secp: Secp256k1::new(),
        }
    }
}

// TODO: refactor ops to return results? could be easier to handle invalid scripts
pub trait TgScriptInterpreter {
    fn eval(&mut self, _script: TgScript) -> Result<()> { Err(Error::Adhoc("")) }
// NOTE: opcode functions - in own trait?
    fn op_pushdata1(&mut self, _n: u8, _bytes: Vec<u8>);
    fn op_pushdata2(&mut self, _n: u16, _bytes: Vec<u8>);
    fn op_pushdata4(&mut self, _n: u32, _bytes: Vec<u8>);
    fn op_0(&mut self);
    fn op_1(&mut self);
    fn op_dup(&mut self);
    fn op_2dup(&mut self);
    fn op_drop(&mut self);
    fn op_if(&mut self, _true_branch: TgScript, _false_branch: Option<TgScript>);
    fn op_validate(&mut self);
    fn op_else(&mut self);
    fn op_endif(&mut self);
    fn op_equal(&mut self);
    fn op_nequal(&mut self);
    fn op_verifysig(&mut self);
    fn op_sha256(&mut self);
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
//            println!("preop stack: {:?}", self.stack);
//            println!("preop stack depth: {:?}", self.stack.len());
            match next {
                OP_0                                =>  self.op_0(),
                OP_1                                =>  self.op_1(),
                OP_DUP                              =>  self.op_dup(),
                OP_2DUP                             =>  self.op_2dup(),
                OP_DROP                             =>  self.op_drop(),
                OP_EQUAL                            =>  self.op_equal(),
                OP_VERIFYSIG                        =>  self.op_verifysig(),
                OP_SHA256                           =>  self.op_sha256(),
                OP_PUSHDATA1(n, bytes)              =>  self.op_pushdata1(n.try_into().unwrap(), bytes),
                OP_PUSHDATA2(n, bytes)              =>  self.op_pushdata2(n.try_into().unwrap(), bytes),
                OP_PUSHDATA4(n, bytes)              =>  self.op_pushdata4(n.try_into().unwrap(), bytes),
                OP_IF(true_branch, false_branch)    =>  self.op_if(true_branch, false_branch),
// we shouldn't directly encounter these so I guess they should panic
// OP_ELSE and OP_ENDIF get consumed while parsing OP_IF to avoid keeping track of conditional
// state during evaluation
// e.g. when the next op is OP_ELSE but shouldn't be executed, need to remember conditional state
// instead, an else block is stored as an optional second field for OP_IF to evaluate directly
                OP_ELSE(_) => self.op_else(),
                OP_ENDIF => self.op_endif(),
                OP_VALIDATE => self.op_validate(),
            }
//            println!("postop stack: {:?}", self.stack);
//            println!("postop stack depth: {:?}", self.stack.len());
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

    fn op_2dup(&mut self) {
        let len = self.stack.len();
        self.stack.push(self.stack[len - 2].clone());
// stack.len() increased by one but we want the last element of the stack before
        self.stack.push(self.stack[len - 1].clone());
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

    fn op_validate(&mut self) {
        if self.stack.pop().unwrap() != vec![OP_0.bytecode()] {
            self.validity = Some(true);
        }
        else {
            self.validity = Some(false);
        }
    }

    fn op_equal(&mut self) {
        let len = self.stack.len();
        if self.stack[len - 1] == self.stack[len - 2] {
            self.op_1();
        }
        else {
            self.op_0();
        }
    }

    fn op_nequal(&mut self) {
        let len = self.stack.len();
        if self.stack[len - 1] != self.stack[len - 2] {
            self.op_1();
        }
        else {
            self.op_0();
        }
    }

    fn op_verifysig(&mut self) {

        assert!(self.payout.is_some());

        let payout = self.payout.as_ref().unwrap();
        let payout_txid: &[u8] = &payout.psbt.clone().extract_tx().txid();

        let script_txid = self.stack.pop().unwrap();
        let pubkey: PublicKey = PublicKey::from_slice(&self.stack.pop().unwrap()).unwrap();
        let sig: Signature = Signature::from_der(&self.stack.pop().unwrap()).unwrap();

        let msg: Message = Message::from_slice(&script_txid).unwrap();

        if payout_txid.to_vec() == script_txid && self.secp.verify(&msg, &sig, &pubkey.key).is_ok() {
            self.op_1();
        }
        else {
            self.op_0();
        }
    }

    fn op_sha256(&mut self) {

    }
}

#[cfg(test)]
mod tests {

    use super::*;
//    use crate::script::TgOpcode::*;
    use crate::script::parser::tg_script;
//    use bitcoin::{
//        hashes::{
//            Hash,
//            hex::FromHex,
//        },
//        consensus::{
//            encode,
//        }
//    };

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

    #[test]
    fn round_trip() {

        let script = TgScript(vec![
            OP_1,
            OP_DUP,
            OP_DROP,
        ]);

        let mut env = TgScriptEnv::default();

        env.eval(script.clone()).unwrap(); 

        let bytes = Vec::<u8>::from(script.clone());
        let (_, script2) = tg_script(&bytes).unwrap();
        let mut env = TgScriptEnv::default();
        env.eval(script2.clone()).unwrap(); 
        assert_eq!(script, script2);

        let script = TgScript(vec![
            OP_PUSHDATA1(4,vec![0xff,0xfe,0xdf,0x1f]),
            OP_DUP,
            OP_DROP,
            OP_IF(TgScript(vec![OP_1]), None),
        ]);

        let mut env = TgScriptEnv::default();
        env.eval(script.clone()).unwrap(); 
        let bytes = Vec::<u8>::from(script.clone());
//        println!("{:?}", bytes);
        let (_, script2) = tg_script(&bytes).unwrap();
        let mut env = TgScriptEnv::default();
        env.eval(script2.clone()).unwrap(); 
        assert_eq!(script, script2);
    }

//    #[test]
}
