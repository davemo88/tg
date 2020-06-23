
use std::{
    str::FromStr,
    fmt,
};
use hex::{
    self,
};

use crate::lib::{
    TgError,
};

#[derive(PartialEq, Eq, PartialOrd, Clone)]
pub struct TgOpcode(pub u8);

impl TgOpcode {
    pub fn is_valid(&self) -> bool {
        use opcodes::*;
        match  *self {
           OP_0 => true, 
           OP_1 => true,
           OP_PUSHDATA1 => true,
           OP_PUSHDATA2 => true,
           OP_PUSHDATA4 => true,
           OP_IF => true,
           OP_ELSE => true,
           OP_ENDIF => true,
           OP_DROP => true,
           OP_DUP => true,
           OP_EQUAL => true,
           OP_NEQUAL => true,
           OP_VERIFYSIG => true,
           OP_SHA256 => true,
           _ => false,
        }
    }
}

impl fmt::Debug for TgOpcode {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.write_str("OP_")?;
        use opcodes::*;
        match *self {
           OP_0 => write!(f, "0"), 
           OP_1 => write!(f, "1"),
           OP_PUSHDATA1 => write!(f, "PUSHDATA1"),
           OP_PUSHDATA2 => write!(f, "PUSHDATA2"),
           OP_PUSHDATA4 => write!(f, "PUSHDATA4"),
           OP_IF => write!(f, "IF"),
           OP_ELSE => write!(f, "ELSE"),
           OP_ENDIF => write!(f, "ENDIF"),
           OP_DROP => write!(f, "DROP"),
           OP_DUP => write!(f, "DUP"),
           OP_EQUAL => write!(f, "EQUAL"),
           OP_NEQUAL => write!(f, "NEQUAL"),
           OP_VERIFYSIG => write!(f, "VERIFYSIG"),
           OP_SHA256 => write!(f, "SHA256"),
           _ => write!(f, "INVALID"),
        }
    }
}

pub mod opcodes {

    use super::TgOpcode;

// constants
    pub const OP_0: TgOpcode = TgOpcode(0x00);
    pub const OP_1: TgOpcode = TgOpcode(0x01);

// adding raw bytes
    pub const OP_PUSHDATA1: TgOpcode = TgOpcode(0xD1);
    pub const OP_PUSHDATA2: TgOpcode = TgOpcode(0xD2);
    pub const OP_PUSHDATA4: TgOpcode = TgOpcode(0xD4);

// branching 
    pub const OP_IF: TgOpcode = TgOpcode(0xF1);
    pub const OP_ELSE: TgOpcode = TgOpcode(0xF2);
    pub const OP_ENDIF: TgOpcode = TgOpcode(0xF3);

// marks payout request as invalid if the top stack value is not true
    pub const OP_VALIDATE: TgOpcode = TgOpcode(0xFF);

// stack manipulation
    pub const OP_DROP: TgOpcode = TgOpcode(0x50);
    pub const OP_DUP: TgOpcode = TgOpcode(0x52);

// comparison
    pub const OP_EQUAL: TgOpcode = TgOpcode(0xE1);
    pub const OP_NEQUAL: TgOpcode = TgOpcode(0xE2);

// crypto
    pub const OP_VERIFYSIG: TgOpcode = TgOpcode(0xC1);
    pub const OP_SHA256: TgOpcode = TgOpcode(0xC2);

}

#[cfg(test)]
mod tests {

    use super::*;

}
