use serde::{Deserialize, Serialize};
use typed_builder::TypedBuilder;

#[derive(Clone, Debug, TypedBuilder, Serialize, Deserialize)]
#[builder(field_defaults(default))]
pub struct IssuerConfig {
    pub institution_did: String,
}
