use display_as_json::Display;
use serde::{Deserialize, Serialize};

use super::SovKeyKind;

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Default, Display)]
#[serde(rename_all = "camelCase")]
#[serde(deny_unknown_fields)]
pub struct ExtraFieldsLegacy {
    #[serde(default)]
    priority: u32,
    #[serde(default)]
    recipient_keys: Vec<SovKeyKind>,
    #[serde(default)]
    routing_keys: Vec<SovKeyKind>,
}

impl ExtraFieldsLegacy {
    pub fn recipient_keys(&self) -> &[SovKeyKind] {
        self.recipient_keys.as_ref()
    }

    pub fn routing_keys(&self) -> &[SovKeyKind] {
        self.routing_keys.as_ref()
    }

    pub fn priority(&self) -> u32 {
        self.priority
    }
}
