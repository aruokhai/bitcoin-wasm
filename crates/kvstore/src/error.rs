use crate::bindings::exports::component::kvstore::types::{Error as GuestError};


#[derive(Debug)]
pub enum Error {
    KeyNotFound,
    KeyAlreadyExists,
    UnexpectedError,
    KeyOverflowError,
    ValueOverflowError,
    TryFromSliceError,
    UTF8Error,
    FilesystemError(u8),
    InvalidMagicBytes,
    StreamError
}

impl std::convert::From<std::io::Error> for Error {
    fn from(_e: std::io::Error) -> Error {
        Error::UnexpectedError
    }
}

impl Into<GuestError> for Error {
    fn into(self) -> GuestError {
        match self {
            Error::KeyNotFound => GuestError::KeyNotFound,
            Error::KeyAlreadyExists => GuestError::KeyAlreadyExists,
            Error::UnexpectedError => GuestError::UnexpectedError,
            Error::KeyOverflowError => GuestError::KeyOverflowError,
            Error::ValueOverflowError => GuestError::ValueOverflowError,
            Error::TryFromSliceError => GuestError::TryFromSliceError,
            Error::UTF8Error => GuestError::Utf8Error,
            Error::FilesystemError(error_code) =>  GuestError::FilesystemError(error_code),
            Error::InvalidMagicBytes => GuestError::InvalidMagicBytes,
            Error::StreamError => GuestError::StreamError,
        }
    }
}
