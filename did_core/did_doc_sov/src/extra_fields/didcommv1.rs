use display_as_json::Display;
use serde::{Deserialize, Serialize};

use super::{SovAcceptType, SovKeyKind};

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Default, Display)]
#[serde(rename_all = "camelCase")]
#[serde(deny_unknown_fields)]
pub struct ExtraFieldsDidCommV1 {
    priority: u32,
    recipient_keys: Vec<SovKeyKind>,
    routing_keys: Vec<SovKeyKind>,
    #[serde(default)]
    accept: Vec<SovAcceptType>,
}

impl ExtraFieldsDidCommV1 {
    pub fn builder() -> ExtraFieldsDidCommV1Builder {
        ExtraFieldsDidCommV1Builder::default()
    }

    pub fn priority(&self) -> u32 {
        self.priority
    }

    pub fn recipient_keys(&self) -> &[SovKeyKind] {
        self.recipient_keys.as_ref()
    }

    pub fn routing_keys(&self) -> &[SovKeyKind] {
        self.routing_keys.as_ref()
    }

    pub fn accept(&self) -> &[SovAcceptType] {
        self.accept.as_ref()
    }
}

pub struct ExtraFieldsDidCommV1Builder {
    priority: u32,
    recipient_keys: Vec<SovKeyKind>,
    routing_keys: Vec<SovKeyKind>,
    accept: Vec<SovAcceptType>,
}

impl Default for ExtraFieldsDidCommV1Builder {
    fn default() -> Self {
        Self {
            priority: 0,
            recipient_keys: Vec::new(),
            routing_keys: Vec::new(),
            accept: vec![SovAcceptType::DIDCommV1],
        }
    }
}

impl ExtraFieldsDidCommV1Builder {
    pub fn set_priority(mut self, priority: u32) -> Self {
        self.priority = priority;
        self
    }

    pub fn set_recipient_keys(mut self, recipient_keys: Vec<SovKeyKind>) -> Self {
        self.recipient_keys = recipient_keys;
        self
    }

    pub fn set_routing_keys(mut self, routing_keys: Vec<SovKeyKind>) -> Self {
        self.routing_keys = routing_keys;
        self
    }

    pub fn set_accept(mut self, accept: Vec<SovAcceptType>) -> Self {
        self.accept = accept;
        self
    }

    pub fn add_accept(mut self, accept: SovAcceptType) -> Self {
        self.accept.push(accept);
        self
    }

    pub fn build(self) -> ExtraFieldsDidCommV1 {
        ExtraFieldsDidCommV1 {
            priority: self.priority,
            recipient_keys: self.recipient_keys,
            routing_keys: self.routing_keys,
            accept: self.accept,
        }
    }
}
