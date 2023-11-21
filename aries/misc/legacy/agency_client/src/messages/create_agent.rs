use crate::messages::{a2a_message::A2AMessageKinds, message_type::MessageType};

#[derive(Serialize, Deserialize, Debug, PartialEq)]
pub struct CreateAgent {
    #[serde(rename = "@type")]
    msg_type: MessageType,
}

impl CreateAgent {
    pub fn build() -> CreateAgent {
        CreateAgent {
            msg_type: MessageType::build_v2(A2AMessageKinds::CreateAgent),
        }
    }
}

#[derive(Serialize, Deserialize, Debug, PartialEq)]
pub struct CreateAgentResponse {
    #[serde(rename = "@type")]
    pub msg_type: MessageType,
    #[serde(rename = "withPairwiseDID")]
    pub from_did: String,
    #[serde(rename = "withPairwiseDIDVerKey")]
    pub from_vk: String,
}
