use std::{
    fmt,
};
use byteorder::{BigEndian, WriteBytesExt};

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
        match self {
           OP_0                                 =>  write!(f, "0"), 
           OP_1                                 =>  write!(f, "1"),
           OP_PUSHDATA1(num_bytes, data)        =>  write!(f, "{}", format!("PUSHDATA1({:?}, {:?})", num_bytes, data)),
           OP_PUSHDATA2(num_bytes, data)        =>  write!(f, "{}", format!("PUSHDATA2({:?}, {:?})", num_bytes, data)),
           OP_PUSHDATA4(num_bytes, data)        =>  write!(f, "{}", format!("PUSHDATA4({:?}, {:?})", num_bytes, data)),
           OP_IF(true_branch, false_branch)     =>  write!(f, "{}", format!("IF({:?}, {:?})", true_branch, false_branch)),
           OP_ELSE(false_branch)                =>  write!(f, "{}", format!("ELSE({:?})", false_branch)),
           OP_ENDIF                             =>  write!(f, "ENDIF"),
           OP_DROP                              =>  write!(f, "DROP"),
           OP_DUP                               =>  write!(f, "DUP"),
           OP_EQUAL                             =>  write!(f, "EQUAL"),
           OP_VERIFYSIG                         =>  write!(f, "VERIFYSIG"),
           OP_SHA256                            =>  write!(f, "SHA256"),
        }
    }
}

impl From<TgOpcode> for Vec<u8> {
    fn from(op: TgOpcode) -> Vec<u8> {
        use TgOpcode::*;
        let mut v = vec![op.bytecode()];
        match op {
            OP_PUSHDATA1(num_bytes, data)           =>  { v.write_u8(num_bytes).unwrap(); v.extend(Vec::from(data.clone())); v } ,
            OP_PUSHDATA2(num_bytes, data)           =>  { v.write_u16::<BigEndian>(num_bytes).unwrap(); v.extend(Vec::from(data)); v } ,
            OP_PUSHDATA4(num_bytes, data)           =>  { v.write_u32::<BigEndian>(num_bytes).unwrap(); v.extend(Vec::from(data)); v } ,
            OP_IF(true_branch, None)                =>  { v.extend(Vec::from(true_branch)); v.push(OP_ENDIF.bytecode()); v },
            OP_IF(true_branch, Some(false_branch))  =>  { v.extend(Vec::from(true_branch)); v.extend(Vec::from(false_branch)); v.push(OP_ENDIF.bytecode()); v },
//NOTE: don't think should ever happen either ... hmmm
            OP_ELSE(false_branch)                   =>  { v.extend(Vec::from(false_branch)); v },
           _ => v,
        }

    }
}

impl TgOpcode {

pub fn bytecode(&self) -> u8 {
    use TgOpcode::*;
    match *self {
        OP_0                =>  0x00,
        OP_1                =>  0x01,
        OP_PUSHDATA1(_,_)   =>  0xD1,
        OP_PUSHDATA2(_,_)   =>  0xD2,
        OP_PUSHDATA4(_,_)   =>  0xD4,
        OP_IF(_,_)          =>  0xF1,
        OP_ELSE(_)          =>  0xF2,
        OP_ENDIF            =>  0xF3,
        OP_DROP             =>  0x50,
        OP_DUP              =>  0x52,
        OP_EQUAL            =>  0xE1,
        OP_VERIFYSIG        =>  0xC1,
        OP_SHA256           =>  0xC2,
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

impl From<TgScript> for Vec<u8> {
    fn from(script: TgScript) -> Vec<u8> {
        let mut v = Vec::<u8>::new();
        for op in script.0 {
            v.extend(Vec::from(op));
        }
        v
    }
}

//#[cfg(test)]
//mod tests {
//    use super::*;
//
//}
