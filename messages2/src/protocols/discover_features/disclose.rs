use messages_macros::Message;
use serde::{Deserialize, Serialize};
use transitive::TransitiveFrom;

use crate::{
    decorators::{Thread, Timing},
    message_type::message_family::discover_features::DiscoverFeaturesV1_0,
    protocols::traits::ConcreteMessage, aries_message::AriesMessage,
};

use super::DiscoverFeatures;

#[derive(Clone, Debug, Deserialize, Serialize, Message, TransitiveFrom)]
#[message(kind = "DiscoverFeaturesV1_0::Disclose")]
#[transitive(into(DiscoverFeatures, AriesMessage))]
pub struct Disclose {
    #[serde(rename = "@id")]
    pub id: String,
    pub protocols: Vec<ProtocolDescriptor>,
    #[serde(rename = "~thread")]
    pub thread: Option<Thread>,
    #[serde(rename = "~timing")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub timing: Option<Timing>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Default)]
pub struct ProtocolDescriptor {
    pub pid: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub roles: Option<Vec<()>>,
}
