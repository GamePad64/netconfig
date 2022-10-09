use std::error::Error as StdError;
use std::io;
use thiserror::Error as ThisError;

#[non_exhaustive]
#[derive(Debug, ThisError)]
pub enum Error {
    #[error("invalid parameter")]
    InvalidParameter,
    #[error("unexpected metadata")]
    UnexpectedMetadata,
    #[error("interface not found")]
    InterfaceNotFound,
    #[error("unknown error: {0}")]
    Unknown(Box<dyn StdError>),
    #[error("I/O error: {0}")]
    Io(io::Error),
}

#[cfg(unix)]
impl From<nix::Error> for Error {
    fn from(e: nix::Error) -> Self {
        Error::Unknown(Box::new(e))
    }
}

#[cfg(windows)]
impl From<windows::core::Error> for Error {
    fn from(e: windows::core::Error) -> Self {
        Self::Unknown(Box::new(e))
    }
}

impl From<io::Error> for Error {
    fn from(e: io::Error) -> Self {
        Self::Io(e)
    }
}

#[cfg(windows)]
impl From<widestring::error::Utf16Error> for Error {
    fn from(_: Utf16Error) -> Self {
        Self::UnexpectedMetadata
    }
}
