use std::io::Error as StdError;
use std::boxed::Box;
use reqwest::Error as ReqwestError;
use rusqlite::Error as RusqliteError;
use hex::FromHexError;
use base58::FromBase58Error;
use bincode::ErrorKind as BincodeError;
use secp256k1::Error as Secp256k1Error;

// TODO split in two error types: NetError and CoreError?

#[derive(Debug)]
pub enum ServerError {
    // FIXME change those generic names to more specific errors
    IoError,
    HttpError,
    DatabaseError,
    SerializeError,
    CryptoError
}

impl From<StdError> for ServerError {
    fn from(_: StdError) -> ServerError {
        ServerError::IoError
    }
}

impl From<ReqwestError> for ServerError {
    fn from(_: ReqwestError) -> ServerError {
        ServerError::HttpError
    }
}

impl From<RusqliteError> for ServerError {
    fn from(_: RusqliteError) -> ServerError {
        ServerError::DatabaseError
    }
}

impl From<FromHexError> for ServerError {
    fn from(_: FromHexError) -> ServerError {
        ServerError::SerializeError
    }
}

impl From<FromBase58Error> for ServerError {
    fn from(_: FromBase58Error) -> ServerError {
        ServerError::SerializeError
    }
}

impl From<Box<BincodeError>> for ServerError {
    fn from(_: Box<BincodeError>) -> ServerError {
        ServerError::SerializeError
    }
}

impl From<Secp256k1Error> for ServerError {
    fn from(_: Secp256k1Error) -> ServerError {
        ServerError::CryptoError
    }
}
