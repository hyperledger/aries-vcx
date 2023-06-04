use serde::{Deserialize, Serialize};

use super::AcceptType;

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Default)]
#[serde(rename_all = "camelCase")]
pub struct ExtraFieldsDidCommV2 {
    accept: Vec<AcceptType>,
    routing_keys: Vec<String>,
}

impl ExtraFieldsDidCommV2 {
    pub fn builder() -> ExtraFieldsDidCommV2Builder {
        ExtraFieldsDidCommV2Builder::default()
    }

    pub fn accept(&self) -> &[AcceptType] {
        self.accept.as_ref()
    }

    pub fn routing_keys(&self) -> &[String] {
        self.routing_keys.as_ref()
    }
}

#[derive(Default)]
pub struct ExtraFieldsDidCommV2Builder {
    routing_keys: Vec<String>,
}

impl ExtraFieldsDidCommV2Builder {
    pub fn set_routing_keys(mut self, routing_keys: Vec<String>) -> Self {
        self.routing_keys = routing_keys;
        self
    }

    pub fn build(self) -> ExtraFieldsDidCommV2 {
        ExtraFieldsDidCommV2 {
            accept: vec![AcceptType::DIDCommV2],
            routing_keys: self.routing_keys,
        }
    }
}
