use std::fmt;
use serde::{Serialize, Deserialize,};
use byteorder::{BigEndian, WriteBytesExt};


#[allow(non_camel_case_types)]
#[derive(PartialEq, Eq, PartialOrd, Clone, Serialize, Deserialize)]
pub enum TgOpcode {
    OP_0,
    OP_1,
    OP_PUSHDATA1(u8,Vec<u8>),
    OP_PUSHDATA2(u16,Vec<u8>),
    OP_PUSHDATA4(u32,Vec<u8>),
    OP_IF(TgScript, Option<TgScript>),
    OP_ELSE(TgScript),
    OP_ENDIF,
    OP_DROP,
    OP_DUP,
    OP_2DUP,
    OP_EQUAL,
    OP_VERIFYSIG,
    OP_SHA256,
    OP_VALIDATE,
    OP_PUSHTXID,
}

impl fmt::Debug for TgOpcode {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.write_str("OP_")?;
        use TgOpcode::*;
        match self {
           OP_0                                 =>  write!(f, "0"), 
           OP_1                                 =>  write!(f, "1"),
           OP_PUSHDATA1(num_bytes, _data)       =>  write!(f, "PUSHDATA1({})", format!("{:?}, [..]", num_bytes)),//"{:?}", num_bytes, data)),
           OP_PUSHDATA2(num_bytes, _data)       =>  write!(f, "PUSHDATA2({})", format!("{:?}, [..]", num_bytes)),//"{:?}", num_bytes, data)),
           OP_PUSHDATA4(num_bytes, _data)       =>  write!(f, "PUSHDATA4({})", format!("{:?}, [..]", num_bytes)),//"{:?}", num_bytes, data)),
           OP_IF(true_branch, false_branch)     =>  write!(f, "IF({})", format!("{:?}, {:?}", true_branch, false_branch)),
           OP_ELSE(false_branch)                =>  write!(f, "ELSE({})", format!("{:?}", false_branch)),
           OP_VALIDATE                          =>  write!(f, "VALIDATE"),
           OP_ENDIF                             =>  write!(f, "ENDIF"),
           OP_DROP                              =>  write!(f, "DROP"),
           OP_DUP                               =>  write!(f, "DUP"),
           OP_2DUP                              =>  write!(f, "2DUP"),
           OP_EQUAL                             =>  write!(f, "EQUAL"),
           OP_VERIFYSIG                         =>  write!(f, "VERIFYSIG"),
           OP_SHA256                            =>  write!(f, "SHA256"),
           OP_PUSHTXID                          =>  write!(f, "PUSHTXID"),
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
            OP_IF(true_branch, Some(false_branch))  =>  { v.extend(Vec::from(true_branch)); v.push(OP_ELSE(TgScript::default()).bytecode()); v.extend(Vec::from(false_branch)); v.push(OP_ENDIF.bytecode()); v },
//NOTE: don't think this should ever happen either ... hmmm
            OP_ELSE(_false_branch)                  =>  { panic!("encountered OP_ELSE outside of OP_IF"); },
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
            OP_PUSHTXID         =>  0xD0,
            OP_IF(_,_)          =>  0xF1,
            OP_ELSE(_)          =>  0xF2,
            OP_ENDIF            =>  0xF3,
            OP_VALIDATE         =>  0xF4,
            OP_DROP             =>  0x50,
            OP_DUP              =>  0x52,
            OP_2DUP             =>  0x53,
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

#[derive(Clone, PartialOrd, PartialEq, Eq, Debug, Serialize, Deserialize)]
pub struct TgScript(pub Vec<TgOpcode>);

impl Default for TgScript {
    fn default() -> Self {
        TgScript(Vec::new())
    }
}

impl From<TgScript> for Vec<u8> {
    fn from(script: TgScript) -> Vec<u8> {
// TODO how to one-liner this ?
        let mut v = Vec::<u8>::new();
        for op in script.0 {
            v.extend(Vec::from(op));
        }
        v
    }
}
