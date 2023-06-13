use std::fmt::Display;

use did_doc::did_parser::DidUrl;
use serde::{Deserialize, Deserializer, Serialize};

use crate::error::DidDocumentSovError;

pub mod aip1;
pub mod didcommv1;
pub mod didcommv2;

#[derive(Serialize, Clone, Debug, PartialEq)]
pub enum AcceptType {
    DIDCommV1,
    DIDCommV2,
    Other(String),
}

impl Display for AcceptType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            AcceptType::DIDCommV1 => write!(f, "didcomm/aip2;env=rfc19"),
            AcceptType::DIDCommV2 => write!(f, "didcomm/v2"),
            AcceptType::Other(other) => write!(f, "{}", other),
        }
    }
}

impl<'de> Deserialize<'de> for AcceptType {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let s = String::deserialize(deserializer)?;
        match s.as_str() {
            "didcomm/aip2;env=rfc19" => Ok(AcceptType::DIDCommV1),
            "didcomm/v2" => Ok(AcceptType::DIDCommV2),
            _ => Ok(AcceptType::Other(s)),
        }
    }
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
#[serde(untagged)]
pub enum KeyKind {
    Reference(DidUrl),
    Value(String),
}

impl Default for KeyKind {
    fn default() -> Self {
        KeyKind::Value(String::default())
    }
}

impl Display for KeyKind {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            KeyKind::Reference(did_url) => write!(f, "{}", did_url),
            KeyKind::Value(value) => write!(f, "{}", value),
        }
    }
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
#[serde(untagged)]
pub enum ExtraFieldsSov {
    DIDCommV1(didcommv1::ExtraFieldsDidCommV1),
    DIDCommV2(didcommv2::ExtraFieldsDidCommV2),
    AIP1(aip1::ExtraFieldsAIP1),
}

impl Default for ExtraFieldsSov {
    fn default() -> Self {
        ExtraFieldsSov::AIP1(aip1::ExtraFieldsAIP1::default())
    }
}

impl ExtraFieldsSov {
    pub fn recipient_keys(&self) -> Result<&[KeyKind], DidDocumentSovError> {
        match self {
            ExtraFieldsSov::DIDCommV1(extra) => Ok(extra.recipient_keys()),
            ExtraFieldsSov::AIP1(_) | ExtraFieldsSov::DIDCommV2(_) => {
                Err(DidDocumentSovError::EmptyCollection("recipient_keys"))
            }
        }
    }

    pub fn routing_keys(&self) -> Result<&[KeyKind], DidDocumentSovError> {
        match self {
            ExtraFieldsSov::DIDCommV1(extra) => Ok(extra.routing_keys()),
            ExtraFieldsSov::DIDCommV2(extra) => Ok(extra.routing_keys()),
            ExtraFieldsSov::AIP1(_) => Err(DidDocumentSovError::EmptyCollection("routing_keys")),
        }
    }

    pub fn first_recipient_key(&self) -> Result<&KeyKind, DidDocumentSovError> {
        self.recipient_keys()?
            .first()
            .ok_or(DidDocumentSovError::EmptyCollection("recipient_keys"))
    }

    pub fn first_routing_key(&self) -> Result<&KeyKind, DidDocumentSovError> {
        self.routing_keys()?
            .first()
            .ok_or(DidDocumentSovError::EmptyCollection("routing_keys"))
    }

    pub fn accept(&self) -> Result<&[AcceptType], DidDocumentSovError> {
        match self {
            ExtraFieldsSov::DIDCommV1(extra) => Ok(extra.accept()),
            ExtraFieldsSov::DIDCommV2(extra) => Ok(extra.accept()),
            ExtraFieldsSov::AIP1(_) => Err(DidDocumentSovError::EmptyCollection("accept")),
        }
    }

    pub fn priority(&self) -> Result<u32, DidDocumentSovError> {
        match self {
            ExtraFieldsSov::DIDCommV1(extra) => Ok(extra.priority()),
            _ => Err(DidDocumentSovError::EmptyCollection("priority")),
        }
    }
}
