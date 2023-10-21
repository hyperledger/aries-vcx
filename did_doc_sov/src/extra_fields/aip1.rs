use serde::{Deserialize, Serialize};
use display_as_json::Display;

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Default, Display)]
#[serde(deny_unknown_fields)]
pub struct ExtraFieldsAIP1 {}
