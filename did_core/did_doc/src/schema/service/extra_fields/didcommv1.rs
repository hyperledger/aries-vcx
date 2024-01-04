use display_as_json::Display;
use serde::{Deserialize, Serialize};
use typed_builder::TypedBuilder;

use crate::schema::service::extra_fields::{ServiceAcceptType, ServiceKeyKind};

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Default, Display, TypedBuilder)]
#[serde(rename_all = "camelCase")]
#[serde(deny_unknown_fields)]
pub struct ExtraFieldsDidCommV1 {
    priority: u32,
    recipient_keys: Vec<ServiceKeyKind>,
    routing_keys: Vec<ServiceKeyKind>,
    #[serde(default)]
    accept: Vec<ServiceAcceptType>,
}

impl ExtraFieldsDidCommV1 {
    pub fn priority(&self) -> u32 {
        self.priority
    }

    pub fn recipient_keys(&self) -> &[ServiceKeyKind] {
        self.recipient_keys.as_ref()
    }

    pub fn routing_keys(&self) -> &[ServiceKeyKind] {
        self.routing_keys.as_ref()
    }

    pub fn accept(&self) -> &[ServiceAcceptType] {
        self.accept.as_ref()
    }
}
