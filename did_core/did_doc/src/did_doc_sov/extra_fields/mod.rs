use std::{collections::HashMap, fmt::Display};

use did_key::DidKey;
use did_parser::DidUrl;
use serde::{Deserialize, Deserializer, Serialize};
use serde_json::Value;

use crate::did_doc_sov::error::DidDocumentSovError;

pub mod aip1;
pub mod didcommv1;
pub mod didcommv2;
pub mod legacy;

pub fn convert_to_hashmap<T: Serialize>(
    value: &T,
) -> Result<HashMap<String, Value>, DidDocumentSovError> {
    let serialized_value = serde_json::to_value(value)?;

    match serialized_value {
        Value::Object(map) => Ok(map.into_iter().collect()),
        _ => Err(DidDocumentSovError::ParsingError(
            "Expected JSON object".to_string(),
        )),
    }
}

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
    pub fn recipient_keys(&self) -> Result<&[ServiceKeyKind], DidDocumentSovError> {
        match self {
            ExtraFieldsSov::DIDCommV1(extra) => Ok(extra.recipient_keys()),
            ExtraFieldsSov::Legacy(extra) => Ok(extra.recipient_keys()),
            ExtraFieldsSov::AIP1(_) | ExtraFieldsSov::DIDCommV2(_) => {
                Err(DidDocumentSovError::EmptyCollection("recipient_keys"))
            }
        }
    }

    pub fn routing_keys(&self) -> Result<&[ServiceKeyKind], DidDocumentSovError> {
        match self {
            ExtraFieldsSov::DIDCommV1(extra) => Ok(extra.routing_keys()),
            ExtraFieldsSov::DIDCommV2(extra) => Ok(extra.routing_keys()),
            ExtraFieldsSov::Legacy(extra) => Ok(extra.routing_keys()),
            ExtraFieldsSov::AIP1(_) => Err(DidDocumentSovError::EmptyCollection("routing_keys")),
        }
    }

    pub fn first_recipient_key(&self) -> Result<&ServiceKeyKind, DidDocumentSovError> {
        self.recipient_keys()?
            .first()
            .ok_or(DidDocumentSovError::EmptyCollection("recipient_keys"))
    }

    pub fn first_routing_key(&self) -> Result<&ServiceKeyKind, DidDocumentSovError> {
        self.routing_keys()?
            .first()
            .ok_or(DidDocumentSovError::EmptyCollection("routing_keys"))
    }

    pub fn accept(&self) -> Result<&[ServiceAcceptType], DidDocumentSovError> {
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
