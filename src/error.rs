use std::io;
use thiserror::Error as ThisError;

#[derive(Debug, ThisError)]
pub enum Error {
    #[error("invalid parameter")]
    InvalidParameter,
    #[error("unexpected metadata")]
    UnexpectedMetadata,
    #[error("interface not found")]
    InterfaceNotFound,
    #[error("unknown internal error")]
    InternalError,
    #[error("I/O error: {0}")]
    Io(io::Error),
}

#[cfg(not(target_os = "windows"))]
impl From<nix::Error> for Error {
    fn from(_: nix::Error) -> Self {
        Error::InternalError
    }
}

impl From<io::Error> for Error {
    fn from(e: io::Error) -> Self {
        Error::Io(e)
    }
}
