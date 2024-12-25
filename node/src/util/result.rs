use hex::FromHexError;
use base58::FromBase58Error;
use wasi::io::streams::StreamError;
use wasi::sockets::tcp::ErrorCode;
use std::io;
use std::string::FromUtf8Error;
use crate::bindings;
use bindings::component::kv::types::Error as StoreError;

/// Standard error type used in the library
#[derive(Debug)]
pub enum Error {
    /// An argument provided is invalid
    DBError(StoreError),
    SerializationError(String),
    FetchCompactFilter(u32),
    FetchCompactFilterHeader(u32),
    FetchBlock(u32),
    FilterMatchEror,
    NetworkError,
    FetchHeader(u32),
    BadArgument(String),
    /// The data given is not valid
    BadData(String),
    /// Base58 string could not be decoded
    FromBase58Error(FromBase58Error),
    /// Hex string could not be decoded
    FromHexError(FromHexError),
    /// UTF8 parsing error
    FromUtf8Error(FromUtf8Error),
    /// The state is not valid
    IllegalState(String),
    /// The operation is not valid on this object
    InvalidOperation(String),
    /// Standard library IO error
    IOError(io::Error),
    /// Error parsing an integer
    ParseIntError(std::num::ParseIntError),
    /// Error evaluating the script
    ScriptError(String),
    /// Error in the Secp256k1 library
    Secp256k1Error(libsecp256k1::Error),
    /// The operation timed out
    Timeout,
    /// An unknown error in the Ring library
    UnspecifiedRingError,
    /// The data or functionality is not supported by this library
    Unsupported(String),
    /// P2P Streaming error
    StreamingError(StreamError),
    /// Wrong P2P Message
    WrongP2PMessage,
    /// TCP Error,
    TCPError(ErrorCode),
    /// Slice Error
    SliceError(String),
    /// Peer Not Found Error
    PeerNotFound
}

impl Error {
    pub fn to_error_code(&self) -> u32 {
        match self {
            Error::BadArgument(_) => 1,
            Error::BadData(_) => 2,
            Error::FromBase58Error(_) => 3,
            Error::FromHexError(_) => 4,
            Error::FromUtf8Error(_) => 5,
            Error::IllegalState(_) => 6,
            Error::InvalidOperation(_) => 7,
            Error::IOError(_) => 8,
            Error::ParseIntError(_) => 9,
            Error::ScriptError(_) => 10,
            Error::Secp256k1Error(_) => 11,
            Error::Timeout => 12,
            Error::UnspecifiedRingError => 13,
            Error::Unsupported(_) => 14,
            Error::StreamingError(_) => 15,
            Error::WrongP2PMessage => 16,
            Error::TCPError(_) => 17,
            Error::SliceError(_) => 18,
            Error::PeerNotFound => 19,
            Error::DBError(_) => 20,
            Error::SerializationError(_) => 21,
            Error::FetchCompactFilter(_) => 22,
            Error::FetchCompactFilterHeader(_) => 23,
            Error::FetchBlock(_) => 24,
            Error::FilterMatchEror => 25,
            Error::NetworkError => 26,
            Error::FetchHeader(_) => 27,
        }
    }
}

impl std::fmt::Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
            Error::BadArgument(s) => f.write_str(&format!("Bad argument: {}", s)),
            Error::BadData(s) => f.write_str(&format!("Bad data: {}", s)),
            Error::FromBase58Error(e) => f.write_str(&format!("Base58 decoding error: {}", "base58")),
            Error::FromHexError(e) => f.write_str(&format!("Hex decoding error: {}", e)),
            Error::FromUtf8Error(e) => f.write_str(&format!("Utf8 parsing error: {}", e)),
            Error::IllegalState(s) => f.write_str(&format!("Illegal state: {}", s)),
            Error::InvalidOperation(s) => f.write_str(&format!("Invalid operation: {}", s)),
            Error::IOError(e) => f.write_str(&format!("IO error: {}", e)),
            Error::ParseIntError(e) => f.write_str(&format!("ParseIntError: {}", e)),
            Error::ScriptError(s) => f.write_str(&format!("Script error: {}", s)),
            Error::Secp256k1Error(e) => f.write_str(&format!("Secp256k1 error: {}", e)),
            Error::Timeout => f.write_str("Timeout"),
            Error::WrongP2PMessage => f.write_str("Wrong P2P message gotten"),
            Error::UnspecifiedRingError => f.write_str("Unspecified ring error"),
            Error::StreamingError(s) => f.write_str(&format!("Srreaming Error: {}", s)),
            Error::Unsupported(s) => f.write_str(&format!("Unsuppored: {}", s)),
            Error::TCPError(c) => f.write_str(&format!("TCP socket Error: {}", c)),
            Error::SliceError(c) => f.write_str(&format!("Slice Error: {}", c)),
            Error::PeerNotFound => f.write_str("P2P peer not found"),
            Error::DBError(error) => f.write_str(&format!("DBError: {}", error)),
            Error::SerializationError(error) => f.write_str(&format!("Serialization: {}", error)),
            Error::FetchCompactFilter(e) => f.write_str(&format!("Fetching Compact Filter Error: {}", e)),
            Error::FetchCompactFilterHeader(e) => f.write_str(&format!("Fetching Compact Filter Header Error: {}", e)),
            Error::FetchBlock(e) => f.write_str(&format!("Fetching Block Error: {}", e)),
            Error::FilterMatchEror => f.write_str(&format!("Filter Match Error")),
            Error::NetworkError => f.write_str(&format!("Network Error")),
            Error::FetchHeader(e) => f.write_str(&format!("Fetching Header Error: {}", e)),

        }
    }
}

impl std::error::Error for Error {
    fn description(&self) -> &str {
        match self {
            Error::BadArgument(_) => "Bad argument",
            Error::BadData(_) => "Bad data",
            Error::FromBase58Error(_) => "Base58 decoding error",
            Error::FromHexError(_) => "Hex decoding error",
            Error::FromUtf8Error(_) => "Utf8 parsing error",
            Error::IllegalState(_) => "Illegal state",
            Error::InvalidOperation(_) => "Invalid operation",
            Error::IOError(_) => "IO error",
            Error::ParseIntError(_) => "Parse int error",
            Error::ScriptError(_) => "Script error",
            Error::Secp256k1Error(_) => "Secp256k1 error",
            Error::Timeout => "Timeout",
            Error::UnspecifiedRingError => "Unspecified ring error",
            Error::Unsupported(_) => "Unsupported",
            Error::StreamingError(_) => "P2P Streaming Error",
            Error::WrongP2PMessage => "Wrong P2P message gotten Error",
            Error::TCPError(_) => "TCP error",
            Error::SliceError(_) => "Slice error",
            Error::PeerNotFound => "P2P Peer Not Found",
            Error::DBError(_) => "DB Error",
            Error::SerializationError(_) => "Serialization Error",
            Error::FetchCompactFilter(_) => "Fetch Compact Filter Error",
            Error::FetchCompactFilterHeader(_) => "Fetch Compact Filter Header Error",
            Error::FetchBlock(_) => "Fetch Block Error",
            Error::FilterMatchEror => "Filter Match Error",
            Error::NetworkError => "Network Error",
            Error::FetchHeader(_) => "Fetch Header Error",
        }
    }

    // fn cause(&self) -> Option<&dyn std::error::Error> {
    //     match self {
    //         Error::FromHexError(e) => Some(e),
    //         Error::FromUtf8Error(e) => Some(e),
    //         Error::IOError(e) => Some(e),
    //         Error::ParseIntError(e) => Some(e),
    //         Error::Secp256k1Error(e) => Some("seckp1 error"),
    //         _ => None,
    //     }
    //}
}

impl From<FromBase58Error> for Error {
    fn from(e: FromBase58Error) -> Self {
        Error::FromBase58Error(e)
    }
}

impl From<FromHexError> for Error {
    fn from(e: FromHexError) -> Self {
        Error::FromHexError(e)
    }
}

impl From<FromUtf8Error> for Error {
    fn from(e: FromUtf8Error) -> Self {
        Error::FromUtf8Error(e)
    }
}

impl From<io::Error> for Error {
    fn from(e: io::Error) -> Self {
        Error::IOError(e)
    }
}

impl From<std::num::ParseIntError> for Error {
    fn from(e: std::num::ParseIntError) -> Self {
        Error::ParseIntError(e)
    }
}

impl From<libsecp256k1::Error> for Error {
    fn from(e: libsecp256k1::Error) -> Self {
        Error::Secp256k1Error(e)
    }
}

impl From<ring::error::Unspecified> for Error {
    fn from(_: ring::error::Unspecified) -> Self {
        Error::UnspecifiedRingError
    }
}

/// Standard Result used in the library
pub type Result<T> = std::result::Result<T, Error>;
