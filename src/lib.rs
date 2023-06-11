pub mod any;
mod de;
mod error;
mod ser;
mod write;

pub use de::{from_bytes, Deserializer};
pub use error::{Error, NoWriterError, Result, WriterError};
#[cfg(feature = "alloc")]
pub use ser::to_bytes;
#[cfg(feature = "std")]
pub use ser::to_writer;
pub use ser::{get_serialized_size, to_buff, Serializer};
pub use write::{BuffWriter, EndOfBuff, Write};

const UNSIZED_STRING_END_MARKER: [u8; 2] = [0xD8, 0x00];

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

    #[test]
    fn test_invalid_utf8_char() {
        let [byte1, byte2] = UNSIZED_STRING_END_MARKER;
        assert_eq!(
            char::from_u32(u32::from_be_bytes([0, 0, byte1, byte2])),
            None
        );
        assert!(core::str::from_utf8(&UNSIZED_STRING_END_MARKER).is_err());
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

        let n_bytes = u64::to_be_bytes(N as u64);
        let len = u64::to_be_bytes(STRING.len() as u64);
        let str_bytes = STRING.as_bytes();

        let check: Vec<u8> = n_bytes
            .into_iter()
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

        assert_eq!(v, &[0, 0, 0, 0])
    }

    #[test]
    fn test_serialize_enum_newtype() {
        let value = TestEnum::NewType(56);

        let mut v: Vec<u8> = Vec::new();
        ser::to_writer(&value, &mut v).unwrap();

        assert_eq!(v, &[0, 0, 0, 1, 56])
    }

    #[test]
    fn test_serialize_enum_tuple() {
        const NUM: f32 = 12.3;
        const STRING: &str = "String";
        let value = TestEnum::Tuple(NUM, STRING.to_string());

        let mut v: Vec<u8> = Vec::new();
        ser::to_writer(&value, &mut v).unwrap();

        let variant_index_bytes = 2u32.to_be_bytes();
        let fbytes = NUM.to_be_bytes();
        let len_bytes = (STRING.len() as u64).to_be_bytes();
        let str_bytes = STRING.as_bytes();
        let vt = variant_index_bytes
            .into_iter()
            .chain(fbytes)
            .chain(len_bytes)
            .chain(str_bytes.iter().copied())
            .collect::<Vec<u8>>();

        assert_eq!(v, vt)
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

        let variant_index_bytes = 3u32.to_be_bytes();
        let fbytes = NUM.to_be_bytes();
        let len_bytes = (VEC.len() as u64).to_be_bytes();
        let vec_bytes = VEC.iter().copied().flat_map(u16::to_be_bytes);
        let vt = variant_index_bytes
            .into_iter()
            .chain(fbytes)
            .chain(len_bytes)
            .chain(vec_bytes)
            .collect::<Vec<u8>>();

        assert_eq!(v, vt)
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
}
