use diddoc::aries::diddoc::AriesDidDoc;
use messages_macros::MessageContent;
use serde::{Deserialize, Serialize};
use transitive::TransitiveFrom;

use crate::{
    aries_message::AriesMessage,
    decorators::{Thread, Timing},
    macros::threadlike_opt_impl,
    message_type::message_family::connection::ConnectionV1_0,
};

use crate::protocols::traits::MessageKind;

use super::Connection;

#[derive(Clone, Debug, Deserialize, Serialize, MessageContent, TransitiveFrom)]
#[message(kind = "ConnectionV1_0::Request")]
#[transitive(into(Connection, AriesMessage))]
pub struct Request {
    #[serde(rename = "@id")]
    pub id: String,
    pub label: String,
    pub connection: ConnectionData,
    #[serde(skip_serializing_if = "Option::is_none")]
    #[serde(rename = "~thread")]
    pub thread: Option<Thread>,
    #[serde(rename = "~timing")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub timing: Option<Timing>,
}

threadlike_opt_impl!(Request);

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct ConnectionData {
    #[serde(rename = "DID")]
    pub did: String,
    #[serde(rename = "DIDDoc")]
    pub did_doc: AriesDidDoc,
}
