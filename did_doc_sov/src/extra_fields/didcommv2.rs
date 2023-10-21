use serde::{Deserialize, Serialize};
use display_as_json::Display;

use super::{AcceptType, KeyKind};

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Default, Display)]
#[serde(rename_all = "camelCase")]
#[serde(deny_unknown_fields)]
pub struct ExtraFieldsDidCommV2 {
    routing_keys: Vec<KeyKind>,
    #[serde(default)]
    accept: Vec<AcceptType>,
}

impl ExtraFieldsDidCommV2 {
    pub fn builder() -> ExtraFieldsDidCommV2Builder {
        ExtraFieldsDidCommV2Builder::default()
    }

    pub fn accept(&self) -> &[AcceptType] {
        self.accept.as_ref()
    }

    pub fn routing_keys(&self) -> &[KeyKind] {
        self.routing_keys.as_ref()
    }
}

pub struct ExtraFieldsDidCommV2Builder {
    routing_keys: Vec<KeyKind>,
    accept: Vec<AcceptType>,
}

impl Default for ExtraFieldsDidCommV2Builder {
    fn default() -> Self {
        Self {
            routing_keys: Vec::new(),
            accept: vec![AcceptType::DIDCommV2],
        }
    }
}

impl ExtraFieldsDidCommV2Builder {
    pub fn set_routing_keys(mut self, routing_keys: Vec<KeyKind>) -> Self {
        self.routing_keys = routing_keys;
        self
    }

    pub fn set_accept(mut self, accept: Vec<AcceptType>) -> Self {
        self.accept = accept;
        self
    }

    pub fn add_accept(mut self, accept: AcceptType) -> Self {
        self.accept.push(accept);
        self
    }

    pub fn build(self) -> ExtraFieldsDidCommV2 {
        ExtraFieldsDidCommV2 {
            routing_keys: self.routing_keys,
            accept: self.accept,
        }
    }
}
