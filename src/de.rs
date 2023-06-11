use serde::{
    de::{self, EnumAccess, MapAccess, SeqAccess, VariantAccess, Visitor},
    serde_if_integer128, Deserialize,
};

use crate::{
    error::{Error, NoWriterError, Result},
    UNSIZED_STRING_END_MARKER,
};

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

    fn pop_bytes_seq(&mut self) -> Result<&'de [u8]> {
        let len = self.pop_usize()?;
        self.pop_slice(len)
    }

    fn parse_str(&mut self) -> Result<&'de str> {
        let len_bytes = self.pop_n()?;
        let len = u64::from_be_bytes(len_bytes);
        let len = if len == u64::MAX {
            // unknown str length, "null" terminated
            self.input
                .windows(UNSIZED_STRING_END_MARKER.len())
                .position(|bytes| bytes == UNSIZED_STRING_END_MARKER)
                .ok_or(Error::Eof)?
        } else {
            len.try_into().map_err(|_| Error::InvalidSize)?
        };

        let bytes = self.pop_slice(len)?;
        let s = core::str::from_utf8(bytes)?;
        Ok(s)
    }
}

macro_rules! implement_number {
    ($fn_name:ident, $visitor_fn_name:ident, $t:ident) => {
        fn $fn_name<V>(self, visitor: V) -> Result<V::Value>
        where
            V: Visitor<'de>,
        {
            let bytes = self.pop_n()?;
            visitor.$visitor_fn_name($t::from_be_bytes(bytes))
        }
    };
}

impl<'de, 'a> de::Deserializer<'de> for &'a mut Deserializer<'de> {
    type Error = Error<NoWriterError>;

    fn is_human_readable(&self) -> bool {
        false
    }

    fn deserialize_any<V>(self, _visitor: V) -> Result<V::Value>
    where
        V: Visitor<'de>,
    {
        Err(Error::Unimplemented(
            "serde::de::Deserializer::deserialize_any",
        ))
    }

    fn deserialize_bool<V>(self, visitor: V) -> Result<V::Value>
    where
        V: Visitor<'de>,
    {
        let [byte] = self.pop_n::<1>()?;
        match byte {
            0 => visitor.visit_bool(false),
            1 => visitor.visit_bool(true),
            _ => Err(Error::InvalidBool(byte)),
        }
    }

    implement_number!(deserialize_i8, visit_i8, i8);
    implement_number!(deserialize_i16, visit_i16, i16);
    implement_number!(deserialize_i32, visit_i32, i32);
    implement_number!(deserialize_i64, visit_i64, i64);
    implement_number!(deserialize_u8, visit_u8, u8);
    implement_number!(deserialize_u16, visit_u16, u16);
    implement_number!(deserialize_u32, visit_u32, u32);
    implement_number!(deserialize_u64, visit_u64, u64);
    implement_number!(deserialize_f32, visit_f32, f32);
    implement_number!(deserialize_f64, visit_f64, f64);

    serde_if_integer128! {
        implement_number!(deserialize_i128, visit_i128, i128);
        implement_number!(deserialize_u128, visit_u128, u128);
    }

    fn deserialize_char<V>(self, visitor: V) -> Result<V::Value>
    where
        V: Visitor<'de>,
    {
        let bytes = self.pop_n()?;
        let c = u32::from_be_bytes(bytes);
        let c = char::from_u32(c).ok_or(Error::InvalidChar(c))?;
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
        let bytes = self.pop_bytes_seq()?;
        visitor.visit_bytes(bytes)
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
        let [byte] = self.pop_n()?;
        match byte {
            0 => visitor.visit_none(),
            1 => visitor.visit_some(self),
            _ => Err(Error::InvalidOptionTag(byte)),
        }
    }

    fn deserialize_unit<V>(self, visitor: V) -> Result<V::Value>
    where
        V: Visitor<'de>,
    {
        visitor.visit_unit()
    }

    fn deserialize_unit_struct<V>(self, _name: &'static str, visitor: V) -> Result<V::Value>
    where
        V: Visitor<'de>,
    {
        self.deserialize_unit(visitor)
    }

    fn deserialize_newtype_struct<V>(self, _name: &'static str, visitor: V) -> Result<V::Value>
    where
        V: Visitor<'de>,
    {
        visitor.visit_newtype_struct(self)
    }

    fn deserialize_seq<V>(self, visitor: V) -> Result<V::Value>
    where
        V: Visitor<'de>,
    {
        let seq_des = SeqDeserializer::new(self)?;
        visitor.visit_seq(seq_des)
    }

    fn deserialize_tuple<V>(self, len: usize, visitor: V) -> Result<V::Value>
    where
        V: Visitor<'de>,
    {
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
        visitor.visit_seq(SeqDeserializer::new_with_len(self, len))
    }

    fn deserialize_map<V>(self, visitor: V) -> Result<V::Value>
    where
        V: Visitor<'de>,
    {
        let seq_des = SeqDeserializer::new(self)?;
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
        visitor.visit_seq(SeqDeserializer::new_with_len(self, fields.len()))
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
        visitor.visit_enum(self)
    }

    fn deserialize_identifier<V>(self, visitor: V) -> Result<V::Value>
    where
        V: Visitor<'de>,
    {
        self.deserialize_u32(visitor)
    }

    fn deserialize_ignored_any<V>(self, _visitor: V) -> Result<V::Value>
    where
        V: Visitor<'de>,
    {
        Err(Error::Unimplemented(
            "serde::de::Deserializer::deserialize_ignored_any",
        ))
    }
}

struct SeqDeserializer<'a, 'de: 'a> {
    de: &'a mut Deserializer<'de>,
    remaining: usize,
}

impl<'a, 'de> SeqDeserializer<'a, 'de> {
    fn new(de: &'a mut Deserializer<'de>) -> Result<Self> {
        let len = de.pop_usize()?;
        Ok(Self::new_with_len(de, len))
    }

    fn new_with_len(de: &'a mut Deserializer<'de>, len: usize) -> Self {
        Self { de, remaining: len }
    }
}

impl<'de, 'a> SeqAccess<'de> for SeqDeserializer<'a, 'de> {
    type Error = Error<NoWriterError>;

    fn next_element_seed<T>(&mut self, seed: T) -> Result<Option<T::Value>>
    where
        T: de::DeserializeSeed<'de>,
    {
        if self.remaining == 0 {
            return Ok(None);
        }

        self.remaining -= 1;

        seed.deserialize(&mut *self.de).map(Some)
    }

    fn size_hint(&self) -> Option<usize> {
        Some(self.remaining)
    }
}

impl<'de, 'a> MapAccess<'de> for SeqDeserializer<'a, 'de> {
    type Error = Error<NoWriterError>;

    fn next_key_seed<K>(&mut self, seed: K) -> Result<Option<K::Value>>
    where
        K: de::DeserializeSeed<'de>,
    {
        if self.remaining == 0 {
            return Ok(None);
        }

        self.remaining -= 1;

        seed.deserialize(&mut *self.de).map(Some)
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

impl<'a, 'de> EnumAccess<'de> for &'a mut Deserializer<'de> {
    type Error = Error<NoWriterError>;
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
    type Error = Error<NoWriterError>;

    fn unit_variant(self) -> Result<()> {
        Ok(())
    }

    fn newtype_variant_seed<T>(self, seed: T) -> Result<T::Value>
    where
        T: de::DeserializeSeed<'de>,
    {
        seed.deserialize(self)
    }

    fn tuple_variant<V>(self, len: usize, visitor: V) -> Result<V::Value>
    where
        V: Visitor<'de>,
    {
        visitor.visit_seq(SeqDeserializer::new_with_len(self, len))
    }

    fn struct_variant<V>(self, fields: &'static [&'static str], visitor: V) -> Result<V::Value>
    where
        V: Visitor<'de>,
    {
        visitor.visit_seq(SeqDeserializer::new_with_len(self, fields.len()))
    }
}
