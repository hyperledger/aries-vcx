use agency_comm::message_type::MessageTypeV2;

#[derive(Debug, Deserialize, Serialize, Clone, PartialEq)]
#[serde(untagged)]
pub enum PayloadTypes {
    PayloadTypeV1(PayloadTypeV1),
    PayloadTypeV2(PayloadTypeV2),
}

#[derive(Clone, Deserialize, Serialize, Debug, PartialEq)]
pub struct PayloadTypeV1 {
    pub name: String,
    ver: String,
    fmt: String,
}

type PayloadTypeV2 = MessageTypeV2;

#[derive(Debug, Clone, Deserialize, Serialize, PartialEq)]
pub enum PayloadKinds {
    CredOffer,
    CredReq,
    Cred,
    Proof,
    ProofRequest,
    Other(String),
}
