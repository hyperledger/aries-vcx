use messages_macros::Message;
use serde::{Deserialize, Serialize};

use crate::{
    message_type::message_family::revocation::RevocationV2_0,
    protocols::{common::ack::Ack, traits::ConcreteMessage},
};

#[derive(Clone, Debug, Deserialize, Serialize, Message)]
#[message(kind = "RevocationV2_0::Ack")]
#[serde(transparent)]
pub struct AckRevoke(Ack);
