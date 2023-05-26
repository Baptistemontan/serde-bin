use core::fmt::{Display, Debug};

#[cfg(feature = "alloc")]
extern crate alloc;
#[cfg(all(feature = "alloc", not(feature = "std")))]
use alloc::vec::Vec;

#[cfg(all(feature = "alloc", not(feature = "std")))]
use crate::error::NoWriterError;

#[cfg(feature ="std")]
use std::io;


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