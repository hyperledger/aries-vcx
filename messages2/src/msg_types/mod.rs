pub mod actor;
pub mod types;
pub mod registry;

pub use self::types::Protocol;

use std::{fmt::Arguments, str::FromStr};

use serde::{de::Error, Deserialize, Serialize};

/// Type used for deserialization of a fully qualified message type. After deserialization,
/// it is matched on to determine the actual message struct to deserialize to.
pub(crate) struct MessageType<'a> {
    /// The [`Protocol`] part of the message type (e.g: https://didcomm.org/connections/1.0)
    pub protocol: Protocol,
    /// The message kind of the specific protocol (e.g: request)
    pub kind: &'a str,
}

impl<'de> Deserialize<'de> for MessageType<'de> {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        // Deserialize to &str
        let msg_type_str = <&str>::deserialize(deserializer)?;

        // Split (from the right) at the first '/'.
        // The first element will be the string repr of the protocol
        // while the second will be the message kind.
        let Some((protocol_str, kind)) = msg_type_str.rsplit_once('/') else {
            return Err(D::Error::custom(format!("Invalid message type: {msg_type_str}")));
        };

        // Parse the Protocol instance
        let protocol = match Protocol::from_str(protocol_str) {
            Ok(v) => Ok(v),
            Err(e) => {
                let msg = format!("Cannot parse message type: {msg_type_str}; Error: {e}");
                Err(D::Error::custom(msg))
            }
        }?;

        // Create instance to be passed for specialized message deserialization later.
        let msg_type = Self { protocol, kind };
        Ok(msg_type)
    }
}

/// Type used for serialization of a message along with appending it's type field.
#[derive(Serialize)]
pub(crate) struct MsgWithType<'a, T> {
    #[serde(rename = "@type")]
    msg_type: Arguments<'a>,
    #[serde(flatten)]
    message: &'a T,
}

impl<'a, T> MsgWithType<'a, T> {
    pub fn new(msg_type: Arguments<'a>, message: &'a T) -> Self {
        Self { msg_type, message }
    }
}
