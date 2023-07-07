use serde::{
    de::{self, EnumAccess, IntoDeserializer, MapAccess, SeqAccess, VariantAccess, Visitor},
    serde_if_integer128, Deserialize,
};

use crate::{
    error::{Error as Err, NoWriterError, Result},
    UNSIZED_STRING_END_MARKER,
};

use super::{Tag, TagParsingError};

type Error = Err<NoWriterError>;

macro_rules! match_tag {
    ($tag:expr, $expected:expr, $($tagpat:pat => $x:expr)*) => {
        match $tag {
            $(
                $tagpat => $x,
            )*
            got => unexpected_tag!($expected, got)
        }
    }
}

macro_rules! unexpected_tag {
    ($expected:expr, $got:expr) => {
        return Err(TagParsingError::unexpected($expected, $got).into())
    };
}

macro_rules! check_tag {
    ($tag:pat, $input_tag:expr, $expected:expr) => {{
        match $input_tag {
            popped_tag @ $tag => popped_tag,
            got => return Err(TagParsingError::unexpected($expected, got).into()),
        }
    }};
}

macro_rules! implement_number {
    ($fn_name:ident, $visitor_fn_name:ident, $t:ident, $expected_tag:pat, $expected:expr) => {
        fn $fn_name<V>(self, visitor: V) -> Result<V::Value>
        where
            V: Visitor<'de>,
        {
            check_tag!($expected_tag, self.pop_tag()?, $expected);
            let bytes = self.pop_n()?;
            visitor.$visitor_fn_name($t::from_be_bytes(bytes))
        }
    };
}

pub struct Deserializer<'de> {
    input: &'de [u8],
}

pub fn from_bytes<'a, T>(input: &'a [u8]) -> Result<T>
where
    T: Deserialize<'a>,
{
    let mut deserializer = Deserializer { input };
    let t = T::deserialize(&mut deserializer)?;
    let len = deserializer.input.len();
    (len == 0).then_some(t).ok_or(Error::TrailingBytes(len))
}

impl<'de> Deserializer<'de> {
    fn pop_tag(&mut self) -> Result<Tag> {
        let [byte] = self.pop_n()?;
        let tag = byte.try_into()?;
        Ok(tag)
    }

    fn peek_tag(&mut self) -> Result<Tag> {
        let byte = self.input.first().copied().ok_or(Error::Eof)?;
        let tag = byte.try_into()?;
        Ok(tag)
    }

    fn pop_slice(&mut self, len: usize) -> Result<&'de [u8]> {
        if self.input.len() < len {
            return Err(Error::Eof);
        }
        let (bytes, rem) = self.input.split_at(len);
        self.input = rem;
        Ok(bytes)
    }

    fn pop_n<const N: usize>(&mut self) -> Result<[u8; N]> {
        let bytes = self.pop_slice(N)?;
        let mut buff = [0; N];
        buff.copy_from_slice(bytes);
        Ok(buff)
    }

    fn pop_usize(&mut self) -> Result<usize> {
        let bytes = self.pop_n()?;
        u64::from_be_bytes(bytes)
            .try_into()
            .map_err(|_| Error::InvalidSize)
    }

    fn parse_str_inner(&mut self, len: usize) -> Result<&'de str> {
        let bytes = self.pop_slice(len)?;
        let s = core::str::from_utf8(bytes)?;
        Ok(s)
    }

    fn parse_unknown_len_str(&mut self) -> Result<&'de str> {
        let len = self
            .input
            .windows(UNSIZED_STRING_END_MARKER.len())
            .position(|bytes| bytes == UNSIZED_STRING_END_MARKER)
            .ok_or(Error::Eof)?;
        let s = self.parse_str_inner(len)?;
        self.pop_slice(UNSIZED_STRING_END_MARKER.len())?;
        Ok(s)
    }

    fn parse_known_len_str(&mut self) -> Result<&'de str> {
        let len = self.pop_usize()?;
        self.parse_str_inner(len)
    }

    fn parse_str(&mut self) -> Result<&'de str> {
        match_tag! {
            self.pop_tag()?, "String",
            Tag::String => self.parse_known_len_str()
            Tag::NullTerminatedString => self.parse_unknown_len_str()
        }
    }

    fn parse_tuple<V>(&mut self, visitor: V) -> Result<V::Value>
    where
        V: Visitor<'de>,
    {
        check_tag!(Tag::Tuple, self.pop_tag()?, "Tuple");
        let [len] = self.pop_n()?;
        visitor.visit_seq(SeqDeserializer::new_with_len(self, len.into()))
    }

    fn parse_tuple_struct<V>(&mut self, visitor: V) -> Result<V::Value>
    where
        V: Visitor<'de>,
    {
        check_tag!(Tag::TupleStruct, self.pop_tag()?, "TupleStruct");
        let [len] = self.pop_n()?;
        visitor.visit_seq(SeqDeserializer::new_with_len(self, len.into()))
    }

    fn parse_struct<V>(&mut self, visitor: V) -> Result<V::Value>
    where
        V: Visitor<'de>,
    {
        check_tag!(Tag::Struct, self.pop_tag()?, "Struct");
        let de = StructDeserializer::new(self)?;
        visitor.visit_map(de)
    }
}

impl<'de, 'a> de::Deserializer<'de> for &'a mut Deserializer<'de> {
    type Error = Error;

    fn is_human_readable(&self) -> bool {
        false
    }

    fn deserialize_any<V>(self, visitor: V) -> Result<V::Value>
    where
        V: Visitor<'de>,
    {
        let tag = self.peek_tag()?;
        match tag {
            Tag::None | Tag::Some => self.deserialize_option(visitor),
            Tag::BoolFalse | Tag::BoolTrue => self.deserialize_bool(visitor),
            Tag::I8 => self.deserialize_i8(visitor),
            Tag::I16 => self.deserialize_i16(visitor),
            Tag::I32 => self.deserialize_i32(visitor),
            Tag::I64 => self.deserialize_i64(visitor),
            Tag::U8 => self.deserialize_u8(visitor),
            Tag::U16 => self.deserialize_u16(visitor),
            Tag::U32 => self.deserialize_u32(visitor),
            Tag::U64 => self.deserialize_u64(visitor),
            Tag::F32 => self.deserialize_f32(visitor),
            Tag::F64 => self.deserialize_f64(visitor),
            Tag::Char1 | Tag::Char2 | Tag::Char3 | Tag::Char4 => self.deserialize_char(visitor),
            Tag::String | Tag::NullTerminatedString => self.deserialize_string(visitor),
            Tag::ByteArray => self.deserialize_byte_buf(visitor),
            Tag::Unit => self.deserialize_unit(visitor),
            Tag::UnitStruct => self.deserialize_unit_struct("", visitor),
            Tag::UnitVariant | Tag::NewTypeVariant | Tag::TupleVariant | Tag::StructVariant => {
                self.deserialize_enum("", &[], visitor)
            }
            Tag::NewTypeStruct => self.deserialize_newtype_struct("", visitor),
            Tag::Seq | Tag::UnsizedSeq => self.deserialize_seq(visitor),
            Tag::Tuple => self.parse_tuple(visitor),
            Tag::TupleStruct => self.parse_tuple_struct(visitor),
            Tag::Map | Tag::UnsizedMap => self.deserialize_map(visitor),
            Tag::Struct => self.parse_struct(visitor),
            #[cfg(not(no_integer128))]
            Tag::I128 => self.deserialize_i128(visitor),
            #[cfg(not(no_integer128))]
            Tag::U128 => self.deserialize_u128(visitor),
            Tag::UnsizedSeqEnd => Err(Error::TagParsingError(TagParsingError::unexpected(
                "Any tag other than end of sequence",
                Tag::UnsizedSeqEnd,
            ))),
        }
    }

    fn deserialize_bool<V>(self, visitor: V) -> Result<V::Value>
    where
        V: Visitor<'de>,
    {
        match_tag! {
            self.pop_tag()?, "Boolean",
            Tag::BoolFalse => visitor.visit_bool(false)
            Tag::BoolTrue => visitor.visit_bool(true)
        }
    }

    implement_number!(deserialize_i8, visit_i8, i8, Tag::I8, "i8");
    implement_number!(deserialize_i16, visit_i16, i16, Tag::I16, "i16");
    implement_number!(deserialize_i32, visit_i32, i32, Tag::I32, "i32");
    implement_number!(deserialize_i64, visit_i64, i64, Tag::I64, "i64");
    implement_number!(deserialize_u8, visit_u8, u8, Tag::U8, "u8");
    implement_number!(deserialize_u16, visit_u16, u16, Tag::U16, "u16");
    implement_number!(deserialize_u32, visit_u32, u32, Tag::U32, "u32");
    implement_number!(deserialize_u64, visit_u64, u64, Tag::U64, "u64");
    implement_number!(deserialize_f32, visit_f32, f32, Tag::F32, "f32");
    implement_number!(deserialize_f64, visit_f64, f64, Tag::F64, "f64");

    serde_if_integer128! {
        implement_number!(deserialize_i128, visit_i128, i128, Tag::I128, "i128");
        implement_number!(deserialize_u128, visit_u128, u128, Tag::U128, "u128");
    }

    fn deserialize_char<V>(self, visitor: V) -> Result<V::Value>
    where
        V: Visitor<'de>,
    {
        let len = match_tag! {
            self.pop_tag()?, "char",
            Tag::Char1 => 1
            Tag::Char2 => 2
            Tag::Char3 => 3
            Tag::Char4 => 4
        };
        let bytes = self.pop_slice(len)?;
        // bytes is at least 1 byte, so the decoded &str is not empty,
        // unwraping would be ok but from my test it is not optimised away,
        // unwrap_unchecked could be use but I try to keep it unsafe-free, so unwrap_or_default it is
        let c = core::str::from_utf8(bytes)?
            .chars()
            .next()
            .unwrap_or_default();
        visitor.visit_char(c)
    }

    fn deserialize_str<V>(self, visitor: V) -> Result<V::Value>
    where
        V: Visitor<'de>,
    {
        let s = self.parse_str()?;
        visitor.visit_borrowed_str(s)
    }

    fn deserialize_string<V>(self, visitor: V) -> Result<V::Value>
    where
        V: Visitor<'de>,
    {
        self.deserialize_str(visitor)
    }

    fn deserialize_bytes<V>(self, visitor: V) -> Result<V::Value>
    where
        V: Visitor<'de>,
    {
        check_tag!(Tag::ByteArray, self.pop_tag()?, "ByteArray");
        let len = self.pop_usize()?;
        let bytes = self.pop_slice(len)?;
        visitor.visit_borrowed_bytes(bytes)
    }

    fn deserialize_byte_buf<V>(self, visitor: V) -> Result<V::Value>
    where
        V: Visitor<'de>,
    {
        self.deserialize_bytes(visitor)
    }

    fn deserialize_option<V>(self, visitor: V) -> Result<V::Value>
    where
        V: Visitor<'de>,
    {
        match_tag! {
            self.pop_tag()?, "Option",
            Tag::None => visitor.visit_none()
            Tag::Some => visitor.visit_some(self)
        }
    }

    fn deserialize_unit<V>(self, visitor: V) -> Result<V::Value>
    where
        V: Visitor<'de>,
    {
        check_tag!(Tag::Unit, self.pop_tag()?, "Unit");
        visitor.visit_unit()
    }

    fn deserialize_unit_struct<V>(self, _name: &'static str, visitor: V) -> Result<V::Value>
    where
        V: Visitor<'de>,
    {
        check_tag!(Tag::UnitStruct, self.pop_tag()?, "UnitStruct");
        visitor.visit_unit()
    }

    fn deserialize_newtype_struct<V>(self, _name: &'static str, visitor: V) -> Result<V::Value>
    where
        V: Visitor<'de>,
    {
        check_tag!(Tag::NewTypeStruct, self.pop_tag()?, "NewTypeStruct");
        visitor.visit_newtype_struct(self)
    }

    fn deserialize_seq<V>(self, visitor: V) -> Result<V::Value>
    where
        V: Visitor<'de>,
    {
        let seq_des = match_tag! {
            self.pop_tag()?, "Sequence",
            Tag::Seq => SeqDeserializer::new(self)?
            Tag::UnsizedSeq => SeqDeserializer::new_unsized(self)
        };
        visitor.visit_seq(seq_des)
    }

    fn deserialize_tuple<V>(self, len: usize, visitor: V) -> Result<V::Value>
    where
        V: Visitor<'de>,
    {
        check_tag!(Tag::Tuple, self.pop_tag()?, "Tuple");
        let [encoded_len] = self.pop_n()?;
        let encoded_len: usize = encoded_len.into();
        if len != encoded_len {
            return Err(Err::SeqSizeMismatch {
                expected: len,
                got: encoded_len,
            });
        }
        visitor.visit_seq(SeqDeserializer::new_with_len(self, len))
    }

    fn deserialize_tuple_struct<V>(
        self,
        _name: &'static str,
        len: usize,
        visitor: V,
    ) -> Result<V::Value>
    where
        V: Visitor<'de>,
    {
        check_tag!(Tag::TupleStruct, self.pop_tag()?, "TupleStruct");
        let [encoded_len] = self.pop_n()?;
        let encoded_len: usize = encoded_len.into();
        if len != encoded_len {
            return Err(Err::SeqSizeMismatch {
                expected: len,
                got: encoded_len,
            });
        }
        visitor.visit_seq(SeqDeserializer::new_with_len(self, len))
    }

    fn deserialize_map<V>(self, visitor: V) -> Result<V::Value>
    where
        V: Visitor<'de>,
    {
        let seq_des = match_tag! {
            self.pop_tag()?, "Map",
            Tag::Map => SeqDeserializer::new(self)?
            Tag::UnsizedMap => SeqDeserializer::new_unsized(self)
        };
        visitor.visit_map(seq_des)
    }

    fn deserialize_struct<V>(
        self,
        _name: &'static str,
        fields: &'static [&'static str],
        visitor: V,
    ) -> Result<V::Value>
    where
        V: Visitor<'de>,
    {
        check_tag!(Tag::Struct, self.pop_tag()?, "Struct");
        let len = fields.len();
        let [encoded_len] = self.pop_n()?;
        let encoded_len: usize = encoded_len.into();
        if len != encoded_len {
            return Err(Err::SeqSizeMismatch {
                expected: len,
                got: encoded_len,
            });
        }
        visitor.visit_map(StructDeserializer::new_with_len(self, len))
    }

    fn deserialize_enum<V>(
        self,
        _name: &'static str,
        _variants: &'static [&'static str],
        visitor: V,
    ) -> Result<V::Value>
    where
        V: Visitor<'de>,
    {
        check_tag!(
            Tag::UnitVariant | Tag::NewTypeVariant | Tag::TupleVariant | Tag::StructVariant,
            self.peek_tag()?,
            "Enum"
        );
        visitor.visit_enum(self)
    }

    fn deserialize_identifier<V>(self, visitor: V) -> Result<V::Value>
    where
        V: Visitor<'de>,
    {
        match_tag! {
            self.pop_tag()?, "Identifier",
            Tag::UnitVariant | Tag::NewTypeVariant | Tag::TupleVariant | Tag::StructVariant => {
                let bytes = self.pop_n()?;
                visitor.visit_u32(u32::from_be_bytes(bytes))
            }
            Tag::String => {
                let s = self.parse_known_len_str()?;
                visitor.visit_borrowed_str(s)
            }
        }
    }

    fn deserialize_ignored_any<V>(self, visitor: V) -> Result<V::Value>
    where
        V: Visitor<'de>,
    {
        self.deserialize_any(visitor)
    }
}

struct SeqDeserializer<'a, 'de: 'a> {
    de: &'a mut Deserializer<'de>,
    remaining: Option<usize>,
}

impl<'a, 'de> SeqDeserializer<'a, 'de> {
    fn new(de: &'a mut Deserializer<'de>) -> Result<Self> {
        let len = de.pop_usize()?;
        Ok(Self::new_with_len(de, len))
    }

    fn new_with_len(de: &'a mut Deserializer<'de>, len: usize) -> Self {
        Self {
            de,
            remaining: Some(len),
        }
    }

    fn new_unsized(de: &'a mut Deserializer<'de>) -> Self {
        Self {
            de,
            remaining: None,
        }
    }
}

impl<'de, 'a> SeqAccess<'de> for SeqDeserializer<'a, 'de> {
    type Error = Error;

    fn next_element_seed<T>(&mut self, seed: T) -> Result<Option<T::Value>>
    where
        T: de::DeserializeSeed<'de>,
    {
        if let Some(remaining) = self.remaining.as_mut() {
            if *remaining == 0 {
                return Ok(None);
            }
            *remaining -= 1;
        } else if let Tag::UnsizedSeqEnd = self.de.peek_tag()? {
            self.de.pop_tag()?;
            return Ok(None);
        }

        seed.deserialize(&mut *self.de).map(Some)
    }

    fn size_hint(&self) -> Option<usize> {
        self.remaining
    }
}

impl<'de, 'a> MapAccess<'de> for SeqDeserializer<'a, 'de> {
    type Error = Error;

    fn next_key_seed<K>(&mut self, seed: K) -> Result<Option<K::Value>>
    where
        K: de::DeserializeSeed<'de>,
    {
        self.next_element_seed(seed)
    }

    fn next_value_seed<V>(&mut self, seed: V) -> Result<V::Value>
    where
        V: de::DeserializeSeed<'de>,
    {
        seed.deserialize(&mut *self.de)
    }

    fn size_hint(&self) -> Option<usize> {
        self.remaining
    }
}

impl<'a, 'de> EnumAccess<'de> for &'a mut Deserializer<'de> {
    type Error = Error;
    type Variant = Self;

    fn variant_seed<V>(self, seed: V) -> Result<(V::Value, Self::Variant)>
    where
        V: de::DeserializeSeed<'de>,
    {
        let val = seed.deserialize(&mut *self)?;
        Ok((val, self))
    }
}

impl<'a, 'de> VariantAccess<'de> for &'a mut Deserializer<'de> {
    type Error = Error;

    fn unit_variant(self) -> Result<()> {
        // check_tag!(Tag::UnitVariant, self, "UnitVariant");
        Ok(())
    }

    fn newtype_variant_seed<T>(self, seed: T) -> Result<T::Value>
    where
        T: de::DeserializeSeed<'de>,
    {
        // check_tag!(Tag::NewTypeVariant, self, "NewTypeVariant");
        seed.deserialize(self)
    }

    fn tuple_variant<V>(self, len: usize, visitor: V) -> Result<V::Value>
    where
        V: Visitor<'de>,
    {
        // check_tag!(Tag::TupleVariant, self, "TupleVariant");
        visitor.visit_seq(SeqDeserializer::new_with_len(self, len))
    }

    fn struct_variant<V>(self, fields: &'static [&'static str], visitor: V) -> Result<V::Value>
    where
        V: Visitor<'de>,
    {
        // check_tag!(Tag::StructVariant, self, "StructVariant");
        visitor.visit_seq(SeqDeserializer::new_with_len(self, fields.len()))
    }
}

struct StructDeserializer<'a, 'de: 'a> {
    de: &'a mut Deserializer<'de>,
    remaining: usize,
    current_index: u64,
}

impl<'a, 'de> StructDeserializer<'a, 'de> {
    fn new(de: &'a mut Deserializer<'de>) -> Result<Self> {
        let [len] = de.pop_n()?;
        Ok(Self::new_with_len(de, len.into()))
    }

    fn new_with_len(de: &'a mut Deserializer<'de>, len: usize) -> Self {
        Self {
            de,
            remaining: len,
            current_index: 0,
        }
    }
}

impl<'de, 'a> MapAccess<'de> for StructDeserializer<'a, 'de> {
    type Error = Error;

    fn next_key_seed<K>(&mut self, seed: K) -> Result<Option<K::Value>>
    where
        K: de::DeserializeSeed<'de>,
    {
        if self.remaining == 0 {
            return Ok(None);
        }

        let de = self.current_index.into_deserializer();
        self.remaining -= 1;
        self.current_index += 1;

        seed.deserialize(de).map(Some)
    }

    fn next_value_seed<V>(&mut self, seed: V) -> Result<V::Value>
    where
        V: de::DeserializeSeed<'de>,
    {
        seed.deserialize(&mut *self.de)
    }

    fn size_hint(&self) -> Option<usize> {
        Some(self.remaining)
    }
}
