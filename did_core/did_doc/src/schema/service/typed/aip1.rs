use display_as_json::Display;
use serde::{Deserialize, Serialize};
use typed_builder::TypedBuilder;

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Default, Display, TypedBuilder)]
#[serde(deny_unknown_fields)]
pub struct ExtraFieldsAIP1 {}
