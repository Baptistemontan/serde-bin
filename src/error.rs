use serde::{de, ser};
use std::{error, fmt::Display, io, str::Utf8Error};

pub type Result<T> = std::result::Result<T, Error>;

#[derive(Debug)]
pub enum Error {
    IOError(io::Error),
    Message(String),
    Eof,
    InvalidBool(u8),
    InvalidChar(u32),
    InvalidStr(Utf8Error),
    InvalidSize,
    InvalidOptionTag(u8),
    TrailingBytes(usize),
    Unimplemented(&'static str),
}

impl Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Error::IOError(io_err) => io_err.fmt(f),
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
                "Use of an unemplemented Deserializer function: {}",
                function_name
            )),
        }
    }
}

impl error::Error for Error {}

impl ser::Error for Error {
    fn custom<T>(msg: T) -> Self
    where
        T: Display,
    {
        Error::Message(msg.to_string())
    }
}

impl de::Error for Error {
    fn custom<T>(msg: T) -> Self
    where
        T: Display,
    {
        Error::Message(msg.to_string())
    }
}

impl From<io::Error> for Error {
    fn from(value: io::Error) -> Self {
        Error::IOError(value)
    }
}

impl From<Utf8Error> for Error {
    fn from(value: Utf8Error) -> Self {
        Error::InvalidStr(value)
    }
}
