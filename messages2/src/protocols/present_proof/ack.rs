use messages_macros::MessageContent;
use serde::{Deserialize, Serialize};

use crate::{
    msg_types::types::present_proof::PresentProofV1_0Kind,
    protocols::{
        notification::{AckContent, AckDecorators, AckStatus},
        
    }, Message,
};

pub type AckPresentation = Message<AckPresentationContent, AckDecorators>;

#[derive(Clone, Debug, Deserialize, Serialize, MessageContent)]
#[message(kind = "PresentProofV1_0Kind::Ack")]
#[serde(transparent)]
pub struct AckPresentationContent(pub AckContent);

impl AckPresentationContent {
    pub fn new(status: AckStatus) -> Self {
        Self(AckContent::new(status))
    }
}
