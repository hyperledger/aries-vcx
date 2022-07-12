use serde::{de, Deserialize, Deserializer, Serialize, Serializer};
use serde_json::Value;
use crate::agent_utils::ComMethod;
use crate::message_type::MessageTypes;
use crate::messages::a2a_message::A2AMessageKinds;

#[derive(Serialize, Deserialize, Debug)]
pub struct Connect {
    #[serde(rename = "@type")]
    msg_type: MessageTypes,
    #[serde(rename = "fromDID")]
    from_did: String,
    #[serde(rename = "fromDIDVerKey")]
    from_vk: String,
}

impl Connect {
    pub fn build(from_did: &str, from_vk: &str) -> Connect {
        Connect {
            msg_type: MessageTypes::build(A2AMessageKinds::Connect),
            from_did: from_did.to_string(),
            from_vk: from_vk.to_string(),
        }
    }
}

#[derive(Serialize, Deserialize, Debug)]
pub struct ConnectResponse {
    #[serde(rename = "@type")]
    pub msg_type: MessageTypes,
    #[serde(rename = "withPairwiseDID")]
    pub from_did: String,
    #[serde(rename = "withPairwiseDIDVerKey")]
    pub from_vk: String,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct SignUp {
    #[serde(rename = "@type")]
    msg_type: MessageTypes,
}

impl SignUp {
    pub fn build() -> SignUp {
        SignUp {
            msg_type: MessageTypes::build(A2AMessageKinds::SignUp),
        }
    }
}

#[derive(Serialize, Deserialize, Debug)]
pub struct SignUpResponse {
    #[serde(rename = "@type")]
    msg_type: MessageTypes,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct CreateAgent {
    #[serde(rename = "@type")]
    msg_type: MessageTypes,
}

impl CreateAgent {
    pub fn build() -> CreateAgent {
        CreateAgent {
            msg_type: MessageTypes::build(A2AMessageKinds::CreateAgent),
        }
    }
}

#[derive(Serialize, Deserialize, Debug)]
pub struct CreateAgentResponse {
    #[serde(rename = "@type")]
    pub msg_type: MessageTypes,
    #[serde(rename = "withPairwiseDID")]
    pub from_did: String,
    #[serde(rename = "withPairwiseDIDVerKey")]
    pub from_vk: String,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct ComMethodUpdated {
    #[serde(rename = "@type")]
    msg_type: MessageTypes,
    id: String,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct UpdateComMethod {
    #[serde(rename = "@type")]
    msg_type: MessageTypes,
    #[serde(rename = "comMethod")]
    com_method: ComMethod,
}

#[derive(Debug, PartialEq)]
pub enum ComMethodType {
    A2A,
    Webhook,
}

impl Serialize for ComMethodType {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error> where S: Serializer {
        let value = match self {
            ComMethodType::A2A => "1",
            ComMethodType::Webhook => "2",
        };
        Value::String(value.to_string()).serialize(serializer)
    }
}

impl<'de> Deserialize<'de> for ComMethodType {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error> where D: Deserializer<'de> {
        let value = Value::deserialize(deserializer).map_err(de::Error::custom)?;
        match value.as_str() {
            Some("1") => Ok(ComMethodType::A2A),
            Some("2") => Ok(ComMethodType::Webhook),
            _ => Err(de::Error::custom("Unexpected communication method type."))
        }
    }
}

impl UpdateComMethod {
    pub fn build(com_method: ComMethod) -> UpdateComMethod {
        UpdateComMethod {
            msg_type: MessageTypes::build(A2AMessageKinds::UpdateComMethod),
            com_method,
        }
    }
}
