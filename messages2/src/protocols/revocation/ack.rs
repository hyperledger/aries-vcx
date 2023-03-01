use messages_macros::MessageContent;
use serde::{Deserialize, Serialize};
use transitive::TransitiveFrom;

use crate::{
    aries_message::AriesMessage,
    macros::threadlike_ack,
    message_type::message_family::revocation::RevocationV2_0,
    protocols::{common::ack::Ack, traits::MessageKind},
};

use super::Revocation;

#[derive(Clone, Debug, Deserialize, Serialize, MessageContent, TransitiveFrom)]
#[message(kind = "RevocationV2_0::Ack")]
#[transitive(into(Revocation, AriesMessage))]
#[serde(transparent)]
pub struct AckRevoke(Ack);

threadlike_ack!(AckRevoke);
