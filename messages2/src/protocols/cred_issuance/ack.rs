use messages_macros::MessageContent;
use serde::{Deserialize, Serialize};

use crate::{
    composite_message::Message,
    msg_types::types::cred_issuance::CredentialIssuanceV1_0Kind,
    protocols::{
        notification::{AckContent, AckDecorators, AckStatus},
        traits::ConcreteMessage,
    },
};

pub type AckCredential = Message<AckCredentialContent, AckDecorators>;

#[derive(Clone, Debug, Deserialize, Serialize, MessageContent)]
#[message(kind = "CredentialIssuanceV1_0Kind::Ack")]
#[serde(transparent)]
pub struct AckCredentialContent(pub AckContent);

impl AckCredentialContent {
    pub fn new(status: AckStatus) -> Self {
        Self(AckContent::new(status))
    }
}
