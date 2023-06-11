use core::fmt::Display;

use crate::Error;

mod de;
mod ser;

#[cfg(feature = "alloc")]
pub mod value;

pub use de::{from_bytes, Deserializer};
#[cfg(feature = "alloc")]
pub use ser::to_bytes;
#[cfg(feature = "std")]
pub use ser::to_writer;
pub use ser::{get_serialized_size, to_buff, Serializer};

#[derive(Debug, PartialEq, Eq, Hash, Clone, Copy)]
#[repr(u8)]
pub enum Tag {
    None = 0,
    Some = 1,
    BoolFalse = 2,
    BoolTrue = 3,
    I8 = 4,
    I16 = 5,
    I32 = 6,
    I64 = 7,
    U8 = 8,
    U16 = 9,
    U32 = 10,
    U64 = 11,
    F32 = 12,
    F64 = 13,
    Char1 = 14,
    Char2 = 15,
    Char3 = 16,
    Char4 = 17,
    String = 18,
    ByteArray = 19,
    Unit = 20,
    UnitStruct = 21,
    UnitVariant = 22,
    NewTypeStruct = 23,
    NewTypeVariant = 24,
    Seq = 25,
    Tuple = 26,
    TupleStruct = 27,
    TupleVariant = 28,
    Map = 29,
    Struct = 30,
    StructVariant = 31,
    NullTerminatedString = 32,
    I128 = 33,
    U128 = 34,
}

impl Tag {
    pub fn encode_char(c: char, buff: &mut [u8]) -> (Self, &[u8]) {
        let bytes = c.encode_utf8(buff).as_bytes();
        let tag = match bytes.len() {
            1 => Tag::Char1,
            2 => Tag::Char2,
            3 => Tag::Char3,
            4 => Tag::Char4,
            _ => unreachable!(),
        };
        (tag, bytes)
    }
}

#[derive(Debug, PartialEq, Eq, Hash, Clone, Copy)]
pub enum TagParsingError {
    #[cfg(no_integer128)]
    Integer128,
    InvalidTag(u8),
    UnexpectedTag {
        expected: &'static str,
        got: Tag,
    },
}

impl TagParsingError {
    pub fn unexpected(expected: &'static str, got: Tag) -> Self {
        Self::UnexpectedTag { expected, got }
    }
}

impl Display for TagParsingError {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        match self {
            #[cfg(no_integer128)]
            TagParsingError::Integer128 => {
                f.write_str("This platform doesn't support 128 bits integers.")
            }
            TagParsingError::InvalidTag(tag) => f.write_fmt(format_args!(
                "Invalid tag for data type: expected byte beetween 0 and 31 included, got {}",
                tag
            )),
            TagParsingError::UnexpectedTag { expected, got } => {
                f.write_fmt(format_args!("Expected {} but got {:?}", expected, got))
            }
        }
    }
}

impl TryFrom<u8> for Tag {
    type Error = TagParsingError;

    fn try_from(value: u8) -> core::result::Result<Self, Self::Error> {
        match value {
            0 => Ok(Tag::None),
            1 => Ok(Tag::Some),
            2 => Ok(Tag::BoolFalse),
            3 => Ok(Tag::BoolTrue),
            4 => Ok(Tag::I8),
            5 => Ok(Tag::I16),
            6 => Ok(Tag::I32),
            7 => Ok(Tag::I64),
            8 => Ok(Tag::U8),
            9 => Ok(Tag::U16),
            10 => Ok(Tag::U32),
            11 => Ok(Tag::U64),
            12 => Ok(Tag::F32),
            13 => Ok(Tag::F64),
            14 => Ok(Tag::Char1),
            15 => Ok(Tag::Char2),
            16 => Ok(Tag::Char3),
            17 => Ok(Tag::Char4),
            18 => Ok(Tag::String),
            19 => Ok(Tag::ByteArray),
            20 => Ok(Tag::Unit),
            21 => Ok(Tag::UnitStruct),
            22 => Ok(Tag::UnitVariant),
            23 => Ok(Tag::NewTypeStruct),
            24 => Ok(Tag::NewTypeVariant),
            25 => Ok(Tag::Seq),
            26 => Ok(Tag::Tuple),
            27 => Ok(Tag::TupleStruct),
            28 => Ok(Tag::TupleVariant),
            29 => Ok(Tag::Map),
            30 => Ok(Tag::Struct),
            31 => Ok(Tag::StructVariant),
            32 => Ok(Tag::NullTerminatedString),
            #[cfg(not(no_integer128))]
            33 => Ok(Tag::I128),
            #[cfg(not(no_integer128))]
            34 => Ok(Tag::U128),
            #[cfg(no_integer128)]
            33 | 34 => Err(TagParsingError::Integer128),
            tag => Err(TagParsingError::InvalidTag(tag)),
        }
    }
}

impl From<Tag> for u8 {
    fn from(value: Tag) -> Self {
        value as u8
    }
}

impl<We> From<TagParsingError> for Error<We> {
    fn from(value: TagParsingError) -> Self {
        Error::TagParsingError(value)
    }
}

#[cfg(all(test, feature = "test-utils"))]
mod tests {

    use super::*;
    use serde::{Deserialize, Serialize};

    #[derive(Debug, Serialize, Deserialize, PartialEq, Eq)]
    struct TestStruct {
        a: usize,
        b: String,
    }

    #[derive(Debug, Serialize, Deserialize, PartialEq)]
    enum TestEnum {
        Unit,
        NewType(u8),
        Tuple(f32, String),
        Struct { a: f64, b: Vec<u16> },
    }

    #[derive(Debug, Serialize, Deserialize, PartialEq)]
    #[serde(untagged)]
    enum UntaggedEnum {
        NewType(String),
        Struct { num: usize },
    }

    #[test]
    fn test_serialize_struct() {
        const N: usize = 56;
        const STRING: &str = "Hello";

        let value = TestStruct {
            a: N,
            b: STRING.to_string(),
        };

        let mut v: Vec<u8> = Vec::new();
        ser::to_writer(&value, &mut v).unwrap();
        let struct_tag: u8 = Tag::Struct.into();
        let struct_size: u8 = 2;
        let num_tag: u8 = Tag::U64.into();
        let n_bytes = u64::to_be_bytes(N as u64);
        let string_tag: u8 = Tag::String.into();
        let len = u64::to_be_bytes(STRING.len() as u64);
        let str_bytes = STRING.as_bytes();

        let check: Vec<u8> = [struct_tag, struct_size, num_tag]
            .into_iter()
            .chain(n_bytes)
            .chain(Some(string_tag))
            .chain(len)
            .chain(str_bytes.iter().copied())
            .collect();

        assert_eq!(v, check);
    }

    #[test]
    fn test_serialize_deserialize_struct() {
        let value = TestStruct {
            a: 56,
            b: "Hello".to_string(),
        };

        let mut v: Vec<u8> = Vec::new();
        ser::to_writer(&value, &mut v).unwrap();

        let t: TestStruct = de::from_bytes(&v).unwrap();

        assert_eq!(t, value);
    }

    #[test]
    fn test_serialize_enum_unit() {
        let value = TestEnum::Unit;

        let mut v: Vec<u8> = Vec::new();
        ser::to_writer(&value, &mut v).unwrap();

        assert_eq!(v, &[Tag::UnitVariant.into(), 0, 0, 0, 0])
    }

    #[test]
    fn test_serialize_enum_newtype() {
        let value = TestEnum::NewType(56);

        let mut v: Vec<u8> = Vec::new();
        ser::to_writer(&value, &mut v).unwrap();

        assert_eq!(
            v,
            // variant tag              | variant idx |  u8 tag     | u8
            &[Tag::NewTypeVariant.into(), 0, 0, 0, 1, Tag::U8.into(), 56]
        )
    }

    #[test]
    fn test_serialize_enum_tuple() {
        const NUM: f32 = 12.3;
        const STRING: &str = "String";
        let value = TestEnum::Tuple(NUM, STRING.to_string());

        let mut v: Vec<u8> = Vec::new();
        ser::to_writer(&value, &mut v).unwrap();

        let variant_tag: u8 = Tag::TupleVariant.into();
        let variant_index_bytes = 2u32.to_be_bytes();
        let f32_tag: u8 = Tag::F32.into();
        let fbytes = NUM.to_be_bytes();
        let string_tag: u8 = Tag::String.into();
        let len_bytes = (STRING.len() as u64).to_be_bytes();
        let str_bytes = STRING.as_bytes();
        let vt = [variant_tag]
            .into_iter()
            .chain(variant_index_bytes)
            .chain([f32_tag])
            .chain(fbytes)
            .chain([string_tag])
            .chain(len_bytes)
            .chain(str_bytes.iter().copied())
            .collect::<Vec<u8>>();

        assert_eq!(v, vt);

        // serialized
        //  [
        //      28,                           variant tag
        //      0, 0, 0, 2,                   variant index
        //      12,                           F32 tag
        //      65, 68, 204, 205,             NUM
        //      18,                           String tag
        //      0, 0, 0, 0, 0, 0, 0, 6,       string len
        //      83, 116, 114, 105, 110, 103   string bytes
        //  ]
    }

    #[test]
    fn test_serialize_enum_struct() {
        const NUM: f64 = 42.123;
        const VEC: [u16; 4] = [3, 7, 1, 8];
        let value = TestEnum::Struct {
            a: NUM,
            b: VEC.to_vec(),
        };

        let mut v: Vec<u8> = Vec::new();
        ser::to_writer(&value, &mut v).unwrap();

        let variant_tag: u8 = Tag::StructVariant.into();
        let variant_index_bytes = 3u32.to_be_bytes();
        let num_tag: u8 = Tag::F64.into();
        let fbytes = NUM.to_be_bytes();
        let seq_tag: u8 = Tag::Seq.into();
        let len_bytes = (VEC.len() as u64).to_be_bytes();
        let vec_bytes = VEC
            .iter()
            .copied()
            .map(u16::to_be_bytes)
            .flat_map(|[a, b]| [Tag::U16.into(), a, b]);
        let vt = [variant_tag]
            .into_iter()
            .chain(variant_index_bytes)
            .chain([num_tag])
            .chain(fbytes)
            .chain([seq_tag])
            .chain(len_bytes)
            .chain(vec_bytes)
            .collect::<Vec<u8>>();

        assert_eq!(v, vt);

        //  [
        //      31,                                   variant tag
        //      0, 0, 0, 3,                           variant index
        //      13,                                   F64 tag
        //      64, 69, 15, 190, 118, 200, 180, 57,   f64
        //      25,                                   Seq tag
        //      0, 0, 0, 0, 0, 0, 0, 4,               Seq len
        //      9, 0, 3,                              u16 tag + u16
        //      9, 0, 7,                              same
        //      9, 0, 1,                              same
        //      9, 0, 8                               same
        //  ]
    }

    #[test]
    fn test_serialize_deserialize_enum_unit() {
        let value = TestEnum::Unit;

        let mut v: Vec<u8> = Vec::new();
        ser::to_writer(&value, &mut v).unwrap();

        let res: TestEnum = de::from_bytes(&v).unwrap();

        assert_eq!(value, res);
    }

    #[test]
    fn test_serialize_deserialize_enum_newtype() {
        let value = TestEnum::NewType(56);

        let mut v: Vec<u8> = Vec::new();
        ser::to_writer(&value, &mut v).unwrap();

        let res: TestEnum = de::from_bytes(&v).unwrap();

        assert_eq!(value, res);
    }

    #[test]
    fn test_serialize_deserialize_enum_tuple() {
        const NUM: f32 = 12.3;
        const STRING: &str = "String";
        let value = TestEnum::Tuple(NUM, STRING.to_string());

        let mut v: Vec<u8> = Vec::new();
        ser::to_writer(&value, &mut v).unwrap();

        let res: TestEnum = de::from_bytes(&v).unwrap();

        assert_eq!(value, res);
    }

    #[test]
    fn test_serialize_deserialize_enum_struct() {
        const NUM: f64 = 42.123;
        const VEC: [u16; 4] = [3, 7, 1, 8];
        let value = TestEnum::Struct {
            a: NUM,
            b: VEC.to_vec(),
        };

        let mut v: Vec<u8> = Vec::new();
        ser::to_writer(&value, &mut v).unwrap();

        let res: TestEnum = de::from_bytes(&v).unwrap();

        assert_eq!(value, res);
    }

    #[test]
    fn test_serialize_deserialize_char1() {
        let c = 'Y';

        let mut v: Vec<u8> = Vec::new();
        ser::to_writer(&c, &mut v).unwrap();

        let res: char = de::from_bytes(&v).unwrap();

        assert_eq!(c, res);
    }

    #[test]
    fn test_serialize_deserialize_char2() {
        let c = 'î'; // 0xC3AE

        let mut v: Vec<u8> = Vec::new();
        ser::to_writer(&c, &mut v).unwrap();

        let res: char = de::from_bytes(&v).unwrap();

        assert_eq!(c, res);
    }

    #[test]
    fn test_serialize_deserialize_char3() {
        let c = 'ࠎ'; // 0xE0A08E

        let mut v: Vec<u8> = Vec::new();
        ser::to_writer(&c, &mut v).unwrap();

        let res: char = de::from_bytes(&v).unwrap();

        assert_eq!(c, res);
    }

    #[test]
    fn test_serialize_deserialize_char4() {
        let c = '𒀀'; // 0xF0928080

        let mut v: Vec<u8> = Vec::new();
        ser::to_writer(&c, &mut v).unwrap();

        let res: char = de::from_bytes(&v).unwrap();

        assert_eq!(c, res);
    }

    #[test]
    fn test_serialize_deserialize_untagged_enum_variant1() {
        let value = UntaggedEnum::NewType("t".into());

        let mut v: Vec<u8> = Vec::new();
        ser::to_writer(&value, &mut v).unwrap();

        let res: UntaggedEnum = de::from_bytes(&v).unwrap();

        assert_eq!(value, res);
    }

    #[test]
    fn test_serialize_deserialize_untagged_enum_variant2() {
        let value = UntaggedEnum::Struct { num: 12 };

        let mut v: Vec<u8> = Vec::new();
        ser::to_writer(&value, &mut v).unwrap();

        let res: UntaggedEnum = de::from_bytes(&v).unwrap();

        assert_eq!(value, res);
    }
}
