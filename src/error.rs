//! The error type returned by the dialog functions.

use std::error::Error;
use std::ffi;
use std::fmt;
use std::str;

/// An error opening a dialog or reading its result.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum NFDError {
    /// An argument (filter, path, title, ...) contained an interior NUL byte.
    NulError(ffi::NulError),
    /// A path, either passed in or returned by the dialog, was not valid UTF-8.
    Utf8Error(str::Utf8Error),
    /// The native dialog reported an error. The message comes from the platform backend.
    Error(String),
}

impl fmt::Display for NFDError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            NFDError::NulError(_) => f.write_str("dialog argument contains a NUL byte"),
            NFDError::Utf8Error(_) => f.write_str("path is not valid UTF-8"),
            NFDError::Error(message) => f.write_str(message),
        }
    }
}

impl Error for NFDError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self {
            NFDError::NulError(err) => Some(err),
            NFDError::Utf8Error(err) => Some(err),
            NFDError::Error(_) => None,
        }
    }
}

impl From<ffi::NulError> for NFDError {
    fn from(err: ffi::NulError) -> NFDError {
        NFDError::NulError(err)
    }
}

impl From<str::Utf8Error> for NFDError {
    fn from(err: str::Utf8Error) -> NFDError {
        NFDError::Utf8Error(err)
    }
}
