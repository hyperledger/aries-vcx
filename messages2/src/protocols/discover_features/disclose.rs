use messages_macros::MessageContent;
use serde::{Deserialize, Serialize};

use super::ProtocolDescriptor;
use crate::{
    decorators::{Thread, Timing},
    msg_types::{registry::PROTOCOL_REGISTRY, types::discover_features::DiscoverFeaturesV1_0Kind},
    Message,
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

        for entries in PROTOCOL_REGISTRY.clone().into_values() {
            for entry in entries {
                let mut pd = ProtocolDescriptor::new(entry.protocol.into());
                pd.roles = Some(entry.actors);
                protocols.push(pd);
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
