use core::fmt::{Debug, Display};

#[cfg(feature = "alloc")]
extern crate alloc;
#[cfg(all(feature = "alloc", not(feature = "std")))]
use alloc::vec::Vec;

#[cfg(feature = "std")]
use std::io;

#[cfg(all(feature = "alloc", not(feature = "std")))]
use crate::error::NoWriterError;

pub trait Write {
    type Error: Display + Debug;

    fn write_byte(&mut self, byte: u8) -> Result<(), Self::Error>;

    fn write_bytes(&mut self, bytes: &[u8]) -> Result<usize, Self::Error>;
}

#[cfg(all(feature = "alloc", not(feature = "std")))]
impl<'a> Write for &'a mut Vec<u8> {
    type Error = NoWriterError;

    fn write_byte(&mut self, byte: u8) -> Result<(), Self::Error> {
        self.push(byte);
        Ok(())
    }

    fn write_bytes(&mut self, bytes: &[u8]) -> Result<usize, Self::Error> {
        self.extend_from_slice(bytes);
        Ok(bytes.len())
    }
}

#[cfg(feature = "std")]
impl<W: io::Write> Write for W {
    type Error = io::Error;

    fn write_byte(&mut self, byte: u8) -> Result<(), Self::Error> {
        self.write_all(&[byte])?;
        Ok(())
    }

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
}

#[derive(Debug, Clone, Copy)]
pub struct EndOfBuff;

impl Display for EndOfBuff {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str("Reached end of buffer before end of serialization.")
    }
}

impl<'a> Write for BuffWriter<'a> {
    type Error = EndOfBuff;

    fn write_byte(&mut self, byte: u8) -> Result<(), Self::Error> {
        let spot = self.buff.get_mut(self.head).ok_or(EndOfBuff)?;
        *spot = byte;
        self.head += 1;
        Ok(())
    }

    fn write_bytes(&mut self, bytes: &[u8]) -> Result<usize, Self::Error> {
        let spot = self
            .buff
            .get_mut(self.head..self.head + bytes.len())
            .ok_or(EndOfBuff)?;
        spot.copy_from_slice(bytes);
        Ok(bytes.len())
    }
}
