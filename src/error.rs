// #[cfg(feature = "std")]
use core::{
    fmt::{Debug, Display},
    str::Utf8Error,
};
use serde::{de, ser};
#[cfg(feature = "std")]
use std::error;
#[cfg(feature = "alloc")]
extern crate alloc;
#[cfg(feature = "alloc")]
use alloc::string::{String, ToString};

pub type Result<T, We = NoWriterError> = core::result::Result<T, Error<We>>;

#[derive(Debug)]
pub enum NoWriterError {}

impl Display for NoWriterError {
    fn fmt(&self, _f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        unreachable!()
    }
}

#[derive(Debug)]
pub enum Error<T: Debug> {
    WriterError(T),
    #[cfg(feature = "alloc")]
    Message(String),
    #[cfg(not(feature = "alloc"))]
    UnknownSeqLength,
    Eof,
    InvalidBool(u8),
    InvalidChar(u32),
    InvalidStr(Utf8Error),
    InvalidSize,
    InvalidOptionTag(u8),
    TrailingBytes(usize),
    Unimplemented(&'static str),
}

impl<T: Display + Debug> Display for Error<T> {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        match self {
            Error::WriterError(w_err) => Display::fmt(w_err, f),
            #[cfg(feature = "alloc")]
            Error::Message(msg) => f.write_str(msg),
            Error::Eof => f.write_str("Reached EOF before end of deserialization"),
            Error::InvalidBool(byte) => f.write_fmt(format_args!(
                "Error deserializing bool: Expecting 0 or 1, found {}",
                byte
            )),
            Error::InvalidChar(c) => f.write_fmt(format_args!(
                "Error deserializing char: Expected valid UTF-8 char, found {}",
                c
            )),
            Error::InvalidStr(error) => {
                f.write_fmt(format_args!("Error deserializing str: {}", error))
            }
            Error::InvalidSize => f.write_fmt(format_args!("Error deserializing sequence length")),
            Error::InvalidOptionTag(byte) => f.write_fmt(format_args!(
                "Error deserializing option: Expected tag with value 0 or 1, found {}",
                byte
            )),
            Error::TrailingBytes(remaining) => f.write_fmt(format_args!(
                "Reached end of deserialization but {} bytes are remaining",
                remaining
            )),
            Error::Unimplemented(function_name) => f.write_fmt(format_args!(
                "Use of an unimplemented Deserializer function: {}",
                function_name
            )),
            #[cfg(not(feature = "alloc"))]
            Error::UnknownSeqLength => f.write_str(
                "Tried to serialize a sequence with an unknown length in a no alloc env.",
            ),
        }
    }
}

#[cfg(feature = "std")]
impl<We: Display + Debug> error::Error for Error<We> {}

// #[cfg(feature = "std")]
impl<We: Display + Debug> ser::Error for Error<We> {
    #[cfg(feature = "alloc")]
    fn custom<T>(msg: T) -> Self
    where
        T: Display,
    {
        Error::Message(msg.to_string())
    }

    #[cfg(not(feature = "alloc"))]
    fn custom<T>(_msg: T) -> Self
    where
        T: Display,
    {
        todo!()
    }
}

// #[cfg(feature = "std")]
impl<We: Display + Debug> de::Error for Error<We> {
    #[cfg(feature = "alloc")]
    fn custom<T>(msg: T) -> Self
    where
        T: Display,
    {
        Error::Message(msg.to_string())
    }

    #[cfg(not(feature = "alloc"))]
    fn custom<T>(_msg: T) -> Self
    where
        T: Display,
    {
        todo!()
    }
}

impl<We: Debug> From<Utf8Error> for Error<We> {
    fn from(value: Utf8Error) -> Self {
        Error::InvalidStr(value)
    }
}
