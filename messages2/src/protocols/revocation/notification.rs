use messages_macros::Message;
use serde::{Deserialize, Serialize};
use transitive::TransitiveFrom;

use crate::{
    decorators::{PleaseAck, Thread, Timing},
    message_type::message_family::revocation::RevocationV2_0,
    protocols::traits::ConcreteMessage, aries_message::AriesMessage, macros::threadlike_opt_impl,
};

use super::Revocation;

#[derive(Clone, Debug, Deserialize, Serialize, Message, TransitiveFrom)]
#[message(kind = "RevocationV2_0::Revoke")]
#[transitive(into(Revocation, AriesMessage))]
pub struct Revoke {
    #[serde(rename = "@id")]
    id: String,
    credential_id: String,
    revocation_format: RevocationFormat,
    #[serde(skip_serializing_if = "Option::is_none")]
    comment: Option<String>,
    #[serde(rename = "~please_ack")]
    #[serde(skip_serializing_if = "Option::is_none")]
    please_ack: Option<PleaseAck>,
    #[serde(rename = "~timing")]
    #[serde(skip_serializing_if = "Option::is_none")]
    timing: Option<Timing>,
    #[serde(rename = "~thread")]
    #[serde(skip_serializing_if = "Option::is_none")]
    thread: Option<Thread>,
}

threadlike_opt_impl!(Revoke);

#[derive(Debug, Serialize, Deserialize, PartialEq, Clone, Default)]
#[serde(rename_all = "kebab-case")]
pub enum RevocationFormat {
    #[default]
    IndyAnoncreds,
}
