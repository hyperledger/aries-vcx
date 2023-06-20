use serde::{Deserialize, Serialize};

use super::{AcceptType, KeyKind};

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Default)]
#[serde(rename_all = "camelCase")]
pub struct ExtraFieldsDidCommV1 {
    priority: u32,
    recipient_keys: Vec<KeyKind>,
    routing_keys: Vec<KeyKind>,
    accept: Vec<AcceptType>,
}

impl ExtraFieldsDidCommV1 {
    pub fn builder() -> ExtraFieldsDidCommV1Builder {
        ExtraFieldsDidCommV1Builder::default()
    }

    pub fn priority(&self) -> u32 {
        self.priority
    }

    pub fn recipient_keys(&self) -> &[KeyKind] {
        self.recipient_keys.as_ref()
    }

    pub fn routing_keys(&self) -> &[KeyKind] {
        self.routing_keys.as_ref()
    }

    pub fn accept(&self) -> &[AcceptType] {
        self.accept.as_ref()
    }
}

#[derive(Default)]
pub struct ExtraFieldsDidCommV1Builder {
    priority: u32,
    recipient_keys: Vec<KeyKind>,
    routing_keys: Vec<KeyKind>,
}

impl ExtraFieldsDidCommV1Builder {
    pub fn set_priority(mut self, priority: u32) -> Self {
        self.priority = priority;
        self
    }

    pub fn set_recipient_keys(mut self, recipient_keys: Vec<KeyKind>) -> Self {
        self.recipient_keys = recipient_keys;
        self
    }

    pub fn set_routing_keys(mut self, routing_keys: Vec<KeyKind>) -> Self {
        self.routing_keys = routing_keys;
        self
    }

    pub fn build(self) -> ExtraFieldsDidCommV1 {
        ExtraFieldsDidCommV1 {
            priority: self.priority,
            recipient_keys: self.recipient_keys,
            routing_keys: self.routing_keys,
            accept: vec![AcceptType::DIDCommV1],
        }
    }
}
