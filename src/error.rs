use failure::{Backtrace, Context, Fail};
use std::fmt;
use std::io::ErrorKind as IoError;

#[derive(Debug)]
pub struct Error {
    inner: Context<ErrorKind>,
}

impl Error {
    pub fn kind(&self) -> &ErrorKind {
        self.inner.get_context()
    }
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        fmt::Display::fmt(&self.inner, f)
    }
}

impl Fail for Error {
    fn cause(&self) -> Option<&Fail> {
        self.inner.cause()
    }

    fn backtrace(&self) -> Option<&Backtrace> {
        self.inner.backtrace()
    }
}

impl From<ErrorKind> for Error {
    fn from(err: ErrorKind) -> Error {
        Error {
            inner: Context::new(err),
        }
    }
}

impl From<Context<ErrorKind>> for Error {
    fn from(ctx: Context<ErrorKind>) -> Error {
        Error { inner: ctx }
    }
}

#[derive(Fail, Debug)]
pub enum ErrorKind {
    #[fail(display = "Bad Data Structure Code: {}", _0)]
    BadDataStructureCode(String),
    #[fail(display = "Bad Data Type Code: {}", _0)]
    BadDataTypeCode(String),
    #[fail(display = "Bad Directory Data")]
    BadDirectoryData,
    #[fail(display = "Bad Truncated Escape Sequence: '{}'", _0)]
    BadTruncEscSeq(String),
    #[fail(display = "Bad Field Control")]
    BadFieldControl,
    #[fail(display = "Could Not Parse The Catalog File")]
    CouldNotParseCatalog,
    #[fail(display = "Could Not Parse Name")]
    CouldNotParseName,
    #[fail(display = "Empty Format Controls")]
    EmptyFormatControls,
    #[fail(display = "The Data Descriptive Record is not correct.")]
    InvalidDDR,
    #[fail(display = "A Data Record is not correct.")]
    InvalidDR,
    #[fail(display = "The Leader is not correct.")]
    InvalidLeader,
    #[fail(display = "Invalid Field with name: '{}'", _0)]
    InvalidDDF(String),
    #[fail(display = "A Data Descriptive Field is not correct")]
    InvalidDDFS,
    #[fail(display = "Invalid Header")]
    InvalidHeader,
    #[fail(display = "EOF")]
    EOF,
    #[fail(display = "IOError: {:?}", _0)]
    IOError(IoError),
    #[fail(display = "Could not parse '{}' as integer.", _1)]
    ParseIntError(#[cause] std::num::ParseIntError, String),
    #[fail(display = "Could not parse '{}' as float.", _1)]
    ParseFloatError(#[cause] std::num::ParseFloatError, String),
    #[fail(display = "Can not parse Format Control '{}'", _0)]
    UnParsableFormatControl(String),
    #[fail(display = "UtfError")]
    UtfError(#[cause] std::str::Utf8Error),
    #[doc(hidden)]
    #[fail(display = "")]
    __Nonexhaustive,
}
