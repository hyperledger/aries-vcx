use diddoc::aries::diddoc::AriesDidDoc;
use messages_macros::Message;
use serde::{Deserialize, Serialize};
use transitive::TransitiveFrom;

use crate::{
    decorators::{Thread, Timing},
    message_type::message_family::connection::ConnectionV1_0, aries_message::AriesMessage, macros::threadlike_opt_impl,
};

use crate::protocols::traits::ConcreteMessage;

use super::Connection;

#[derive(Clone, Debug, Deserialize, Serialize, Message, TransitiveFrom)]
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
