use std::{
    collections::HashMap,
    error::Error,
    fmt::{self, Display, Formatter},
    str::FromStr,
};

use serde::{Deserialize, Serialize};
use serde_json::Value;
use thiserror::Error;

#[derive(Debug, Error)]
pub struct JsonWebKeyError {
    reason: &'static str,
    #[source]
    source: Box<dyn Error + Sync + Send>,
}

impl Display for JsonWebKeyError {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "JsonWebKeyError, reason: {}, source: {}",
            self.reason, self.source
        )
    }
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
// TODO: Introduce proper custom type
// Unfortunately only supports curves from the original RFC
// pub struct JsonWebKey(jsonwebkey::JsonWebKey);
pub struct JsonWebKey {
    pub kty: String,
    pub crv: String,
    pub x: String,
    #[serde(flatten)]
    #[serde(skip_serializing_if = "HashMap::is_empty")]
    #[serde(default)]
    pub extra: HashMap<String, Value>,
}

impl JsonWebKey {
    // todo: More future-proof way would be creating custom error type, but seems as overkill atm?
    pub fn new(jwk: &str) -> Result<Self, JsonWebKeyError> {
        serde_json::from_str(jwk).map_err(|err| JsonWebKeyError {
            reason: "Parsing JWK failed",
            source: Box::new(err),
        })
    }
}

impl FromStr for JsonWebKey {
    type Err = JsonWebKeyError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Self::new(s)
    }
}

impl Display for JsonWebKey {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        write!(f, "{}", serde_json::to_string(&self).unwrap())
    }
}
