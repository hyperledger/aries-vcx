use messages_macros::Message;
use serde::{Deserialize, Serialize};
use transitive::TransitiveFrom;

use crate::{
    decorators::Timing, message_type::message_family::discover_features::DiscoverFeaturesV1_0,
    protocols::traits::ConcreteMessage, aries_message::AriesMessage,
};

use super::DiscoverFeatures;

#[derive(Clone, Debug, Deserialize, Serialize, Message, TransitiveFrom)]
#[message(kind = "DiscoverFeaturesV1_0::Query")]
#[transitive(into(DiscoverFeatures, AriesMessage))]
pub struct Query {
    #[serde(rename = "@id")]
    pub id: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub query: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub comment: Option<String>,
    #[serde(rename = "~timing")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub timing: Option<Timing>,
}
