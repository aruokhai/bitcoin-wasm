use crate::bindings::exports::component::kv::types::Error as GuestError;


#[derive(Debug)]
pub enum Error {
    OpenFileError,
    StreamError,
    FileNotFound(u64),
    InvalidData,
    ParseError,
    EntryNotFound
}

impl From<Error> for GuestError {
    fn from(val: Error) -> Self {
        match val {
            Error::OpenFileError => GuestError::OpenFileError,
            Error::StreamError => GuestError::StreamError,
            Error::EntryNotFound => GuestError::EntryNotFound,
            Error::InvalidData => GuestError::InvalidData,
            Error::ParseError => GuestError::ParseError,
            Error::FileNotFound(error_code) => GuestError::FileNotFound(error_code),
        }
    }
}