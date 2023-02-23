use messages_macros::Message;
use serde::{Deserialize, Serialize};

use crate::{
    decorators::{Attachment, Thread, Timing},
    message_type::message_family::present_proof::PresentProofV1_0,
    protocols::traits::ConcreteMessage,
};

#[derive(Clone, Debug, Deserialize, Serialize, Message)]
#[message(kind = "PresentProofV1_0::RequestPresentation")]
pub struct RequestPresentation {
    #[serde(rename = "@id")]
    pub id: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub comment: Option<String>,
    #[serde(rename = "request_presentations~attach")]
    pub request_presentations_attach: Vec<Attachment>,
    #[serde(rename = "~thread")]
    pub thread: Option<Thread>,
    #[serde(rename = "~timing")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub timing: Option<Timing>,
}
