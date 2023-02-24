use messages_macros::Message;
use serde::{Deserialize, Serialize};
use transitive::TransitiveFrom;

use crate::{
    aries_message::AriesMessage,
    decorators::{Attachment, Thread, Timing},
    macros::threadlike_opt_impl,
    message_type::message_family::cred_issuance::CredentialIssuanceV1_0,
    protocols::traits::ConcreteMessage,
};

use super::CredentialIssuance;

#[derive(Clone, Debug, Deserialize, Serialize, Message, TransitiveFrom)]
#[message(kind = "CredentialIssuanceV1_0::RequestCredential")]
#[transitive(into(CredentialIssuance, AriesMessage))]
pub struct RequestCredential {
    #[serde(rename = "@id")]
    pub id: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub comment: Option<String>,
    #[serde(rename = "requests~attach")]
    pub requests_attach: Vec<Attachment>,
    #[serde(rename = "~thread")]
    pub thread: Option<Thread>,
    #[serde(rename = "~timing")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub timing: Option<Timing>,
}

threadlike_opt_impl!(RequestCredential);
