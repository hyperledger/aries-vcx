use crate::message_type::MessageType;

#[derive(Debug, Deserialize, Serialize, Clone, PartialEq)]
#[serde(untagged)]
pub enum PayloadTypes {
    PayloadTypeV2(MessageType),
}

#[derive(Debug, Clone, Deserialize, Serialize, PartialEq)]
pub enum PayloadKinds {
    CredOffer,
    CredReq,
    Cred,
    Proof,
    ProofRequest,
    ConnRequest,
    Other(String),
}
