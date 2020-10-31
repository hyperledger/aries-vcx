use serde_json::Value;

use error::{VcxError, VcxErrorKind, VcxResult};
use agency_comm::get_message::MessagePayload;
use agency_comm::message_type::*;
use agency_comm::thread::Thread;
use utils::libindy::crypto;

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
