use std::{
    fmt::{self, Display, Formatter},
    str::FromStr,
};

use serde::{Deserialize, Serialize};
use serde_json::Value;

use crate::error::DIDDocumentBuilderError;

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
// TODO: Introduce custom type instead of Value
// Unfortunately only supports curves from the original RFC
// pub struct JsonWebKey(jsonwebkey::JsonWebKey);
pub struct JsonWebKey(Value);

impl JsonWebKey {
    pub fn new(jwk: &str) -> Result<Self, DIDDocumentBuilderError> {
        if is_valid_jwk(jwk) {
            Ok(Self(Value::from_str(jwk)?))
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
        Self::new(s)
    }
}

impl Display for JsonWebKey {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        write!(f, "{}", serde_json::to_string(&self.0).unwrap())
    }
}

fn is_valid_jwk(jwk: &str) -> bool {
    // TODO: Validate properly
    // TODO: Use is_ok_and once stabilized
    match serde_json::from_str::<Value>(jwk) {
        Ok(jwk) => {
            jwk.is_object()
                && jwk.get("kty").is_some()
                && jwk.get("crv").is_some()
                && jwk.get("x").is_some()
        }
        Err(_) => false,
    }
}
