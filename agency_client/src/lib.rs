#![crate_name = "agency_client"]

#[macro_use]
extern crate lazy_static;

#[macro_use]
extern crate log;

#[macro_use]
extern crate serde_derive;

use serde::{de, Deserialize, Deserializer, Serialize, Serializer};
use serde_json::Value;

pub mod error;
use error::prelude::*;

mod utils;
use utils::validation;

#[macro_use]
pub mod agency_client;

pub mod api;
pub mod configuration;

pub mod httpclient;
pub mod messages;
pub mod testing;
pub mod wallet;

mod internal;

#[derive(Clone, Debug, PartialEq)]
pub enum MessageStatusCode {
    Received,
    Reviewed,
}

impl std::string::ToString for MessageStatusCode {
    fn to_string(&self) -> String {
        match self {
            MessageStatusCode::Received => "MS-103",
            MessageStatusCode::Reviewed => "MS-106",
        }
        .to_string()
    }
}

impl Serialize for MessageStatusCode {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let value = self.to_string();
        Value::String(value).serialize(serializer)
    }
}

impl<'de> Deserialize<'de> for MessageStatusCode {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let value = Value::deserialize(deserializer).map_err(de::Error::custom)?;
        match value.as_str() {
            Some("MS-103") => Ok(MessageStatusCode::Received),
            Some("MS-106") => Ok(MessageStatusCode::Reviewed),
            _ => Err(de::Error::custom("Unexpected message type.")),
        }
    }
}
