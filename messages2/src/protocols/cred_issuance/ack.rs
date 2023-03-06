use messages_macros::MessageContent;
use serde::{Deserialize, Serialize};

use crate::{
    composite_message::Message,
    message_type::message_family::cred_issuance::CredentialIssuanceV1_0,
    protocols::{
        notification::{AckContent, AckDecorators},
        traits::MessageKind,
    },
};

pub type AckCredential = Message<AckCredentialContent, AckDecorators>;

#[derive(Clone, Debug, Deserialize, Serialize, MessageContent)]
#[message(kind = "CredentialIssuanceV1_0::Ack")]
#[serde(transparent)]
pub struct AckCredentialContent(pub AckContent);
