use messages_macros::MessageContent;
use serde::{Deserialize, Serialize};

use crate::{
    composite_message::Message,
    decorators::{Thread, Timing},
    message_type::{
        message_protocol::discover_features::DiscoverFeaturesV1_0Kind,
        registry::{ProtocolDescriptor, PROTOCOL_REGISTRY},
    },
    protocols::traits::ConcreteMessage,
};

pub type Disclose = Message<DiscloseContent, DiscloseDecorators>;

#[derive(Clone, Debug, Deserialize, Serialize, MessageContent)]
#[message(kind = "DiscoverFeaturesV1_0Kind::Disclose")]
pub struct DiscloseContent {
    pub protocols: Vec<ProtocolDescriptor>,
}

impl DiscloseContent {
    pub fn new() -> Self {
        let mut protocols = Vec::new();

        for versions in PROTOCOL_REGISTRY.clone().into_values() {
            for minor in versions.into_values() {
                for pd in minor.into_values() {
                    protocols.push(pd);
                }
            }
        }

        Self { protocols }
    }
}

#[derive(Clone, Debug, Deserialize, Serialize, Default)]
pub struct DiscloseDecorators {
    #[serde(rename = "~thread")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub thread: Option<Thread>,
    #[serde(rename = "~timing")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub timing: Option<Timing>,
}
