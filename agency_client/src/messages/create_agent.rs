use crate::message_type::MessageTypes;
use crate::messages::a2a_message::A2AMessageKinds;

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
