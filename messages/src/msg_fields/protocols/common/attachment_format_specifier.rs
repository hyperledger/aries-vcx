use serde::{Deserialize, Serialize};
use shared::maybe_known::MaybeKnown;
use typed_builder::TypedBuilder;

/// Specifies that a particular Attachment, with the id of `attach_id`, has the format of `format`.
#[derive(Debug, Serialize, Deserialize, Clone, PartialEq, TypedBuilder)]
#[serde(rename_all = "snake_case")]
pub struct AttachmentFormatSpecifier<F> {
    pub attach_id: String,
    pub format: MaybeKnown<F>,
}

/// If `attach_id` is not [None], this specifies that a particular Attachment, with the id of
/// `attach_id`, has the format of `format`. If `attach_id` is [None], this structure is used to
/// indicate that a particular attachment `format` is supported by the sender of the message.
#[derive(Debug, Serialize, Deserialize, Clone, PartialEq, TypedBuilder)]
#[serde(rename_all = "snake_case")]
pub struct OptionalIdAttachmentFormatSpecifier<F> {
    #[builder(default)]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub attach_id: Option<String>,
    pub format: MaybeKnown<F>,
}
