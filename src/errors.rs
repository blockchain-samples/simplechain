use std::io::Error as StdError;
use std::boxed::Box;
use reqwest::Error as ReqwestError;
use rusqlite::Error as RusqliteError;
use hex::FromHexError;
use base58::FromBase58Error;
use bincode::ErrorKind as BincodeError;
use secp256k1::Error as Secp256k1Error;
use r2d2::InitializationError as R2d2InitializationError;
use postgres::Error as PostgresError;

use rouille::input::json::JsonError;

#[derive(Debug)]
pub enum CoreError {
    // FIXME change those generic names to more specific errors
    IoError,
    HttpError,
    DatabaseError,
    SerializeError,
    CryptoError,
    WalletError
}

impl From<StdError> for CoreError {
    fn from(_: StdError) -> CoreError {
        CoreError::IoError
    }
}

impl From<ReqwestError> for CoreError {
    fn from(_: ReqwestError) -> CoreError {
        CoreError::HttpError
    }
}

impl From<RusqliteError> for CoreError {
    fn from(_: RusqliteError) -> CoreError {
        CoreError::DatabaseError
    }
}

impl From<FromHexError> for CoreError {
    fn from(_: FromHexError) -> CoreError {
        CoreError::SerializeError
    }
}

impl From<FromBase58Error> for CoreError {
    fn from(_: FromBase58Error) -> CoreError {
        CoreError::SerializeError
    }
}

impl From<Box<BincodeError>> for CoreError {
    fn from(_: Box<BincodeError>) -> CoreError {
        CoreError::SerializeError
    }
}

impl From<Secp256k1Error> for CoreError {
    fn from(_: Secp256k1Error) -> CoreError {
        CoreError::CryptoError
    }
}

impl From<R2d2InitializationError> for CoreError {
    fn from(_: R2d2InitializationError) -> CoreError {
        CoreError::DatabaseError
    }
}

impl From<PostgresError> for CoreError {
    fn from(_: PostgresError) -> CoreError {
        CoreError::DatabaseError
    }
}

#[derive(Debug)]
pub enum ServerError {
    CoreError,
    BodyParseError,
    SerializeError,

    NotFound,
    InvalidTransaction,
    InvalidBlock
}

impl From<CoreError> for ServerError {
    fn from(_: CoreError) -> ServerError {
        ServerError::CoreError
    }
}

impl From<FromBase58Error> for ServerError {
    fn from(_: FromBase58Error) -> ServerError {
        ServerError::SerializeError
    }
}

impl From<FromHexError> for ServerError {
    fn from(_: FromHexError) -> ServerError {
        ServerError::SerializeError
    }
}

impl From<JsonError> for ServerError {
    fn from(_: JsonError) -> ServerError {
        ServerError::BodyParseError
    }
}
