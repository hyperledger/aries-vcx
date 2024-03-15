use std::fmt::Display;

use did_key::DidKey;
use did_parser_nom::DidUrl;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
#[serde(untagged)]
pub enum ServiceKeyKind {
    DidKey(DidKey),
    Reference(DidUrl),
    Value(String),
}

impl Display for ServiceKeyKind {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ServiceKeyKind::Reference(did_url) => write!(f, "{}", did_url),
            ServiceKeyKind::Value(value) => write!(f, "{}", value),
            ServiceKeyKind::DidKey(did_key) => write!(f, "{}", did_key),
        }
    }
}
