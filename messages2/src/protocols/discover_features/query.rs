use messages_macros::MessageContent;
use serde::{Deserialize, Serialize};

use crate::{
    decorators::Timing, message_type::message_family::discover_features::DiscoverFeaturesV1_0,
    protocols::traits::MessageKind,
};

#[derive(Clone, Debug, Deserialize, Serialize, MessageContent)]
#[message(kind = "DiscoverFeaturesV1_0::Query")]
pub struct Query {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub query: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub comment: Option<String>,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct QueryDecorators {
    #[serde(rename = "~timing")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub timing: Option<Timing>,
}
