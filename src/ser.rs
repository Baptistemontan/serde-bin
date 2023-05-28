use serde::{ser, serde_if_integer128, Serialize};

#[cfg(feature = "std")]
use std::io;

use crate::error::{Error, NoWriterError, Result};
use crate::write::{BuffWriter, DummyWriter, EndOfBuff, Write};
use core::fmt;

#[cfg(feature = "alloc")]
extern crate alloc;
#[cfg(feature = "alloc")]
use alloc::vec::Vec;

pub struct Serializer<T> {
    writer: T,
}

impl<W: Write> Serializer<W> {
    pub fn new(writer: W) -> Self {
        Serializer { writer }
    }
    pub fn to_writer<T>(value: &T, writer: W) -> Result<usize, W::Error>
    where
        T: Serialize,
    {
        let mut serializer = Serializer::new(writer);

        value.serialize(&mut serializer)
    }
}

#[cfg(feature = "std")]
pub fn to_writer<W, T>(value: &T, writer: W) -> Result<usize, W::Error>
where
    T: Serialize,
    W: Write,
{
    Serializer::to_writer(value, writer)
}

#[cfg(all(feature = "alloc", not(feature = "std")))]
pub fn to_bytes<T>(value: &T) -> Result<Vec<u8>, NoWriterError>
where
    T: Serialize,
{
    let mut output = Vec::new();
    Serializer::to_writer(value, &mut output)?;
    Ok(output)
}

#[cfg(feature = "std")]
pub fn to_bytes<T>(value: &T) -> Result<Vec<u8>, io::Error>
where
    T: Serialize,
{
    let mut output = Vec::new();
    Serializer::to_writer(value, &mut output)?;
    Ok(output)
}

pub fn to_buff<T>(value: &T, buff: &mut [u8]) -> Result<usize, EndOfBuff>
where
    T: Serialize,
{
    let buff_writer = BuffWriter::new(buff);
    Serializer::to_writer(value, buff_writer)
}

pub fn get_serialized_size<T>(value: &T) -> Result<usize, NoWriterError>
where
    T: Serialize,
{
    Serializer::to_writer(value, DummyWriter)
}

macro_rules! implement_number {
    ($fn_name:ident, $t:ident) => {
        fn $fn_name(self, value: $t) -> Result<Self::Ok, W::Error> {
            self.writer
                .write_bytes(&value.to_be_bytes())
                .map_err(Into::into)
        }
    };
}

impl<'a, W: Write> ser::Serializer for &'a mut Serializer<W> {
    type Ok = usize;

    type Error = Error<W::Error>;

    type SerializeSeq = SeqSerializer<'a, W>;
    type SerializeTuple = SeqSerializer<'a, W>;
    type SerializeTupleStruct = SeqSerializer<'a, W>;
    type SerializeTupleVariant = SeqSerializer<'a, W>;
    type SerializeMap = SeqSerializer<'a, W>;
    type SerializeStruct = SeqSerializer<'a, W>;
    type SerializeStructVariant = SeqSerializer<'a, W>;

    fn is_human_readable(&self) -> bool {
        false
    }

    fn serialize_bool(self, v: bool) -> Result<Self::Ok, W::Error> {
        let byte: u8 = v.into();
        let writted_bytes = self.writer.write_byte(byte)?;
        Ok(writted_bytes)
    }

    implement_number!(serialize_i8, i8);
    implement_number!(serialize_i16, i16);
    implement_number!(serialize_i32, i32);
    implement_number!(serialize_i64, i64);
    implement_number!(serialize_u8, u8);
    implement_number!(serialize_u16, u16);
    implement_number!(serialize_u32, u32);
    implement_number!(serialize_u64, u64);
    implement_number!(serialize_f32, f32);
    implement_number!(serialize_f64, f64);

    serde_if_integer128! {
        implement_number!(serialize_i128, i128);
        implement_number!(serialize_u128, u128);
    }

    fn serialize_char(self, v: char) -> Result<Self::Ok, W::Error> {
        let bytes: u32 = v.into();
        self.writer
            .write_bytes(&bytes.to_be_bytes())
            .map_err(Error::WriterError)
    }

    fn serialize_str(self, v: &str) -> Result<Self::Ok, W::Error> {
        Self::serialize_bytes(self, v.as_bytes())
    }

    fn serialize_bytes(self, v: &[u8]) -> Result<Self::Ok, W::Error> {
        let len = v.len() as u64;
        let writted_bytes = self.writer.write_bytes(&len.to_be_bytes())?;
        self.writer
            .write_bytes(v)
            .map(|wb| wb + writted_bytes)
            .map_err(Error::WriterError)
    }

    fn serialize_unit(self) -> Result<Self::Ok, W::Error> {
        Ok(0)
    }

    fn serialize_unit_struct(self, _name: &'static str) -> Result<Self::Ok, W::Error> {
        Self::serialize_unit(self)
    }

    fn serialize_unit_variant(
        self,
        _name: &'static str,
        variant_index: u32,
        _variant: &'static str,
    ) -> Result<Self::Ok, W::Error> {
        Self::serialize_u32(self, variant_index)
    }

    fn serialize_newtype_struct<T: ?Sized>(
        self,
        _name: &'static str,
        value: &T,
    ) -> Result<Self::Ok, W::Error>
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
    ) -> Result<Self::Ok, W::Error>
    where
        T: Serialize,
    {
        let written_bytes = self.writer.write_bytes(&variant_index.to_be_bytes())?;
        value.serialize(self).map(|wb| wb + written_bytes)
    }

    fn serialize_seq(self, len: Option<usize>) -> Result<Self::SerializeSeq, W::Error> {
        match len {
            Some(len) => {
                let len: u64 = len as u64;
                let written_bytes = self.writer.write_bytes(&len.to_be_bytes())?;
                Ok(SeqSerializer::new_known(self, written_bytes))
            }
            None => SeqSerializer::new_unknown(self),
        }
    }

    fn serialize_none(self) -> Result<Self::Ok, W::Error> {
        self.writer.write_byte(0).map_err(Error::WriterError)
    }

    fn serialize_some<T: ?Sized>(self, value: &T) -> Result<Self::Ok, W::Error>
    where
        T: Serialize,
    {
        let written_bytes = self.writer.write_byte(1)?;
        value.serialize(self).map(|wb| wb + written_bytes)
    }

    fn serialize_tuple(self, _len: usize) -> Result<Self::SerializeTuple, W::Error> {
        Ok(SeqSerializer::new_known(self, 0))
    }

    fn serialize_tuple_struct(
        self,
        _name: &'static str,
        _len: usize,
    ) -> Result<Self::SerializeTupleStruct, W::Error> {
        Ok(SeqSerializer::new_known(self, 0))
    }

    fn serialize_tuple_variant(
        self,
        _name: &'static str,
        variant_index: u32,
        _variant: &'static str,
        _len: usize,
    ) -> Result<Self::SerializeTupleVariant, W::Error> {
        let written_bytes = self.writer.write_bytes(&variant_index.to_be_bytes())?;
        Ok(SeqSerializer::new_known(self, written_bytes))
    }

    fn serialize_map(self, len: Option<usize>) -> Result<Self::SerializeMap, W::Error> {
        match len {
            Some(len) => {
                let len: u64 = len as u64;
                let written_bytes = self.writer.write_bytes(&len.to_be_bytes())?;
                Ok(SeqSerializer::new_known(self, written_bytes))
            }
            None => SeqSerializer::new_unknown(self),
        }
    }

    fn serialize_struct(
        self,
        _name: &'static str,
        _len: usize,
    ) -> Result<Self::SerializeStruct, W::Error> {
        Ok(SeqSerializer::new_known(self, 0))
    }

    fn serialize_struct_variant(
        self,
        _name: &'static str,
        variant_index: u32,
        _variant: &'static str,
        _len: usize,
    ) -> Result<Self::SerializeStructVariant, W::Error> {
        let written_bytes = self.writer.write_bytes(&variant_index.to_be_bytes())?;
        Ok(SeqSerializer::new_known(self, written_bytes))
    }

    fn collect_str<T: ?Sized>(self, value: &T) -> Result<Self::Ok, W::Error>
    where
        T: fmt::Display,
    {
        // unknown str length marker
        let mut written_bytes = self.writer.write_bytes(&u64::MAX.to_be_bytes())?;
        let mut collector = StrCollector::new(&mut self.writer);
        fmt::write(&mut collector, format_args!("{}", value))?;
        written_bytes += collector.written_bytes;
        // "null" terminated str
        written_bytes += self.writer.write_byte(u8::MAX)?;
        Ok(written_bytes)
    }
}

#[cfg(all(feature = "alloc", not(feature = "no-unsized-seq")))]
pub enum SeqSerializer<'a, W> {
    KnownSize {
        serializer: &'a mut Serializer<W>,
        written_bytes: usize,
    },
    UnknownSize {
        serializer: &'a mut Serializer<W>,
        count: u64,
        bytes: Vec<u8>,
    },
}

#[cfg(any(not(feature = "alloc"), feature = "no-unsized-seq"))]
pub struct SeqSerializer<'a, W> {
    serializer: &'a mut Serializer<W>,
    written_bytes: usize,
}

#[cfg(all(feature = "alloc", not(feature = "no-unsized-seq")))]
impl<'a, W: Write> SeqSerializer<'a, W> {
    pub fn new_known(serializer: &'a mut Serializer<W>, written_bytes: usize) -> Self {
        Self::KnownSize {
            serializer,
            written_bytes,
        }
    }

    pub fn new_unknown(serializer: &'a mut Serializer<W>) -> Result<Self, W::Error> {
        Ok(Self::UnknownSize {
            count: 0,
            bytes: Vec::new(),
            serializer,
        })
    }

    pub fn ser_value<T: ?Sized>(&mut self, value: &T) -> Result<(), W::Error>
    where
        T: Serialize,
    {
        match self {
            SeqSerializer::KnownSize {
                serializer,
                written_bytes,
            } => {
                *written_bytes += value.serialize(&mut **serializer)?;
                Ok(())
            }
            SeqSerializer::UnknownSize { count, bytes, .. } => {
                let mut serializer = Serializer { writer: bytes };
                *count += 1;
                value
                    .serialize(&mut serializer)
                    .map_err(Error::unwrap_writer_error)?;
                Ok(())
            }
        }
    }

    pub fn finish(self) -> Result<usize, W::Error> {
        match self {
            SeqSerializer::KnownSize { written_bytes, .. } => Ok(written_bytes),
            SeqSerializer::UnknownSize {
                count,
                bytes,
                serializer,
            } => {
                let written_bytes = serializer.writer.write_bytes(&count.to_be_bytes())?;
                serializer
                    .writer
                    .write_bytes(&bytes)
                    .map(|wb| wb + written_bytes)
                    .map_err(Error::WriterError)
            }
        }
    }
}

#[cfg(any(not(feature = "alloc"), feature = "no-unsized-seq"))]
impl<'a, W: Write> SeqSerializer<'a, W> {
    pub fn new_known(serializer: &'a mut Serializer<W>, written_bytes: usize) -> Self {
        Self {
            serializer,
            written_bytes,
        }
    }

    pub fn new_unknown(_serializer: &'a mut Serializer<W>) -> Result<Self, W::Error> {
        Err(Error::UnknownSeqLength)
    }

    pub fn ser_value<T: ?Sized>(&mut self, value: &T) -> Result<(), W::Error>
    where
        T: Serialize,
    {
        self.written_bytes += value.serialize(&mut *self.serializer)?;
        Ok(())
    }

    pub fn finish(self) -> Result<usize, W::Error> {
        Ok(self.written_bytes)
    }
}

impl<'a, W: Write> ser::SerializeSeq for SeqSerializer<'a, W> {
    type Ok = usize;

    type Error = Error<W::Error>;

    fn serialize_element<T: ?Sized>(&mut self, value: &T) -> Result<(), W::Error>
    where
        T: Serialize,
    {
        self.ser_value(value)
    }

    fn end(self) -> Result<Self::Ok, W::Error> {
        self.finish()
    }
}

impl<'a, W: Write> ser::SerializeTuple for SeqSerializer<'a, W> {
    type Ok = usize;

    type Error = Error<W::Error>;

    fn serialize_element<T: ?Sized>(&mut self, value: &T) -> Result<(), W::Error>
    where
        T: Serialize,
    {
        self.ser_value(value)
    }

    fn end(self) -> Result<Self::Ok, W::Error> {
        self.finish()
    }
}

impl<'a, W: Write> ser::SerializeTupleStruct for SeqSerializer<'a, W> {
    type Ok = usize;

    type Error = Error<W::Error>;

    fn serialize_field<T: ?Sized>(&mut self, value: &T) -> Result<(), W::Error>
    where
        T: Serialize,
    {
        self.ser_value(value)
    }

    fn end(self) -> Result<Self::Ok, W::Error> {
        self.finish()
    }
}

impl<'a, W: Write> ser::SerializeTupleVariant for SeqSerializer<'a, W> {
    type Ok = usize;

    type Error = Error<W::Error>;

    fn serialize_field<T: ?Sized>(&mut self, value: &T) -> Result<(), W::Error>
    where
        T: Serialize,
    {
        self.ser_value(value)
    }

    fn end(self) -> Result<Self::Ok, W::Error> {
        self.finish()
    }
}

impl<'a, W: Write> ser::SerializeMap for SeqSerializer<'a, W> {
    type Ok = usize;

    type Error = Error<W::Error>;

    fn serialize_key<T: ?Sized>(&mut self, key: &T) -> Result<(), W::Error>
    where
        T: Serialize,
    {
        self.ser_value(key)
    }

    fn serialize_value<T: ?Sized>(&mut self, value: &T) -> Result<(), W::Error>
    where
        T: Serialize,
    {
        self.ser_value(value)
    }

    fn end(self) -> Result<Self::Ok, W::Error> {
        self.finish()
    }
}

impl<'a, W: Write> ser::SerializeStruct for SeqSerializer<'a, W> {
    type Ok = usize;

    type Error = Error<W::Error>;

    fn serialize_field<T: ?Sized>(&mut self, _key: &'static str, value: &T) -> Result<(), W::Error>
    where
        T: Serialize,
    {
        self.ser_value(value)
    }

    fn end(self) -> Result<Self::Ok, W::Error> {
        self.finish()
    }
}

impl<'a, W: Write> ser::SerializeStructVariant for SeqSerializer<'a, W> {
    type Ok = usize;

    type Error = Error<W::Error>;

    fn serialize_field<T: ?Sized>(&mut self, _key: &'static str, value: &T) -> Result<(), W::Error>
    where
        T: Serialize,
    {
        self.ser_value(value)
    }

    fn end(self) -> Result<Self::Ok, W::Error> {
        self.finish()
    }
}

struct StrCollector<'a, W> {
    writer: &'a mut W,
    written_bytes: usize,
}

impl<'a, W: Write> StrCollector<'a, W> {
    pub fn new(writer: &'a mut W) -> Self {
        StrCollector {
            writer,
            written_bytes: 0,
        }
    }
}

impl<'a, W: Write> fmt::Write for StrCollector<'a, W> {
    fn write_str(&mut self, s: &str) -> fmt::Result {
        match self.writer.write_bytes(s.as_bytes()) {
            Ok(written_bytes) => {
                self.written_bytes += written_bytes;
                Ok(())
            }
            Err(_) => Err(fmt::Error),
        }
    }
}
