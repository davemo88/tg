use std::{
    fmt,
};

#[derive(PartialEq, Eq, PartialOrd, Clone)]
pub enum TgOpcode {
    OP_0,
    OP_1,
    OP_PUSHDATA1(u8,Vec<u8>),
    OP_PUSHDATA2(u16,Vec<u8>),
    OP_PUSHDATA4(u32,Vec<u8>),
    OP_IF(TgScript, Option<TgScript>),
//    OP_IF(TgScript, Box<Option<TgOpcode>>),
    OP_ELSE(TgScript),
    OP_ENDIF,
    OP_DROP,
    OP_DUP,
    OP_EQUAL,
    OP_VERIFYSIG,
    OP_SHA256,
}

impl fmt::Debug for TgOpcode {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.write_str("OP_")?;
        use TgOpcode::*;
        match *self {
           OP_0 => write!(f, "0"), 
           OP_1 => write!(f, "1"),
           OP_PUSHDATA1(_,_) => write!(f, "PUSHDATA1"),
           OP_PUSHDATA2(_,_) => write!(f, "PUSHDATA2"),
           OP_PUSHDATA4(_,_) => write!(f, "PUSHDATA4"),
           OP_IF(_,_) => write!(f, "IF"),
           OP_ELSE(_) => write!(f, "ELSE"),
           OP_ENDIF => write!(f, "ENDIF"),
           OP_DROP => write!(f, "DROP"),
           OP_DUP => write!(f, "DUP"),
           OP_EQUAL => write!(f, "EQUAL"),
           OP_VERIFYSIG => write!(f, "VERIFYSIG"),
           OP_SHA256 => write!(f, "SHA256"),
        }
    }
}

impl TgOpcode {

pub fn bytecode(&self) -> u8 {
    use TgOpcode::*;
    match *self {
        OP_0                =>      0x00,
        OP_1                =>      0x01,
        OP_PUSHDATA1(_,_)   =>      0xD1,
        OP_PUSHDATA2(_,_)   =>      0xD2,
        OP_PUSHDATA4(_,_)   =>      0xD4,
        OP_IF(_,_)          =>      0xF1,
        OP_ELSE(_)          =>      0xF2,
        OP_ENDIF            =>      0xF3,
        OP_DROP             =>      0x50,
        OP_DUP              =>      0x52,
        OP_EQUAL            =>      0xE1,
        OP_VERIFYSIG        =>      0xC1,
        OP_SHA256           =>      0xC2,
    }
}


}

impl Into<u8> for TgOpcode {
    fn into(self) -> u8 {
        self.bytecode()
    }
}

#[derive(Clone, PartialOrd, PartialEq, Eq, Debug,)]
pub struct TgScript(pub Vec<TgOpcode>);

impl Default for TgScript {
    fn default() -> Self {
        TgScript(Vec::new())
    }
}

//#[cfg(test)]
//mod tests {
