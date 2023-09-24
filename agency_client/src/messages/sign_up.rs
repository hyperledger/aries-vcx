use crate::messages::{a2a_message::A2AMessageKinds, message_type::MessageType};

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
