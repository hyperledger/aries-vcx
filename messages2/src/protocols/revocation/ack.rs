use messages_macros::Message;
use serde::{Deserialize, Serialize};
use transitive::TransitiveFrom;

use crate::{
    message_type::message_family::revocation::RevocationV2_0,
    protocols::{common::ack::Ack, traits::ConcreteMessage}, aries_message::AriesMessage, macros::threadlike_ack,
};

use super::Revocation;

#[derive(Clone, Debug, Deserialize, Serialize, Message, TransitiveFrom)]
#[message(kind = "RevocationV2_0::Ack")]
#[transitive(into(Revocation, AriesMessage))]
#[serde(transparent)]
pub struct AckRevoke(Ack);

threadlike_ack!(AckRevoke);