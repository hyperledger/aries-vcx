use crate::messages::{a2a_message::A2AMessageKinds, message_type::MessageType};

#[derive(Serialize, Deserialize, Debug, PartialEq)]
pub struct Connect {
    #[serde(rename = "@type")]
    msg_type: MessageType,
    #[serde(rename = "fromDID")]
    from_did: String,
    #[serde(rename = "fromDIDVerKey")]
    from_vk: String,
}

impl Connect {
    pub fn build(from_did: &str, from_vk: &str) -> Connect {
        Connect {
            msg_type: MessageType::build_v2(A2AMessageKinds::Connect),
            from_did: from_did.to_string(),
            from_vk: from_vk.to_string(),
        }
    }
}

#[derive(Serialize, Deserialize, Debug, PartialEq)]
pub struct ConnectResponse {
    #[serde(rename = "@type")]
    pub msg_type: MessageType,
    #[serde(rename = "withPairwiseDID")]
    pub from_did: String,
    #[serde(rename = "withPairwiseDIDVerKey")]
    pub from_vk: String,
}
