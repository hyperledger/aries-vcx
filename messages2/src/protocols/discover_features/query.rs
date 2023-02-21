use messages_macros::Message;
use serde::{Deserialize, Serialize};

use crate::{
    decorators::Timing, message_type::message_family::discover_features::DiscoverFeaturesV1_0,
    protocols::traits::ConcreteMessage,
};

#[derive(Clone, Debug, Deserialize, Serialize, Message)]
#[message(kind = "DiscoverFeaturesV1_0::Query")]
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
