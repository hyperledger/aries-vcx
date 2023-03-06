use messages_macros::MessageContent;
use serde::{Deserialize, Serialize};

use crate::{
    message_type::message_family::present_proof::PresentProofV1_0,
    protocols::{notification::{AckContent, AckDecorators}, traits::MessageKind}, composite_message::Message,
};

pub type AckPresentation = Message<AckPresentationContent, AckDecorators>;

#[derive(Clone, Debug, Deserialize, Serialize, MessageContent)]
#[message(kind = "PresentProofV1_0::Ack")]
#[serde(transparent)]
pub struct AckPresentationContent(pub AckContent);
