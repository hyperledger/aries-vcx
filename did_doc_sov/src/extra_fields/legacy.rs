use serde::{Deserialize, Serialize};

use super::KeyKind;

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Default)]
#[serde(rename_all = "camelCase")]
pub struct ExtraFieldsLegacy {
    #[serde(default)]
    priority: u32,
    #[serde(default)]
    recipient_keys: Vec<KeyKind>,
    #[serde(default)]
    routing_keys: Vec<KeyKind>,
}

impl ExtraFieldsLegacy {
    pub fn recipient_keys(&self) -> &[KeyKind] {
        self.recipient_keys.as_ref()
    }

    pub fn routing_keys(&self) -> &[KeyKind] {
        self.routing_keys.as_ref()
    }

    pub fn priority(&self) -> u32 {
        self.priority
    }
}
