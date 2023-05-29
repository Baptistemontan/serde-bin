use core::fmt::{Debug, Display};
use core::ops::{Deref, DerefMut};

#[cfg(feature = "alloc")]
extern crate alloc;
#[cfg(all(feature = "alloc", not(feature = "std")))]
use alloc::vec::Vec;

#[cfg(feature = "std")]
use std::io;

use crate::error::{NoWriterError, WriterError};

pub trait Write {
    type Error: WriterError;

    fn write_bytes(&mut self, bytes: &[u8]) -> Result<usize, Self::Error>;

    fn write_byte(&mut self, byte: u8) -> Result<usize, Self::Error> {
        self.write_bytes(core::slice::from_ref(&byte))
    }
}

#[cfg(all(feature = "alloc", not(feature = "std")))]
impl<'a> Write for &'a mut Vec<u8> {
    type Error = NoWriterError;

    fn write_byte(&mut self, byte: u8) -> Result<usize, Self::Error> {
        self.push(byte);
        Ok(1)
    }

    fn write_bytes(&mut self, bytes: &[u8]) -> Result<usize, Self::Error> {
        self.extend_from_slice(bytes);
        Ok(bytes.len())
    }
}

#[cfg(feature = "std")]
impl<W: io::Write> Write for W {
    type Error = io::Error;

    fn write_bytes(&mut self, bytes: &[u8]) -> Result<usize, Self::Error> {
        self.write_all(bytes)?;
        Ok(bytes.len())
    }
}

pub struct BuffWriter<'a> {
    buff: &'a mut [u8],
    head: usize,
}

impl<'a> BuffWriter<'a> {
    pub fn new(buff: &'a mut [u8]) -> Self {
        BuffWriter { buff, head: 0 }
    }

    pub fn unwrap(self) -> (usize, &'a mut [u8]) {
        (self.head, self.buff)
    }

    pub fn len(&self) -> usize {
        self.head
    }

    pub fn is_empty(&self) -> bool {
        self.head == 0
    }

    pub fn get(&self) -> &[u8] {
        &self.buff[..self.head]
    }

    pub fn get_mut(&mut self) -> &mut [u8] {
        &mut self.buff[..self.head]
    }
}

impl<'a> Deref for BuffWriter<'a> {
    type Target = [u8];

    fn deref(&self) -> &Self::Target {
        self.get()
    }
}

impl<'a> DerefMut for BuffWriter<'a> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        self.get_mut()
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct EndOfBuff;

impl WriterError for EndOfBuff {}

impl Display for EndOfBuff {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str("Reached end of buffer before end of serialization.")
    }
}

impl<'a, 'b> Write for &'a mut BuffWriter<'b> {
    type Error = EndOfBuff;

    fn write_bytes(&mut self, bytes: &[u8]) -> Result<usize, Self::Error> {
        let spot = self
            .buff
            .get_mut(self.head..self.head + bytes.len())
            .ok_or(EndOfBuff)?;
        spot.copy_from_slice(bytes);
        Ok(bytes.len())
    }
}

pub struct DummyWriter;

impl Write for DummyWriter {
    type Error = NoWriterError;

    fn write_bytes(&mut self, bytes: &[u8]) -> Result<usize, Self::Error> {
        Ok(bytes.len())
    }
}
