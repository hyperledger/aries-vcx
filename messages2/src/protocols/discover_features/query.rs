use messages_macros::MessageContent;
use serde::{Deserialize, Serialize};

use crate::{
    composite_message::Message,
    decorators::Timing,
    message_type::{
        message_protocol::discover_features::DiscoverFeaturesV1_0Kind,
        registry::{ProtocolDescriptor, PROTOCOL_REGISTRY},
    },
    protocols::traits::ConcreteMessage,
};

pub type Query = Message<QueryContent, QueryDecorators>;

#[derive(Clone, Debug, Deserialize, Serialize, MessageContent)]
#[message(kind = "DiscoverFeaturesV1_0Kind::Query")]
pub struct QueryContent {
    pub query: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub comment: Option<String>,
}

impl QueryContent {
    pub fn new(query: String) -> Self {
        Self { query, comment: None }
    }

    pub fn lookup(&self) -> Vec<ProtocolDescriptor> {
        let mut protocols = Vec::new();
        let query = self.query.split('*').next().expect("at least one value");

        for versions in PROTOCOL_REGISTRY.values() {
            for minor in versions.values() {
                for pd in minor.values() {
                    if pd.pid.starts_with(query) {
                        protocols.push(pd.clone());
                    }
                }
            }
        }

        protocols
    }
}

#[derive(Clone, Debug, Deserialize, Serialize, Default)]
pub struct QueryDecorators {
    #[serde(rename = "~timing")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub timing: Option<Timing>,
}
