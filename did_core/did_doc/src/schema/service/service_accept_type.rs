use std::fmt::Display;

use serde::{Deserialize, Deserializer, Serialize};

#[derive(Clone, Debug, Eq, PartialEq, Hash)]
pub enum ServiceAcceptType {
    DIDCommV1,
    DIDCommV2,
    Other(String),
}

impl From<&str> for ServiceAcceptType {
    fn from(s: &str) -> Self {
        match s {
            "didcomm/aip2;env=rfc19" => ServiceAcceptType::DIDCommV1,
            "didcomm/v2" => ServiceAcceptType::DIDCommV2,
            _ => ServiceAcceptType::Other(s.to_string()),
        }
    }
}

impl Display for ServiceAcceptType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ServiceAcceptType::DIDCommV1 => write!(f, "didcomm/aip2;env=rfc19"),
            ServiceAcceptType::DIDCommV2 => write!(f, "didcomm/v2"),
            ServiceAcceptType::Other(other) => write!(f, "{}", other),
        }
    }
}

impl<'de> Deserialize<'de> for ServiceAcceptType {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let s = String::deserialize(deserializer)?;
        match s.as_str() {
            "didcomm/aip2;env=rfc19" => Ok(ServiceAcceptType::DIDCommV1),
            "didcomm/v2" => Ok(ServiceAcceptType::DIDCommV2),
            _ => Ok(ServiceAcceptType::Other(s)),
        }
    }
}

impl Serialize for ServiceAcceptType {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        match self {
            ServiceAcceptType::DIDCommV1 => serializer.serialize_str("didcomm/aip2;env=rfc19"),
            ServiceAcceptType::DIDCommV2 => serializer.serialize_str("didcomm/v2"),
            ServiceAcceptType::Other(other) => serializer.serialize_str(other),
        }
    }
}
