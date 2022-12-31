#![allow(clippy::or_fun_call)]
#![allow(clippy::module_inception)]
#![allow(clippy::derive_partial_eq_without_eq)]
#![allow(clippy::new_without_default)]
#![allow(clippy::inherent_to_string)]
#![allow(clippy::large_enum_variant)]
#![crate_name = "agency_client"]

#[macro_use]
extern crate lazy_static;

#[macro_use]
extern crate log;

#[macro_use]
extern crate serde_derive;

use serde::{de, Deserialize, Deserializer, Serialize, Serializer};
use serde_json::Value;

mod utils;

#[macro_use]
pub mod agency_client;

pub mod api;
pub mod configuration;

pub mod httpclient;
pub mod messages;
pub mod testing;
pub mod wallet;

pub mod errors;
mod internal;

#[derive(Clone, Debug, PartialEq)]
pub enum MessageStatusCode {
    Received,
    Reviewed,
}

impl ToString for MessageStatusCode {
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
