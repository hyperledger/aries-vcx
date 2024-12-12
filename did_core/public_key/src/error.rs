use std::error::Error;

use thiserror::Error;

use crate::KeyType;

#[derive(Debug, Error)]
pub enum PublicKeyError {
    #[error("Base 64 decoding error")]
    Base64DecodingError(#[from] base64::DecodeError),
    #[error("Base 58 decoding error")]
    Base58DecodingError(#[from] bs58::decode::Error),
    #[error("Multibase decoding error")]
    MultibaseDecodingError(#[from] multibase::Error),
    #[error("Varint decoding error")]
    VarintDecodingError(#[from] VarintDecodingError),
    #[error("JWK decoding error")]
    JwkDecodingError(#[from] Box<dyn Error + Send + Sync>),
    #[error("Unsupported multicodec descriptor: {0}")]
    UnsupportedMulticodecDescriptor(u64),
    #[error("Unsupported multicodec descriptor: {0}")]
    UnsupportedKeyType(String),
    #[error("Invalid KeyType {0}, expected KeyType: {1}")]
    InvalidKeyType(KeyType, KeyType),
}

#[derive(Debug, Error)]
pub struct VarintDecodingError(unsigned_varint::decode::Error);

impl std::fmt::Display for VarintDecodingError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Varint decoding error: {}", self.0)
    }
}

impl From<unsigned_varint::decode::Error> for VarintDecodingError {
    fn from(error: unsigned_varint::decode::Error) -> Self {
        Self(error)
    }
}

impl From<unsigned_varint::decode::Error> for PublicKeyError {
    fn from(error: unsigned_varint::decode::Error) -> Self {
        Self::VarintDecodingError(error.into())
    }
}
