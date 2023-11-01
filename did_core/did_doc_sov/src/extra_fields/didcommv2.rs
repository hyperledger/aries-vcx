use display_as_json::Display;
use serde::{Deserialize, Serialize};

use super::{SovAcceptType, SovKeyKind};

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Default, Display)]
#[serde(rename_all = "camelCase")]
#[serde(deny_unknown_fields)]
pub struct ExtraFieldsDidCommV2 {
    routing_keys: Vec<SovKeyKind>,
    #[serde(default)]
    accept: Vec<SovAcceptType>,
}

impl ExtraFieldsDidCommV2 {
    pub fn builder() -> ExtraFieldsDidCommV2Builder {
        ExtraFieldsDidCommV2Builder::default()
    }

    pub fn accept(&self) -> &[SovAcceptType] {
        self.accept.as_ref()
    }

    pub fn routing_keys(&self) -> &[SovKeyKind] {
        self.routing_keys.as_ref()
    }
}

pub struct ExtraFieldsDidCommV2Builder {
    routing_keys: Vec<SovKeyKind>,
    accept: Vec<SovAcceptType>,
}

impl Default for ExtraFieldsDidCommV2Builder {
    fn default() -> Self {
        Self {
            routing_keys: Vec::new(),
            accept: vec![SovAcceptType::DIDCommV2],
        }
    }
}

impl ExtraFieldsDidCommV2Builder {
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

    pub fn build(self) -> ExtraFieldsDidCommV2 {
        ExtraFieldsDidCommV2 {
            routing_keys: self.routing_keys,
            accept: self.accept,
        }
    }
}
