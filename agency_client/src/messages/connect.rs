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
