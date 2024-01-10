use thiserror::Error;
use crate::schema::types::jsonwebkey::JsonWebKeyError;

use crate::schema::types::multibase::MultibaseWrapperError;

#[derive(Debug, Error)]
pub enum KeyDecodingError {
    #[error("Json decoding error: ${0}")]
    JsonError(serde_json::Error),
    #[error("Pem decoding error: ${0}")]
    PemError(pem::PemError),
    #[error("Unsupported key error: ${0}")]
    UnsupportedPublicKeyField(&'static str),
    #[error("Base 58 decoding error: ${0}")]
    Base58DecodeError(bs58::decode::Error),
    #[error("Base 64 decoding error: ${0}")]
    Base64DecodeError(base64::DecodeError),
    #[error("Hex decoding error: ${0}")]
    HexDecodeError(hex::FromHexError),
    #[error("Jwk decoding error: ${0}")]
    JwkDecodeError(JsonWebKeyError),
    #[error("Multibase decoding error ${0}")]
    MultibaseError(MultibaseWrapperError),
}

impl From<pem::PemError> for KeyDecodingError {
    fn from(error: pem::PemError) -> Self {
        KeyDecodingError::PemError(error)
    }
}

impl From<bs58::decode::Error> for KeyDecodingError {
    fn from(error: bs58::decode::Error) -> Self {
        KeyDecodingError::Base58DecodeError(error)
    }
}

impl From<base64::DecodeError> for KeyDecodingError {
    fn from(error: base64::DecodeError) -> Self {
        KeyDecodingError::Base64DecodeError(error)
    }
}

impl From<hex::FromHexError> for KeyDecodingError {
    fn from(error: hex::FromHexError) -> Self {
        KeyDecodingError::HexDecodeError(error)
    }
}
