use messages_macros::MessageContent;
use serde::{Deserialize, Serialize};

use crate::{
    composite_message::Message,
    message_type::message_family::revocation::RevocationV2_0,
    protocols::{
        notification::{AckContent, AckDecorators},
        traits::MessageKind,
    },
};

pub type AckRevoke = Message<AckRevokeContent, AckDecorators>;

#[derive(Clone, Debug, Deserialize, Serialize, MessageContent)]
#[message(kind = "RevocationV2_0::Ack")]
#[serde(transparent)]
pub struct AckRevokeContent(pub AckContent);
