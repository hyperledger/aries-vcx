use std::{
    collections::HashMap,
    fmt::{self, Display, Formatter},
    str::FromStr,
};

use serde::{Deserialize, Serialize};
use serde_json::Value;

use crate::error::DidDocumentBuilderError;

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
// TODO: Introduce proper custom type
// Unfortunately only supports curves from the original RFC
// pub struct JsonWebKey(jsonwebkey::JsonWebKey);
pub struct JsonWebKey {
    kty: String,
    crv: String,
    x: String,
    #[serde(flatten)]
    #[serde(skip_serializing_if = "HashMap::is_empty")]
    #[serde(default)]
    extra: HashMap<String, Value>,
}

impl JsonWebKey {
    pub fn new(jwk: &str) -> Result<Self, DidDocumentBuilderError> {
        Ok(serde_json::from_str(jwk)?)
    }
}

impl FromStr for JsonWebKey {
    type Err = DidDocumentBuilderError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Self::new(s)
    }
}

impl Display for JsonWebKey {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        write!(f, "{}", serde_json::to_string(&self).unwrap())
    }
}
