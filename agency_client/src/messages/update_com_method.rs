use serde::{de, Deserialize, Deserializer, Serialize, Serializer};
use serde_json::Value;

use crate::{
    api::agent::ComMethod,
    messages::{a2a_message::A2AMessageKinds, message_type::MessageType},
};

#[derive(Serialize, Deserialize, Debug, PartialEq)]
pub struct ComMethodUpdated {
    #[serde(rename = "@type")]
    msg_type: MessageType,
    id: String,
}

#[derive(Serialize, Deserialize, Debug, PartialEq)]
pub struct UpdateComMethod {
    #[serde(rename = "@type")]
    msg_type: MessageType,
    #[serde(rename = "comMethod")]
    com_method: ComMethod,
}

#[derive(Debug, PartialEq)]
pub enum ComMethodType {
    A2A,
    Webhook,
}

impl Serialize for ComMethodType {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let value = match self {
            ComMethodType::A2A => "1",
            ComMethodType::Webhook => "2",
        };
        Value::String(value.to_string()).serialize(serializer)
    }
}

impl<'de> Deserialize<'de> for ComMethodType {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let value = Value::deserialize(deserializer).map_err(de::Error::custom)?;
        match value.as_str() {
            Some("1") => Ok(ComMethodType::A2A),
            Some("2") => Ok(ComMethodType::Webhook),
            _ => Err(de::Error::custom("Unexpected communication method type.")),
        }
    }
}

impl UpdateComMethod {
    pub fn build(com_method: ComMethod) -> UpdateComMethod {
        UpdateComMethod {
            msg_type: MessageType::build_v2(A2AMessageKinds::UpdateComMethod),
            com_method,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    #[cfg(feature = "general_test")]
    fn test_method_type_serialization() {
        assert_eq!(
            "\"1\"",
            serde_json::to_string::<ComMethodType>(&ComMethodType::A2A).unwrap()
        );
        assert_eq!(
            "\"2\"",
            serde_json::to_string::<ComMethodType>(&ComMethodType::Webhook).unwrap()
        );
    }

    #[test]
    #[cfg(feature = "general_test")]
    fn test_method_type_deserialization() {
        assert_eq!(
            ComMethodType::A2A,
            serde_json::from_str::<ComMethodType>("\"1\"").unwrap()
        );
        assert_eq!(
            ComMethodType::Webhook,
            serde_json::from_str::<ComMethodType>("\"2\"").unwrap()
        );
    }
}
