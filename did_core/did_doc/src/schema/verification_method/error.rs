use std::{
    error::Error,
    fmt,
    fmt::{Display, Formatter},
};

use thiserror::Error;

use crate::schema::types::{jsonwebkey::JsonWebKeyError, multibase::MultibaseWrapperError};

#[derive(Debug, Error)]
pub struct KeyDecodingError {
    reason: &'static str,
    #[source]
    source: Option<Box<dyn Error + Sync + Send>>,
}

impl KeyDecodingError {
    pub fn new(reason: &'static str) -> Self {
        KeyDecodingError {
            reason,
            source: None,
        }
    }
}

impl Display for KeyDecodingError {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        match &self.source {
            Some(source) => write!(
                f,
                "KeyDecodingError, reason: {}, source: {}",
                self.reason, source
            ),
            None => write!(f, "KeyDecodingError, reason: {}", self.reason),
        }
    }
}

impl From<pem::PemError> for KeyDecodingError {
    fn from(error: pem::PemError) -> Self {
        KeyDecodingError {
            reason: "Failed to decode PEM",
            source: Some(Box::new(error)),
        }
    }
}

impl From<bs58::decode::Error> for KeyDecodingError {
    fn from(error: bs58::decode::Error) -> Self {
        KeyDecodingError {
            reason: "Failed to decode base58",
            source: Some(Box::new(error)),
        }
    }
}

impl From<base64::DecodeError> for KeyDecodingError {
    fn from(error: base64::DecodeError) -> Self {
        KeyDecodingError {
            reason: "Failed to decode base64",
            source: Some(Box::new(error)),
        }
    }
}

impl From<hex::FromHexError> for KeyDecodingError {
    fn from(error: hex::FromHexError) -> Self {
        KeyDecodingError {
            reason: "Failed to decode hex value",
            source: Some(Box::new(error)),
        }
    }
}

impl From<MultibaseWrapperError> for KeyDecodingError {
    fn from(error: MultibaseWrapperError) -> Self {
        KeyDecodingError {
            reason: "Failed to decode multibase value",
            source: Some(Box::new(error)),
        }
    }
}

impl From<JsonWebKeyError> for KeyDecodingError {
    fn from(error: JsonWebKeyError) -> Self {
        KeyDecodingError {
            reason: "Failed to decode JWK",
            source: Some(Box::new(error)),
        }
    }
}

impl From<public_key::PublicKeyError> for KeyDecodingError {
    fn from(error: public_key::PublicKeyError) -> Self {
        KeyDecodingError {
            reason: "Failed to decode multibase public key",
            source: Some(Box::new(error)),
        }
    }
}
