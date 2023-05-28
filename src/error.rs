// #[cfg(feature = "std")]
use core::{
    fmt::{self, Debug, Display},
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

pub trait WriterError: Debug + Display {}

impl WriterError for NoWriterError {}

impl Display for NoWriterError {
    fn fmt(&self, _f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        unreachable!()
    }
}

#[cfg(not(feature = "alloc"))]
#[derive(Debug)]
pub enum ErrorKind {
    Serialization,
    Deserialization,
}

#[derive(Debug)]
pub enum Error<T: Debug> {
    WriterError(T),
    #[cfg(feature = "alloc")]
    Message(String),
    #[cfg(not(feature = "alloc"))]
    Custom(ErrorKind),
    #[cfg(any(not(feature = "alloc"), feature = "no-unsized-seq"))]
    UnknownSeqLength,
    Eof,
    InvalidBool(u8),
    InvalidChar(u32),
    InvalidStr(Utf8Error),
    InvalidSize,
    InvalidOptionTag(u8),
    TrailingBytes(usize),
    Unimplemented(&'static str),
    FormattingError,
}

impl<W: WriterError> Error<W> {
    pub fn map_writer_error<We, F>(self, map_fn: F) -> Error<We>
    where
        We: WriterError,
        F: FnOnce(W) -> We,
    {
        match self {
            Error::WriterError(err) => Error::WriterError(map_fn(err)),
            #[cfg(feature = "alloc")]
            Error::Message(x) => Error::Message(x),
            #[cfg(not(feature = "alloc"))]
            Error::Custom(kind) => Error::Custom(kind),
            #[cfg(any(not(feature = "alloc"), feature = "no-unsized-seq"))]
            Error::UnknownSeqLength => Error::UnknownSeqLength,
            Error::Eof => Error::Eof,
            Error::InvalidBool(x) => Error::InvalidBool(x),
            Error::InvalidChar(x) => Error::InvalidChar(x),
            Error::InvalidStr(x) => Error::InvalidStr(x),
            Error::InvalidSize => Error::InvalidSize,
            Error::InvalidOptionTag(x) => Error::InvalidOptionTag(x),
            Error::TrailingBytes(x) => Error::TrailingBytes(x),
            Error::Unimplemented(x) => Error::Unimplemented(x),
            Error::FormattingError => Error::FormattingError,
        }
    }

    pub fn unwrap_writer_error<We: WriterError>(self) -> Error<We> {
        self.map_writer_error(|err| panic!("{}", err))
    }
}

impl<T: Display + Debug> Display for Error<T> {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        match self {
            Error::WriterError(w_err) => Display::fmt(w_err, f),
            #[cfg(feature = "alloc")]
            Error::Message(msg) => f.write_str(msg),
            #[cfg(not(feature = "alloc"))]
            Error::Custom(ErrorKind::Serialization) => {
                f.write_str("An error occured during serialization.")
            }
            #[cfg(not(feature = "alloc"))]
            Error::Custom(ErrorKind::Deserialization) => {
                f.write_str("An error occured during deserialization.")
            }
            #[cfg(any(not(feature = "alloc"), feature = "no-unsized-seq"))]
            Error::UnknownSeqLength => f.write_str(
                "Tried to serialize a sequence with an unknown length in a no alloc env.",
            ),
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
            Error::FormattingError => f.write_str("An error occured while formatting a value."),
        }
    }
}

#[cfg(feature = "std")]
impl<We: Display + Debug> error::Error for Error<We> {}

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
        Error::Custom(ErrorKind::Serialization)
    }
}

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
        Error::Custom(ErrorKind::Deserialization)
    }
}

impl<We: Debug> From<Utf8Error> for Error<We> {
    fn from(value: Utf8Error) -> Self {
        Error::InvalidStr(value)
    }
}

impl<We: WriterError> From<We> for Error<We> {
    fn from(value: We) -> Self {
        Error::WriterError(value)
    }
}

impl<We: Debug> From<fmt::Error> for Error<We> {
    fn from(_value: fmt::Error) -> Self {
        Error::FormattingError
    }
}

#[cfg(feature = "std")]
impl WriterError for std::io::Error {}
