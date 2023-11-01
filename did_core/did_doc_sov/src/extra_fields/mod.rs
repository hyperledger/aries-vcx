use std::collections::HashMap;
use std::fmt::Display;

use did_doc::did_parser::DidUrl;
use did_key::DidKey;
use serde::{Deserialize, Deserializer, Serialize};
use serde_json::Value;

use crate::error::DidDocumentSovError;

pub mod aip1;
pub mod didcommv1;
pub mod didcommv2;
pub mod legacy;

pub fn convert_to_hashmap<T: Serialize>(value: &T) -> Result<HashMap<String, Value>, DidDocumentSovError> {
    let serialized_value = serde_json::to_value(value)?;

    match serialized_value {
        Value::Object(map) => Ok(map.into_iter().collect()),
        _ => Err(DidDocumentSovError::ParsingError("Expected JSON object".to_string())),
    }
}

#[derive(Clone, Debug, Eq, PartialEq, Hash)]
pub enum SovAcceptType {
    DIDCommV1,
    DIDCommV2,
    Other(String),
}

impl From<&str> for SovAcceptType {
    fn from(s: &str) -> Self {
        match s {
            "didcomm/aip2;env=rfc19" => SovAcceptType::DIDCommV1,
            "didcomm/v2" => SovAcceptType::DIDCommV2,
            _ => SovAcceptType::Other(s.to_string()),
        }
    }
}

impl Display for SovAcceptType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            SovAcceptType::DIDCommV1 => write!(f, "didcomm/aip2;env=rfc19"),
            SovAcceptType::DIDCommV2 => write!(f, "didcomm/v2"),
            SovAcceptType::Other(other) => write!(f, "{}", other),
        }
    }
}

impl<'de> Deserialize<'de> for SovAcceptType {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let s = String::deserialize(deserializer)?;
        match s.as_str() {
            "didcomm/aip2;env=rfc19" => Ok(SovAcceptType::DIDCommV1),
            "didcomm/v2" => Ok(SovAcceptType::DIDCommV2),
            _ => Ok(SovAcceptType::Other(s)),
        }
    }
}

impl Serialize for SovAcceptType {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        match self {
            SovAcceptType::DIDCommV1 => serializer.serialize_str("didcomm/aip2;env=rfc19"),
            SovAcceptType::DIDCommV2 => serializer.serialize_str("didcomm/v2"),
            SovAcceptType::Other(other) => serializer.serialize_str(other),
        }
    }
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
#[serde(untagged)]
pub enum SovKeyKind {
    DidKey(DidKey),
    Reference(DidUrl),
    Value(String),
}

impl Display for SovKeyKind {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            SovKeyKind::Reference(did_url) => write!(f, "{}", did_url),
            SovKeyKind::Value(value) => write!(f, "{}", value),
            SovKeyKind::DidKey(did_key) => write!(f, "{}", did_key),
        }
    }
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, display_as_json::Display)]
#[serde(untagged)]
pub enum ExtraFieldsSov {
    DIDCommV1(didcommv1::ExtraFieldsDidCommV1),
    DIDCommV2(didcommv2::ExtraFieldsDidCommV2),
    AIP1(aip1::ExtraFieldsAIP1),
    Legacy(legacy::ExtraFieldsLegacy),
}

impl Default for ExtraFieldsSov {
    fn default() -> Self {
        ExtraFieldsSov::AIP1(aip1::ExtraFieldsAIP1::default())
    }
}

impl ExtraFieldsSov {
    pub fn recipient_keys(&self) -> Result<&[SovKeyKind], DidDocumentSovError> {
        match self {
            ExtraFieldsSov::DIDCommV1(extra) => Ok(extra.recipient_keys()),
            ExtraFieldsSov::Legacy(extra) => Ok(extra.recipient_keys()),
            ExtraFieldsSov::AIP1(_) | ExtraFieldsSov::DIDCommV2(_) => {
                Err(DidDocumentSovError::EmptyCollection("recipient_keys"))
            }
        }
    }

    pub fn routing_keys(&self) -> Result<&[SovKeyKind], DidDocumentSovError> {
        match self {
            ExtraFieldsSov::DIDCommV1(extra) => Ok(extra.routing_keys()),
            ExtraFieldsSov::DIDCommV2(extra) => Ok(extra.routing_keys()),
            ExtraFieldsSov::Legacy(extra) => Ok(extra.routing_keys()),
            ExtraFieldsSov::AIP1(_) => Err(DidDocumentSovError::EmptyCollection("routing_keys")),
        }
    }

    pub fn first_recipient_key(&self) -> Result<&SovKeyKind, DidDocumentSovError> {
        self.recipient_keys()?
            .first()
            .ok_or(DidDocumentSovError::EmptyCollection("recipient_keys"))
    }

    pub fn first_routing_key(&self) -> Result<&SovKeyKind, DidDocumentSovError> {
        self.routing_keys()?
            .first()
            .ok_or(DidDocumentSovError::EmptyCollection("routing_keys"))
    }

    pub fn accept(&self) -> Result<&[SovAcceptType], DidDocumentSovError> {
        match self {
            ExtraFieldsSov::DIDCommV1(extra) => Ok(extra.accept()),
            ExtraFieldsSov::DIDCommV2(extra) => Ok(extra.accept()),
            ExtraFieldsSov::AIP1(_) | ExtraFieldsSov::Legacy(_) => {
                Err(DidDocumentSovError::EmptyCollection("accept"))
            }
        }
    }

    pub fn priority(&self) -> Result<u32, DidDocumentSovError> {
        match self {
            ExtraFieldsSov::DIDCommV1(extra) => Ok(extra.priority()),
            ExtraFieldsSov::Legacy(extra) => Ok(extra.priority()),
            _ => Err(DidDocumentSovError::EmptyCollection("priority")),
        }
    }
}
