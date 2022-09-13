#![crate_name = "agency_client"]
extern crate async_std;
extern crate failure;
extern crate futures;
extern crate indyrs as indy;
#[macro_use]
extern crate lazy_static;
#[macro_use]
extern crate log;
extern crate reqwest;
extern crate rmp_serde;
extern crate serde;
#[macro_use]
extern crate serde_derive;
extern crate serde_json;
extern crate url;

use serde::{de, Deserialize, Deserializer, Serialize, Serializer};
use serde_json::Value;

use self::error::prelude::*;
use self::utils::validation;

pub mod utils;
#[macro_use]
pub mod agency_client;
pub mod api;
pub mod configuration;
pub mod error;
pub mod httpclient;
mod internal;
pub mod messages;
pub mod testing;

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
