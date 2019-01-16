use failure::{Backtrace, Context, Fail, ResultExt};
use std::fmt;
use std::io::ErrorKind as IoError;
use std::result;

#[derive(Debug)]
pub struct Error {
    inner: Context<ErrorKind>,
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
    #[fail(display = "Bad Data Structure Code")]
    BadDataStructureCode,
    #[fail(display = "Bad Data Type Code")]
    BadDataTypeCode,
    #[fail(display = "Bad Directory Data")]
    BadDirectoryData,
    #[fail(display = "Bad Truncated Escape Sequence")]
    BadTruncEscSeq,
    #[fail(display = "Empty Format Controls")]
    EmptyFormatControls,
    #[fail(display = "Invalid Header")]
    InvalidHeader,
    #[fail(display = "IOError")]
    IOError(IoError),
    #[fail(display = "ParseIntError")]
    ParseIntError(#[cause] std::num::ParseIntError),
    #[fail(display = "ParseFloatError")]
    ParseFloatError(#[cause] std::num::ParseFloatError),
    #[fail(display = "UnParsable: {}", _0)]
    UnParsable(String),
    #[fail(display = "UtfError")]
    UtfError(#[cause] std::str::Utf8Error),
    #[doc(hidden)]
    #[fail(display = "")]
    __Nonexhaustive,
}
