use messages_macros::Message;
use serde::{Deserialize, Serialize};

use crate::{message_type::message_family::present_proof::PresentProofV1_0, decorators::{Thread, PleaseAck, Attachment, Timing}, protocols::traits::ConcreteMessage};

#[derive(Clone, Debug, Deserialize, Serialize, Message)]
#[message(kind = "PresentProofV1_0::Presentation")]
pub struct Presentation{
    #[serde(rename = "@id")]
    pub id: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub comment: Option<String>,
    #[serde(rename = "presentations~attach")]
    pub presentations_attach: Vec<Attachment>,
    #[serde(rename = "~thread")]
    pub thread: Thread,
    #[serde(rename = "~please_ack")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub please_ack: Option<PleaseAck>,
    #[serde(rename = "~timing")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub timing: Option<Timing>,
}