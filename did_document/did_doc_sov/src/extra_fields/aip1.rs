use display_as_json::Display;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Default, Display)]
#[serde(deny_unknown_fields)]
pub struct ExtraFieldsAIP1 {}
