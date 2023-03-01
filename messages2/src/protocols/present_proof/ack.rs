use messages_macros::MessageContent;
use serde::{Deserialize, Serialize};
use transitive::TransitiveFrom;

use crate::{
    aries_message::AriesMessage,
    macros::threadlike_ack,
    message_type::message_family::present_proof::PresentProofV1_0,
    protocols::{common::ack::Ack, traits::MessageKind},
};

use super::PresentProof;

#[derive(Clone, Debug, Deserialize, Serialize, MessageContent, TransitiveFrom)]
#[message(kind = "PresentProofV1_0::Ack")]
#[transitive(into(PresentProof, AriesMessage))]
#[serde(transparent)]
pub struct AckPresentation(Ack);

threadlike_ack!(AckPresentation);
