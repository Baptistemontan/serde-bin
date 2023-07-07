use self::map::ValueMap;
use core::fmt::{self, Debug};

extern crate alloc;

use alloc::{
    boxed::Box,
    string::{String, ToString},
    vec::Vec,
};
use serde::{
    de::{DeserializeSeed, Visitor},
    serde_if_integer128, Deserialize,
};

mod map;

const MAX_PREALLOC_SIZE: usize = 256;

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Number {
    I8(i8),
    I16(i16),
    I32(i32),
    I64(i64),
    U8(u8),
    U16(u16),
    U32(u32),
    U64(u64),
    F32(f32),
    F64(f64),
    #[cfg(not(no_integer128))]
    I128(i128),
    #[cfg(not(no_integer128))]
    U128(u128),
}

#[derive(Debug, Clone, PartialEq)]
pub struct EnumValue<'de> {
    variant: Value<'de>,
    value: Value<'de>,
}

#[derive(Clone, PartialEq, Default)]
pub enum Value<'de> {
    #[default]
    Unit,
    Bool(bool),
    Option(Option<Box<Self>>),
    Number(Number),
    Char(char),
    String(&'de str),
    OwnedString(String),
    Bytes(&'de [u8]),
    OwnedBytes(Vec<u8>),
    Array(Vec<Self>),
    Map(ValueMap<'de>),
    Enum(Box<EnumValue<'de>>),
}

impl<'de> Debug for Value<'de> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Value::Unit => f.write_str("()"),
            Value::Bool(boolean) => write!(f, "Bool({})", boolean),
            Value::Option(option) => {
                f.write_str("Option ")?;
                Debug::fmt(option, f)
            }
            Value::Number(number) => Debug::fmt(number, f),
            Value::Char(c) => write!(f, "'{}'", c),
            Value::String(string) => write!(f, "String({:?})", string),
            Value::OwnedString(string) => write!(f, "OwnedString({:?})", string),
            Value::Bytes(bytes) => write!(f, "Bytes({:?})", bytes),
            Value::OwnedBytes(bytes) => write!(f, "OwnedBytes({:?})", bytes),
            Value::Array(vec) => {
                f.write_str("Array ")?;
                Debug::fmt(vec, f)
            }
            Value::Map(map) => {
                f.write_str("Object ")?;
                Debug::fmt(map, f)
            }
            Value::Enum(e) => Debug::fmt(e, f),
        }
    }
}

impl<'de> Deserialize<'de> for Value<'de> {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        deserializer.deserialize_any(ValueVisitor)
    }
}

struct ValueVisitor;

impl<'de> DeserializeSeed<'de> for ValueVisitor {
    type Value = Value<'de>;

    fn deserialize<D>(self, deserializer: D) -> Result<Self::Value, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        deserializer.deserialize_identifier(self)
    }
}

macro_rules! implement_number {
    ($fn_name:ident, $t:ident, $variant:ident) => {
        fn $fn_name<E>(self, v: $t) -> Result<Self::Value, E>
        where
            E: serde::de::Error,
        {
            Ok(Value::Number(Number::$variant(v)))
        }
    };
}

macro_rules! implement_value {
    ($fn_name:ident, $t:ty, $variant:ident) => {
        fn $fn_name<E>(self, v: $t) -> Result<Self::Value, E>
        where
            E: serde::de::Error,
        {
            Ok(Value::$variant(v))
        }
    };
}

fn size_hint_caution(hint: Option<usize>) -> usize {
    core::cmp::min(hint.unwrap_or(0), MAX_PREALLOC_SIZE)
}

impl<'de> Visitor<'de> for ValueVisitor {
    type Value = Value<'de>;

    fn expecting(&self, f: &mut core::fmt::Formatter) -> core::fmt::Result {
        f.write_str("anything")
    }

    fn visit_bool<E>(self, v: bool) -> Result<Self::Value, E>
    where
        E: serde::de::Error,
    {
        Ok(Value::Bool(v))
    }

    implement_number!(visit_i8, i8, I8);
    implement_number!(visit_i16, i16, I16);
    implement_number!(visit_i32, i32, I32);
    implement_number!(visit_i64, i64, I64);
    implement_number!(visit_u8, u8, U8);
    implement_number!(visit_u16, u16, U16);
    implement_number!(visit_u32, u32, U32);
    implement_number!(visit_u64, u64, U64);
    implement_number!(visit_f32, f32, F32);
    implement_number!(visit_f64, f64, F64);

    serde_if_integer128! {
        implement_number!(visit_i128, i128, I128);
        implement_number!(visit_u128, u128, U128);
    }

    implement_value!(visit_char, char, Char);
    implement_value!(visit_borrowed_str, &'de str, String);
    implement_value!(visit_string, String, OwnedString);
    implement_value!(visit_borrowed_bytes, &'de [u8], Bytes);
    implement_value!(visit_byte_buf, Vec<u8>, OwnedBytes);

    fn visit_str<E>(self, v: &str) -> Result<Self::Value, E>
    where
        E: serde::de::Error,
    {
        self.visit_string(v.to_string())
    }

    fn visit_bytes<E>(self, v: &[u8]) -> Result<Self::Value, E>
    where
        E: serde::de::Error,
    {
        self.visit_byte_buf(Vec::from(v))
    }

    fn visit_none<E>(self) -> Result<Self::Value, E>
    where
        E: serde::de::Error,
    {
        Ok(Value::Option(None))
    }

    fn visit_some<D>(self, deserializer: D) -> Result<Self::Value, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let value = deserializer.deserialize_any(self)?;
        Ok(Value::Option(Some(Box::new(value))))
    }

    fn visit_unit<E>(self) -> Result<Self::Value, E>
    where
        E: serde::de::Error,
    {
        Ok(Value::Unit)
    }

    fn visit_newtype_struct<D>(self, deserializer: D) -> Result<Self::Value, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        deserializer.deserialize_any(self)
    }

    fn visit_seq<A>(self, mut seq: A) -> Result<Self::Value, A::Error>
    where
        A: serde::de::SeqAccess<'de>,
    {
        let mut buff = Vec::with_capacity(size_hint_caution(seq.size_hint()));
        while let Some(v) = seq.next_element()? {
            buff.push(v);
        }
        buff.shrink_to_fit();
        Ok(Value::Array(buff))
    }

    fn visit_map<A>(self, map: A) -> Result<Self::Value, A::Error>
    where
        A: serde::de::MapAccess<'de>,
    {
        let map = ValueMap::from_map_access(map)?;
        Ok(Value::Map(map))
    }

    fn visit_enum<A>(self, _data: A) -> Result<Self::Value, A::Error>
    where
        A: serde::de::EnumAccess<'de>,
    {
        todo!()
    }
}
