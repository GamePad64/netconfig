use std::error::Error as StdError;
use std::io;
use thiserror::Error as ThisError;
use widestring::error::Utf16Error;

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

#[cfg(not(target_os = "windows"))]
impl From<nix::Error> for Error {
    fn from(e: nix::Error) -> Self {
        Error::Unknown(Box::new(e))
    }
}

#[cfg(target_os = "windows")]
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

#[cfg(target_os = "windows")]
impl From<Utf16Error> for Error {
    fn from(_: Utf16Error) -> Self {
        Self::UnexpectedMetadata
    }
}
