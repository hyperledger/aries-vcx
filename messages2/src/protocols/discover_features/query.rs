use messages_macros::MessageContent;
use serde::{Deserialize, Serialize};

use crate::{
    composite_message::Message, decorators::Timing,
    message_type::message_family::discover_features::DiscoverFeaturesV1_0, protocols::traits::MessageKind,
};

pub type Query = Message<QueryContent, QueryDecorators>;

#[derive(Clone, Debug, Deserialize, Serialize, MessageContent)]
#[message(kind = "DiscoverFeaturesV1_0::Query")]
pub struct QueryContent {
    pub query: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub comment: Option<String>,
}

impl QueryContent {
    pub fn new(query: String) -> Self {
        Self {
            query,
            comment: None,
        }
    }
}

#[derive(Clone, Debug, Deserialize, Serialize, Default)]
pub struct QueryDecorators {
    #[serde(rename = "~timing")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub timing: Option<Timing>,
}
