
use crate::{
    script::{
        lib::{
            TgOpcode,
            OpcodeOrData,
            TgScript,
            opcodes::*,
        },
    },
};

struct TgScriptEnv<'a> {
    stack: Vec<&'a [u8]>,
    script: TgScript,
}

trait TgScriptInterpreter {
// etc ???
    fn op_pushdata1();
    fn op_pushdata2();
    fn op_pushdata4();
    fn op_dup();
    fn op_drop();
    fn op_if();
    fn op_else();
    fn op_endif();
}
