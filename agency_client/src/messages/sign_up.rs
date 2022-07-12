use serde::{de, Deserialize, Deserializer, Serialize, Serializer};
use serde_json::Value;
use crate::agent_utils::ComMethod;
use crate::message_type::MessageTypes;
use crate::messages::a2a_message::A2AMessageKinds;

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
