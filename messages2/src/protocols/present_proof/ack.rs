use messages_macros::MessageContent;
use serde::{Deserialize, Serialize};

use crate::{
    composite_message::Message,
    message_type::message_family::present_proof::PresentProofV1_0,
    protocols::{
        notification::{AckContent, AckDecorators},
        traits::MessageKind,
    },
};

pub type AckPresentation = Message<AckPresentationContent, AckDecorators>;

#[derive(Clone, Debug, Deserialize, Serialize, MessageContent)]
#[message(kind = "PresentProofV1_0::Ack")]
#[serde(transparent)]
pub struct AckPresentationContent(pub AckContent);
