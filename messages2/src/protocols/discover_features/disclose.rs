use messages_macros::Message;
use serde::{Deserialize, Serialize};
use transitive::TransitiveFrom;

use crate::{
    aries_message::AriesMessage,
    decorators::{Thread, Timing},
    macros::threadlike_opt_impl,
    message_type::message_family::discover_features::DiscoverFeaturesV1_0,
    protocols::traits::ConcreteMessage,
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

threadlike_opt_impl!(Disclose);

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Default)]
pub struct ProtocolDescriptor {
    pub pid: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub roles: Option<Vec<()>>,
}
