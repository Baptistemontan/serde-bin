use self::map::ValueMap;
use core::fmt::{self, Debug};

extern crate alloc;

use alloc::{boxed::Box, string::String, vec::Vec};

mod map;

#[derive(Debug, Clone, Copy, Eq, PartialEq, Hash)]
pub enum Number {
    I8(i8),
    I16(i16),
    I32(i32),
    I64(i64),
    U8(u8),
    U16(u16),
    U32(u32),
    U64(u64),
}

#[derive(Clone, Eq, PartialEq, Hash, Default)]
pub enum Value {
    #[default]
    Unit,
    Bool(bool),
    Option(Option<Box<Self>>),
    Number(Number),
    String(String),
    Array(Vec<Self>),
    Map(ValueMap),
}

impl Debug for Value {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Value::Unit => f.write_str("()"),
            Value::Bool(boolean) => write!(f, "Bool({})", boolean),
            Value::Number(number) => Debug::fmt(number, f),
            Value::String(string) => write!(f, "String({:?})", string),
            Value::Array(vec) => {
                f.write_str("Array ")?;
                Debug::fmt(vec, f)
            }
            Value::Map(map) => {
                f.write_str("Object ")?;
                Debug::fmt(map, f)
            }
            Value::Option(option) => {
                f.write_str("Option ")?;
                Debug::fmt(option, f)
            }
        }
    }
}
