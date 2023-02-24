use messages_macros::Message;
use serde::{Deserialize, Serialize};
use transitive::TransitiveFrom;

use crate::{
    aries_message::AriesMessage,
    message_type::message_family::present_proof::PresentProofV1_0,
    protocols::{common::ack::Ack, traits::ConcreteMessage}, macros::threadlike_ack,
};

use super::PresentProof;

#[derive(Clone, Debug, Deserialize, Serialize, Message, TransitiveFrom)]
#[message(kind = "PresentProofV1_0::Ack")]
#[transitive(into(PresentProof, AriesMessage))]
#[serde(transparent)]
pub struct AckPresentation(Ack);

threadlike_ack!(AckPresentation);