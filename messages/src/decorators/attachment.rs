use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use shared_vcx::maybe_known::MaybeKnown;
use typed_builder::TypedBuilder;
use url::Url;

use crate::misc::MimeType;

/// Struct representing the `~attach` decorator from its [RFC](<https://github.com/hyperledger/aries-rfcs/blob/main/concepts/0017-attachments/README.md>).
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, TypedBuilder)]
#[serde(rename_all = "snake_case")]
pub struct Attachment {
    #[builder(default, setter(strip_option))]
    #[serde(skip_serializing_if = "Option::is_none")]
    #[serde(rename = "@id")]
    pub id: Option<String>,
    #[builder(default, setter(strip_option))]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    #[builder(default, setter(strip_option))]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub filename: Option<String>,
    #[builder(default, setter(strip_option))]
    #[serde(skip_serializing_if = "Option::is_none")]
    #[serde(rename = "mime-type")]
    pub mime_type: Option<MimeType>,
    #[builder(default, setter(strip_option))]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub lastmod_time: Option<DateTime<Utc>>,
    #[builder(default, setter(strip_option))]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub byte_count: Option<u64>,
    pub data: AttachmentData,
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, TypedBuilder)]
pub struct AttachmentData {
    // There probably is a better type for this???
    #[builder(default, setter(strip_option))]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub jws: Option<String>,
    // Better type for this as well?
    #[builder(default, setter(strip_option))]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub sha256: Option<String>,
    #[serde(flatten)]
    pub content: AttachmentType,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "lowercase")]
pub enum AttachmentType {
    // Base64 encoded string
    Base64(String),
    // A valid JSON value
    Json(Value),
    // An URL list
    Links(Vec<Url>),
}

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq, TypedBuilder)]
#[serde(rename_all = "snake_case")]
pub struct AttachmentFormatSpecifier<F> {
    pub attach_id: String,
    pub format: MaybeKnown<F>,
}

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq, TypedBuilder)]
#[serde(rename_all = "snake_case")]
pub struct OptionalIdAttachmentFormatSpecifier<F> {
    #[builder(default)]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub attach_id: Option<String>,
    pub format: MaybeKnown<F>,
}

#[cfg(test)]
#[allow(clippy::unwrap_used)]
#[allow(clippy::field_reassign_with_default)]
pub mod tests {
    use serde_json::json;

    use super::*;
    use crate::misc::test_utils;

    pub fn make_minimal_attachment() -> Attachment {
        let data = json!({
            "field": "test_json_data"
        });

        let content = AttachmentType::Json(data);
        let attach_data = AttachmentData::builder().content(content).build();
        Attachment::builder().data(attach_data).build()
    }

    pub fn make_extended_attachment() -> Attachment {
        let data = json!({
            "field": "test_json_data"
        });

        let content = AttachmentType::Json(data);
        let attach_data = AttachmentData::builder().content(content).build();
        Attachment::builder()
            .data(attach_data)
            .id("test_id".to_owned())
            .description("test_description".to_owned())
            .filename("test_filename".to_owned())
            .mime_type(MimeType::Json)
            .lastmod_time(DateTime::<Utc>::default())
            .byte_count(64)
            .build()
    }

    #[test]
    fn test_base64_attach_type() {
        let data = "test_base64_str".to_owned();

        let expected = json!({ "base64": data });
        let attach_type = AttachmentType::Base64(data);

        test_utils::test_serde(attach_type, expected);
    }

    #[test]
    fn test_json_attach_type() {
        let data = json!({
            "field": "test_json_data"
        });

        let expected = json!({ "json": data });
        let attach_type = AttachmentType::Json(data);

        test_utils::test_serde(attach_type, expected);
    }

    #[test]
    fn test_links_attach_type() {
        let data = vec!["https://dummy.dummy/dummy".parse().unwrap()];

        let expected = json!({ "links": data });
        let attach_type = AttachmentType::Links(data);

        test_utils::test_serde(attach_type, expected);
    }

    #[test]
    fn test_minimal_attach_data() {
        let data = json!({
            "field": "test_json_data"
        });

        let expected = json!({ "json": data });

        let content = AttachmentType::Json(data);
        let attach_data = AttachmentData::builder().content(content).build();

        test_utils::test_serde(attach_data, expected);
    }

    #[test]
    fn test_extended_attach_data() {
        let jws = "test_jws".to_owned();
        let sha256 = "test_sha256".to_owned();

        let data = json!({
            "field": "test_json_data"
        });

        let expected = json!({
            "json": data,
            "jws": jws,
            "sha256": sha256
        });

        let content = AttachmentType::Json(data);
        let attach_data = AttachmentData::builder()
            .content(content)
            .jws(jws)
            .sha256(sha256)
            .build();

        test_utils::test_serde(attach_data, expected);
    }

    #[test]
    fn test_minimal_attachment() {
        let attachment = make_minimal_attachment();
        let expected = json!({
            "data": attachment.data
        });

        test_utils::test_serde(attachment, expected);
    }

    #[test]
    fn test_extended_attachment() {
        let attachment = make_extended_attachment();

        let expected = json!({
            "@id": attachment.id,
            "description": attachment.description,
            "filename": attachment.filename,
            "mime-type": attachment.mime_type,
            "lastmod_time": attachment.lastmod_time,
            "byte_count": attachment.byte_count,
            "data": attachment.data
        });

        test_utils::test_serde(attachment, expected);
    }
}
