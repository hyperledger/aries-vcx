use std::{
    collections::HashMap,
    fmt::{self, Display, Formatter},
    str::FromStr,
};

use serde::{Deserialize, Serialize};
use serde_json::Value;

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
    // todo: More future-proof way would be creating custom error type, but seems as overkill atm?
    pub fn new(jwk: &str) -> Result<Self, serde_json::Error> {
        serde_json::from_str(jwk)
    }

    pub fn to_vec(&self) -> Result<Vec<u8>, serde_json::Error> {
        serde_json::to_vec(self)
    }
}

impl FromStr for JsonWebKey {
    type Err = serde_json::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Self::new(s)
    }
}

impl Display for JsonWebKey {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        write!(f, "{}", serde_json::to_string(&self).unwrap())
    }
}
