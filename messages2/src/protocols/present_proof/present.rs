use messages_macros::Message;
use serde::{Deserialize, Serialize};
use transitive::TransitiveFrom;

use crate::{
    decorators::{Attachment, PleaseAck, Thread, Timing},
    message_type::message_family::present_proof::PresentProofV1_0,
    protocols::traits::ConcreteMessage, aries_message::AriesMessage, macros::threadlike_impl,
};

use super::PresentProof;

#[derive(Clone, Debug, Deserialize, Serialize, Message, TransitiveFrom)]
#[message(kind = "PresentProofV1_0::Presentation")]
#[transitive(into(PresentProof, AriesMessage))]
pub struct Presentation {
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

threadlike_impl!(Presentation);