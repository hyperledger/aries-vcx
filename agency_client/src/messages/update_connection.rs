use serde::{de, Deserialize, Deserializer, Serialize, Serializer};
use serde_json::Value;

use crate::messages::{a2a_message::A2AMessageKinds, message_type::MessageType};

#[derive(Clone, Deserialize, Serialize, Debug, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct UpdateConnection {
    #[serde(rename = "@type")]
    msg_type: MessageType,
    #[serde(rename = "statusCode")]
    status_code: ConnectionStatus,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum ConnectionStatus {
    AlreadyConnected,
    NotConnected,
    Deleted,
}

impl Serialize for ConnectionStatus {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let value = match self {
            ConnectionStatus::AlreadyConnected => "CS-101",
            ConnectionStatus::NotConnected => "CS-102",
            ConnectionStatus::Deleted => "CS-103",
        };
        serde_json::Value::String(value.to_string()).serialize(serializer)
    }
}

impl<'de> Deserialize<'de> for ConnectionStatus {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let value = Value::deserialize(deserializer).map_err(de::Error::custom)?;
        match value.as_str() {
            Some("CS-101") => Ok(ConnectionStatus::AlreadyConnected),
            Some("CS-102") => Ok(ConnectionStatus::NotConnected),
            Some("CS-103") => Ok(ConnectionStatus::Deleted),
            _ => Err(de::Error::custom("Unexpected message type.")),
        }
    }
}

#[derive(Debug, Deserialize, Serialize, PartialEq)]
pub struct UpdateConnectionResponse {
    #[serde(rename = "@type")]
    msg_type: MessageType,
    #[serde(rename = "statusCode")]
    pub status_code: ConnectionStatus,
}

#[derive(Debug)]
pub struct DeleteConnectionBuilder {
    status_code: ConnectionStatus,
}

impl DeleteConnectionBuilder {
    pub fn create() -> DeleteConnectionBuilder {
        trace!("DeleteConnection::create_message >>>");

        DeleteConnectionBuilder {
            status_code: ConnectionStatus::Deleted,
        }
    }

    pub fn build(&self) -> UpdateConnection {
        UpdateConnection {
            msg_type: MessageType::build_v2(A2AMessageKinds::UpdateConnectionStatus),
            status_code: self.status_code.clone(),
        }
    }
}
