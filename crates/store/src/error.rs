use crate::bindings::exports::component::store::types::{Error as GuestError};


#[derive(Debug)]
pub enum Error {
    KeyNotFound,
    KeyAlreadyExists,
    UnexpectedError,
    KeyOverflowError,
    ValueOverflowError,
    TryFromSliceError(&'static str),
    UTF8Error,
    FilesystemError(String),
    InvalidMagicBytes,
}

impl std::convert::From<std::io::Error> for Error {
    fn from(_e: std::io::Error) -> Error {
        Error::UnexpectedError
    }
}

impl Into<GuestError> for Error {
    
    fn into(self) -> GuestError {
        return GuestError::Nae;
    }
}
