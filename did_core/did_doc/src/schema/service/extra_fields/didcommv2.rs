use display_as_json::Display;
use serde::{Deserialize, Serialize};

use crate::schema::service::extra_fields::{ServiceAcceptType, ServiceKeyKind};

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Default, Display)]
#[serde(rename_all = "camelCase")]
#[serde(deny_unknown_fields)]
pub struct ExtraFieldsDidCommV2 {
    #[serde(skip_serializing_if = "Vec::is_empty")]
    routing_keys: Vec<ServiceKeyKind>,
    #[serde(default)]
    accept: Vec<ServiceAcceptType>,
}

impl ExtraFieldsDidCommV2 {
    pub fn builder() -> ExtraFieldsDidCommV2Builder {
        ExtraFieldsDidCommV2Builder::default()
    }

    pub fn accept(&self) -> &[ServiceAcceptType] {
        self.accept.as_ref()
    }

    pub fn routing_keys(&self) -> &[ServiceKeyKind] {
        self.routing_keys.as_ref()
    }
}

pub struct ExtraFieldsDidCommV2Builder {
    routing_keys: Vec<ServiceKeyKind>,
    accept: Vec<ServiceAcceptType>,
}

impl Default for ExtraFieldsDidCommV2Builder {
    fn default() -> Self {
        Self {
            routing_keys: Vec::new(),
            accept: vec![ServiceAcceptType::DIDCommV2],
        }
    }
}

impl ExtraFieldsDidCommV2Builder {
    pub fn set_routing_keys(mut self, routing_keys: Vec<ServiceKeyKind>) -> Self {
        self.routing_keys = routing_keys;
        self
    }

    pub fn set_accept(mut self, accept: Vec<ServiceAcceptType>) -> Self {
        self.accept = accept;
        self
    }

    pub fn add_accept(mut self, accept: ServiceAcceptType) -> Self {
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
