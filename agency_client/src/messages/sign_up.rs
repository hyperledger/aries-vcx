use serde::{de, Deserialize, Deserializer, Serialize, Serializer};
use serde_json::Value;
use crate::agent_utils::ComMethod;
use crate::message_type::MessageType;
use crate::messages::a2a_message::A2AMessageKinds;

#[derive(Serialize, Deserialize, Debug, PartialEq)]
pub struct SignUp {
    #[serde(rename = "@type")]
    msg_type: MessageType,
}

impl SignUp {
    pub fn build() -> SignUp {
        SignUp {
            msg_type: MessageType::build_v2(A2AMessageKinds::SignUp),
        }
    }
}

#[derive(Serialize, Deserialize, Debug, PartialEq)]
pub struct SignUpResponse {
    #[serde(rename = "@type")]
    msg_type: MessageType,
}
