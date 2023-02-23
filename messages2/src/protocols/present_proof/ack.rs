use messages_macros::Message;
use serde::{Deserialize, Serialize};

use crate::{
    message_type::message_family::present_proof::PresentProofV1_0,
    protocols::{common::ack::Ack, traits::ConcreteMessage},
};

#[derive(Clone, Debug, Deserialize, Serialize, Message)]
#[message(kind = "PresentProofV1_0::Ack")]
#[serde(transparent)]
pub struct AckPresentation(Ack);
