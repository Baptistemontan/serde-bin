use serde::{
    de::{self, EnumAccess, MapAccess, SeqAccess, VariantAccess, Visitor},
    Deserialize, serde_if_integer128,
};

use crate::error::{Error, Result};

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
    fn pop_n<const N: usize>(&mut self) -> Result<[u8; N]> {
        let bytes = self.input.get(..N).ok_or(Error::Eof)?;
        let mut buff = [0; N];

        buff.copy_from_slice(bytes);

        self.input = &self.input[N..];

        Ok(buff)
    }

    fn pop_usize(&mut self) -> Result<usize> {
        let bytes = self.pop_n()?;
        u64::from_be_bytes(bytes)
            .try_into()
            .map_err(|_| Error::InvalidSize)
    }

    fn pop_slice(&mut self) -> Result<&'de [u8]> {
        let len = self.pop_usize()?;
        let bytes = self.input.get(..len).ok_or(Error::Eof)?;
        self.input = &self.input[len..];
        Ok(bytes)
    }

    fn parse_str(&mut self) -> Result<&'de str> {
        let bytes = self.pop_slice()?;
        let s = std::str::from_utf8(bytes)?;
        Ok(s)
    }
}

impl<'de, 'a> de::Deserializer<'de> for &'a mut Deserializer<'de> {
    type Error = Error;

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

    fn deserialize_i8<V>(self, visitor: V) -> Result<V::Value>
    where
        V: Visitor<'de>,
    {
        let bytes = self.pop_n()?;
        visitor.visit_i8(i8::from_be_bytes(bytes))
    }

    fn deserialize_i16<V>(self, visitor: V) -> Result<V::Value>
    where
        V: Visitor<'de>,
    {
        let bytes = self.pop_n()?;
        visitor.visit_i16(i16::from_be_bytes(bytes))
    }

    fn deserialize_i32<V>(self, visitor: V) -> Result<V::Value>
    where
        V: Visitor<'de>,
    {
        let bytes = self.pop_n()?;
        visitor.visit_i32(i32::from_be_bytes(bytes))
    }

    fn deserialize_i64<V>(self, visitor: V) -> Result<V::Value>
    where
        V: Visitor<'de>,
    {
        let bytes = self.pop_n()?;
        visitor.visit_i64(i64::from_be_bytes(bytes))
    }

    serde_if_integer128! {
        
        fn deserialize_i128<V>(self, visitor: V) -> Result<V::Value>
        where
            V: Visitor<'de>
        {
            let bytes = self.pop_n()?;
            visitor.visit_i128(i128::from_be_bytes(bytes))
        }

    }

    fn deserialize_u8<V>(self, visitor: V) -> Result<V::Value>
    where
        V: Visitor<'de>,
    {
        let bytes = self.pop_n()?;
        visitor.visit_u8(u8::from_be_bytes(bytes))
    }

    fn deserialize_u16<V>(self, visitor: V) -> Result<V::Value>
    where
        V: Visitor<'de>,
    {
        let bytes = self.pop_n()?;
        visitor.visit_u16(u16::from_be_bytes(bytes))
    }

    fn deserialize_u32<V>(self, visitor: V) -> Result<V::Value>
    where
        V: Visitor<'de>,
    {
        let bytes = self.pop_n()?;
        visitor.visit_u32(u32::from_be_bytes(bytes))
    }

    fn deserialize_u64<V>(self, visitor: V) -> Result<V::Value>
    where
        V: Visitor<'de>,
    {
        let bytes = self.pop_n()?;
        visitor.visit_u64(u64::from_be_bytes(bytes))
    }

    serde_if_integer128! {
        
        fn deserialize_u128<V>(self, visitor: V) -> Result<V::Value>
        where
            V: Visitor<'de>
        {
            let bytes = self.pop_n()?;
            visitor.visit_u128(u128::from_be_bytes(bytes))
        }

    }

    fn deserialize_f32<V>(self, visitor: V) -> Result<V::Value>
    where
        V: Visitor<'de>,
    {
        let bytes = self.pop_n()?;
        visitor.visit_f32(f32::from_be_bytes(bytes))
    }

    fn deserialize_f64<V>(self, visitor: V) -> Result<V::Value>
    where
        V: Visitor<'de>,
    {
        let bytes = self.pop_n()?;
        visitor.visit_f64(f64::from_be_bytes(bytes))
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
        let bytes = self.pop_slice()?;
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
        visitor.visit_enum(EnumDeserializer { de: self })
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
    type Error = Error;

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
}

impl<'de, 'a> MapAccess<'de> for SeqDeserializer<'a, 'de> {
    type Error = Error;

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
}

struct EnumDeserializer<'a, 'de: 'a> {
    de: &'a mut Deserializer<'de>,
}

impl<'a, 'de> EnumAccess<'de> for EnumDeserializer<'a, 'de> {
    type Error = Error;
    type Variant = Self;

    fn variant_seed<V>(self, seed: V) -> Result<(V::Value, Self::Variant)>
    where
        V: de::DeserializeSeed<'de>,
    {
        let val = seed.deserialize(&mut *self.de)?;
        Ok((val, self))
    }
}

impl<'a, 'de> VariantAccess<'de> for EnumDeserializer<'a, 'de> {
    type Error = Error;

    fn unit_variant(self) -> Result<()> {
        Ok(())
    }

    fn newtype_variant_seed<T>(self, seed: T) -> Result<T::Value>
    where
        T: de::DeserializeSeed<'de>,
    {
        seed.deserialize(self.de)
    }

    fn tuple_variant<V>(self, len: usize, visitor: V) -> Result<V::Value>
    where
        V: Visitor<'de>,
    {
        visitor.visit_seq(SeqDeserializer::new_with_len(self.de, len))
    }

    fn struct_variant<V>(self, fields: &'static [&'static str], visitor: V) -> Result<V::Value>
    where
        V: Visitor<'de>,
    {
        visitor.visit_seq(SeqDeserializer::new_with_len(self.de, fields.len()))
    }
}
