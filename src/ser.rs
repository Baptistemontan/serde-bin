use serde::{ser, serde_if_integer128, Serialize};
use std::io::Write;

use crate::error::{Error, Result};

pub struct Serializer<T> {
    writer: T,
}

impl<W: Write> Serializer<W> {
    pub fn new(writer: W) -> Self {
        Serializer { writer }
    }
    pub fn to_writer<T>(value: &T, writer: W) -> Result<()>
    where
        T: Serialize,
    {
        let mut serializer = Serializer::new(writer);

        value.serialize(&mut serializer)
    }
}

pub fn to_writer<W, T>(value: &T, writer: W) -> Result<()>
where
    T: Serialize,
    W: Write,
{
    Serializer::to_writer(value, writer)
}

pub fn to_bytes<W, T>(value: &T) -> Result<Vec<u8>>
where
    T: Serialize,
    W: Write,
{
    let mut output = vec![];
    Serializer::to_writer(value, &mut output)?;
    Ok(output)
}

impl<'a, W: Write> ser::Serializer for &'a mut Serializer<W> {
    type Ok = ();

    type Error = Error;

    type SerializeSeq = SeqSerializer<'a, W>;
    type SerializeTuple = Self;
    type SerializeTupleStruct = Self;
    type SerializeTupleVariant = Self;
    type SerializeMap = SeqSerializer<'a, W>;
    type SerializeStruct = Self;
    type SerializeStructVariant = Self;

    fn is_human_readable(&self) -> bool {
        false
    }

    fn serialize_bool(self, v: bool) -> Result<Self::Ok> {
        let byte: u8 = v.into();
        self.writer.write_all(&[byte])?;
        Ok(())
    }

    fn serialize_i8(self, v: i8) -> Result<Self::Ok> {
        self.writer.write_all(&v.to_be_bytes())?;
        Ok(())
    }

    fn serialize_i16(self, v: i16) -> Result<Self::Ok> {
        self.writer.write_all(&v.to_be_bytes())?;
        Ok(())
    }

    fn serialize_i32(self, v: i32) -> Result<Self::Ok> {
        self.writer.write_all(&v.to_be_bytes())?;
        Ok(())
    }

    fn serialize_i64(self, v: i64) -> Result<Self::Ok> {
        self.writer.write_all(&v.to_be_bytes())?;
        Ok(())
    }

    serde_if_integer128! {

        fn serialize_i128(self, v: i128) -> std::result::Result<Self::Ok,Self::Error> {
            self.writer.write_all(&v.to_be_bytes())?;
            Ok(())
        }

    }

    fn serialize_u8(self, v: u8) -> Result<Self::Ok> {
        self.writer.write_all(&v.to_be_bytes())?;
        Ok(())
    }

    fn serialize_u16(self, v: u16) -> Result<Self::Ok> {
        self.writer.write_all(&v.to_be_bytes())?;
        Ok(())
    }

    fn serialize_u32(self, v: u32) -> Result<Self::Ok> {
        self.writer.write_all(&v.to_be_bytes())?;
        Ok(())
    }

    fn serialize_u64(self, v: u64) -> Result<Self::Ok> {
        self.writer.write_all(&v.to_be_bytes())?;
        Ok(())
    }

    fn serialize_f32(self, v: f32) -> Result<Self::Ok> {
        self.writer.write_all(&v.to_be_bytes())?;
        Ok(())
    }

    fn serialize_f64(self, v: f64) -> Result<Self::Ok> {
        self.writer.write_all(&v.to_be_bytes())?;
        Ok(())
    }

    serde_if_integer128! {

        fn serialize_u128(self, v: u128) -> std::result::Result<Self::Ok,Self::Error> {
            self.writer.write_all(&v.to_be_bytes())?;
            Ok(())
        }

    }

    fn serialize_char(self, v: char) -> Result<Self::Ok> {
        let bytes: u32 = v.into();
        self.writer.write_all(&bytes.to_be_bytes())?;
        Ok(())
    }

    fn serialize_str(self, v: &str) -> Result<Self::Ok> {
        Self::serialize_bytes(self, v.as_bytes())
    }

    fn serialize_bytes(self, v: &[u8]) -> Result<Self::Ok> {
        let len = v.len() as u64;
        self.writer.write_all(&len.to_be_bytes())?;
        self.writer.write_all(v)?;
        Ok(())
    }

    fn serialize_unit(self) -> Result<Self::Ok> {
        Ok(())
    }

    fn serialize_unit_struct(self, _name: &'static str) -> Result<Self::Ok> {
        Self::serialize_unit(self)
    }

    fn serialize_unit_variant(
        self,
        _name: &'static str,
        variant_index: u32,
        _variant: &'static str,
    ) -> Result<Self::Ok> {
        Self::serialize_u32(self, variant_index)
    }

    fn serialize_newtype_struct<T: ?Sized>(self, _name: &'static str, value: &T) -> Result<Self::Ok>
    where
        T: Serialize,
    {
        value.serialize(self)
    }

    fn serialize_newtype_variant<T: ?Sized>(
        self,
        _name: &'static str,
        variant_index: u32,
        _variant: &'static str,
        value: &T,
    ) -> Result<Self::Ok>
    where
        T: Serialize,
    {
        self.writer.write_all(&variant_index.to_be_bytes())?;
        value.serialize(self)
    }

    fn serialize_seq(self, len: Option<usize>) -> Result<Self::SerializeSeq> {
        match len {
            Some(len) => {
                let len: u64 = len as u64;
                self.writer.write_all(&len.to_be_bytes())?;
                Ok(SeqSerializer::new_known(self))
            }
            None => Ok(SeqSerializer::new_unknown(self)),
        }
    }

    fn serialize_none(self) -> Result<Self::Ok> {
        self.writer.write_all(&[0])?;
        Ok(())
    }

    fn serialize_some<T: ?Sized>(self, value: &T) -> Result<Self::Ok>
    where
        T: Serialize,
    {
        self.writer.write_all(&[1])?;
        value.serialize(self)
    }

    fn serialize_tuple(self, _len: usize) -> Result<Self::SerializeTuple> {
        Ok(self)
    }

    fn serialize_tuple_struct(
        self,
        _name: &'static str,
        _len: usize,
    ) -> Result<Self::SerializeTupleStruct> {
        Ok(self)
    }

    fn serialize_tuple_variant(
        self,
        _name: &'static str,
        variant_index: u32,
        _variant: &'static str,
        _len: usize,
    ) -> Result<Self::SerializeTupleVariant> {
        self.writer.write_all(&variant_index.to_be_bytes())?;
        Ok(self)
    }

    fn serialize_map(self, len: Option<usize>) -> Result<Self::SerializeMap> {
        match len {
            Some(len) => {
                let len: u64 = len as u64;
                self.writer.write_all(&len.to_be_bytes())?;
                Ok(SeqSerializer::new_known(self))
            }
            None => Ok(SeqSerializer::new_unknown(self)),
        }
    }

    fn serialize_struct(self, _name: &'static str, _len: usize) -> Result<Self::SerializeStruct> {
        Ok(self)
    }

    fn serialize_struct_variant(
        self,
        _name: &'static str,
        variant_index: u32,
        _variant: &'static str,
        _len: usize,
    ) -> Result<Self::SerializeStructVariant> {
        self.writer.write_all(&variant_index.to_be_bytes())?;
        Ok(self)
    }
}

pub enum SeqSerializer<'a, W> {
    KnownSize(&'a mut Serializer<W>),
    UnknownSize {
        count: u64,
        bytes: Vec<u8>,
        serializer: &'a mut Serializer<W>,
    },
}

impl<'a, W: Write> SeqSerializer<'a, W> {
    pub fn new_known(serializer: &'a mut Serializer<W>) -> Self {
        Self::KnownSize(serializer)
    }

    pub fn new_unknown(serializer: &'a mut Serializer<W>) -> Self {
        Self::UnknownSize {
            count: 0,
            bytes: Vec::new(),
            serializer,
        }
    }

    pub fn ser_value<T: ?Sized>(&mut self, value: &T) -> Result<()>
    where
        T: Serialize,
    {
        match self {
            SeqSerializer::KnownSize(serializer) => value.serialize(&mut **serializer),
            SeqSerializer::UnknownSize { count, bytes, .. } => {
                let mut serializer = Serializer { writer: bytes };
                *count += 1;
                value.serialize(&mut serializer)
            }
        }
    }

    pub fn finish(self) -> Result<()> {
        match self {
            SeqSerializer::KnownSize(_) => Ok(()),
            SeqSerializer::UnknownSize {
                count,
                bytes,
                serializer,
            } => {
                serializer.writer.write_all(&count.to_be_bytes())?;
                serializer.writer.write_all(&bytes)?;
                Ok(())
            }
        }
    }
}

impl<'a, W: Write> ser::SerializeSeq for SeqSerializer<'a, W> {
    type Ok = ();

    type Error = Error;

    fn serialize_element<T: ?Sized>(&mut self, value: &T) -> Result<()>
    where
        T: Serialize,
    {
        self.ser_value(value)
    }

    fn end(self) -> Result<Self::Ok> {
        self.finish()
    }
}

impl<'a, W: Write> ser::SerializeTuple for &'a mut Serializer<W> {
    type Ok = ();

    type Error = Error;

    fn serialize_element<T: ?Sized>(&mut self, value: &T) -> Result<()>
    where
        T: Serialize,
    {
        value.serialize(&mut **self)
    }

    fn end(self) -> Result<Self::Ok> {
        Ok(())
    }
}

impl<'a, W: Write> ser::SerializeTupleStruct for &'a mut Serializer<W> {
    type Ok = ();

    type Error = Error;

    fn serialize_field<T: ?Sized>(&mut self, value: &T) -> Result<()>
    where
        T: Serialize,
    {
        value.serialize(&mut **self)
    }

    fn end(self) -> Result<Self::Ok> {
        Ok(())
    }
}

impl<'a, W: Write> ser::SerializeTupleVariant for &'a mut Serializer<W> {
    type Ok = ();

    type Error = Error;

    fn serialize_field<T: ?Sized>(&mut self, value: &T) -> Result<()>
    where
        T: Serialize,
    {
        value.serialize(&mut **self)
    }

    fn end(self) -> Result<Self::Ok> {
        Ok(())
    }
}

impl<'a, W: Write> ser::SerializeMap for SeqSerializer<'a, W> {
    type Ok = ();

    type Error = Error;

    fn serialize_key<T: ?Sized>(&mut self, key: &T) -> Result<()>
    where
        T: Serialize,
    {
        self.ser_value(key)
    }

    fn serialize_value<T: ?Sized>(&mut self, value: &T) -> Result<()>
    where
        T: Serialize,
    {
        self.ser_value(value)
    }

    fn end(self) -> Result<Self::Ok> {
        self.finish()
    }
}

impl<'a, W: Write> ser::SerializeStruct for &'a mut Serializer<W> {
    type Ok = ();

    type Error = Error;

    fn serialize_field<T: ?Sized>(&mut self, _key: &'static str, value: &T) -> Result<()>
    where
        T: Serialize,
    {
        value.serialize(&mut **self)
    }

    fn end(self) -> Result<Self::Ok> {
        Ok(())
    }
}

impl<'a, W: Write> ser::SerializeStructVariant for &'a mut Serializer<W> {
    type Ok = ();

    type Error = Error;

    fn serialize_field<T: ?Sized>(&mut self, _key: &'static str, value: &T) -> Result<()>
    where
        T: Serialize,
    {
        value.serialize(&mut **self)
    }

    fn end(self) -> Result<Self::Ok> {
        Ok(())
    }
}
