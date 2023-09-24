use serde::{Serialize, Deserialize};
use typed_builder::TypedBuilder;

use crate::misc::MimeType;

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq, TypedBuilder)]
#[serde(rename_all = "kebab-case")]
pub struct CredentialAttr {
    pub name: String,
    pub value: String,
    #[builder(default, setter(strip_option))]
    #[serde(skip_serializing_if = "Option::is_none")]
    #[serde(rename = "mime-type")]
    pub mime_type: Option<MimeType>,
}
