use std::str::FromStr;

use serde::{Deserialize, Serialize};
use serde_json::Value;

use crate::error::DIDDocumentBuilderError;

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
// TODO: Introduce custom type instead of Value
// Unfortunately only supports curves from the original RFC
// pub struct JsonWebKey(jsonwebkey::JsonWebKey);
pub struct JsonWebKey(Value);

impl JsonWebKey {
    pub fn new(jwk: String) -> Result<Self, DIDDocumentBuilderError> {
        if is_valid_jwk(&jwk) {
            Ok(Self(Value::from_str(&jwk)?))
        } else {
            Err(DIDDocumentBuilderError::InvalidInput(format!(
                "Invalid JWK: {}",
                jwk
            )))
        }
    }
}

impl FromStr for JsonWebKey {
    type Err = DIDDocumentBuilderError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Self::new(s.to_string())
    }
}

impl ToString for JsonWebKey {
    fn to_string(&self) -> String {
        self.0.to_string()
    }
}

fn is_valid_jwk(_jwk: &str) -> bool {
    true
}
