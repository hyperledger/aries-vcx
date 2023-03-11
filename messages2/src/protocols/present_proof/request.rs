use messages_macros::MessageContent;
use serde::{Deserialize, Serialize};

use crate::{
    composite_message::Message,
    decorators::{Attachment, Thread, Timing},
    message_type::message_protocol::present_proof::PresentProofV1_0Kind,
    protocols::traits::ConcreteMessage,
};

pub type RequestPresentation = Message<RequestPresentationContent, RequestPresentationDecorators>;

#[derive(Clone, Debug, Deserialize, Serialize, MessageContent)]
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

#[derive(Clone, Debug, Deserialize, Serialize, Default)]
pub struct RequestPresentationDecorators {
    #[serde(rename = "~thread")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub thread: Option<Thread>,
    #[serde(rename = "~timing")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub timing: Option<Timing>,
}
