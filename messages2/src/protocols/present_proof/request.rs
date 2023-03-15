use messages_macros::MessageContent;
use serde::{Deserialize, Serialize};

use crate::{
    decorators::{Attachment, Thread, Timing},
    msg_types::types::present_proof::PresentProofV1_0Kind,
    Message,
};

pub type RequestPresentation = Message<RequestPresentationContent, RequestPresentationDecorators>;

#[derive(Clone, Debug, Deserialize, Serialize, MessageContent, PartialEq)]
#[message(kind = "PresentProofV1_0Kind::RequestPresentation")]
pub struct RequestPresentationContent {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub comment: Option<String>,
    #[serde(rename = "request_presentations~attach")]
    pub request_presentations_attach: Vec<Attachment>,
}

impl RequestPresentationContent {
    pub fn new(request_presentations_attach: Vec<Attachment>) -> Self {
        Self {
            comment: None,
            request_presentations_attach,
        }
    }
}

#[derive(Clone, Debug, Deserialize, Serialize, Default, PartialEq)]
pub struct RequestPresentationDecorators {
    #[serde(rename = "~thread")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub thread: Option<Thread>,
    #[serde(rename = "~timing")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub timing: Option<Timing>,
}
