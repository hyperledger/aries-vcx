use std::{fmt::Arguments, str::FromStr};

use serde::{de::Error, Deserialize, Serialize};

use super::MessageFamily;

pub(crate) struct MessageType<'a> {
    pub protocol: MessageFamily,
    pub kind: &'a str,
}

impl<'de> Deserialize<'de> for MessageType<'de> {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let msg_type_str = <&str>::deserialize(deserializer)?;
        let Some((protocol_str, kind)) = msg_type_str.rsplit_once('/') else {
            return Err(D::Error::custom(format!("Invalid message type: {msg_type_str}")));
        };

        let protocol = match MessageFamily::from_str(protocol_str) {
            Ok(v) => Ok(v),
            Err(e) => {
                let msg = format!("Cannot parse message type: {msg_type_str}; Error: {e}");
                Err(D::Error::custom(msg))
            }
        }?;
        
        let msg_type = Self { protocol, kind };
        Ok(msg_type)
    }
}

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
