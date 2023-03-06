use messages_macros::MessageContent;
use serde::{Deserialize, Serialize};

use crate::{
    message_type::message_family::revocation::RevocationV2_0,
    protocols::{notification::AckContent, traits::MessageKind},
};

#[derive(Clone, Debug, Deserialize, Serialize, MessageContent)]
#[message(kind = "RevocationV2_0::Ack")]
#[serde(transparent)]
pub struct AckRevoke(pub AckContent);
