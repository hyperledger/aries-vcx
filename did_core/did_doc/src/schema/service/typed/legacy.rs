use display_as_json::Display;
use serde::{Deserialize, Serialize};
use typed_builder::TypedBuilder;

use crate::schema::service::service_key_kind::ServiceKeyKind;

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Default, Display, TypedBuilder)]
#[serde(rename_all = "camelCase")]
#[serde(deny_unknown_fields)]
pub struct ExtraFieldsLegacy {
    #[serde(default)]
    priority: u32,
    #[serde(default)]
    recipient_keys: Vec<ServiceKeyKind>,
    #[serde(default)]
    routing_keys: Vec<ServiceKeyKind>,
}

impl ExtraFieldsLegacy {
    pub fn recipient_keys(&self) -> &[ServiceKeyKind] {
        self.recipient_keys.as_ref()
    }

    pub fn routing_keys(&self) -> &[ServiceKeyKind] {
        self.routing_keys.as_ref()
    }

    pub fn priority(&self) -> u32 {
        self.priority
    }
}
