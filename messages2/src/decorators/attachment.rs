use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use url::Url;

use crate::misc::mime_type::MimeType;

/// Struct representing the `~attach` decorator from its [RFC](https://github.com/hyperledger/aries-rfcs/blob/main/concepts/0017-attachments/README.md).
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "snake_case")]
pub struct Attachment {
    #[serde(skip_serializing_if = "Option::is_none")]
    #[serde(rename = "@id")]
    pub id: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub filename: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    #[serde(rename = "mime-type")]
    pub mime_type: Option<MimeType>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub lastmod_time: Option<DateTime<Utc>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub byte_count: Option<u64>,
    pub data: AttachmentData,
}

impl Attachment {
    pub fn new(data: AttachmentData) -> Self {
        Self {
            id: None,
            description: None,
            filename: None,
            mime_type: None,
            lastmod_time: None,
            byte_count: None,
            data,
        }
    }
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub struct AttachmentData {
    // There probably is a better type for this???
    #[serde(skip_serializing_if = "Option::is_none")]
    pub jws: Option<String>,
    // Better type for this as well?
    #[serde(skip_serializing_if = "Option::is_none")]
    pub sha256: Option<String>,
    #[serde(flatten)]
    pub content: AttachmentType,
}

impl AttachmentData {
    pub fn new(content: AttachmentType) -> Self {
        Self {
            jws: None,
            sha256: None,
            content,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "kebab-case")]
pub enum AttachmentType {
    // Base64 encoded bytes
    Base64(Vec<u8>),
    // A valid JSON value
    Json(Value),
    // An URL list
    Links(Vec<Url>),
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
        let attach_data = AttachmentData::new(content);
        Attachment::new(attach_data)
    }

    pub fn make_extended_attachment() -> Attachment {
        let data = json!({
            "field": "test_json_data"
        });

        let content = AttachmentType::Json(data);
        let attach_data = AttachmentData::new(content);
        let mut attachment = Attachment::new(attach_data);
        let id = "test_id".to_owned();
        let description = "test_description".to_owned();
        let filename = "test_filename".to_owned();
        let mime_type = MimeType::Json;
        let lastmod_time = DateTime::<Utc>::default();
        let byte_count = 64;

        attachment.id = Some(id);
        attachment.description = Some(description);
        attachment.filename = Some(filename);
        attachment.mime_type = Some(mime_type);
        attachment.lastmod_time = Some(lastmod_time);
        attachment.byte_count = Some(byte_count);

        attachment
    }

    #[test]
    fn test_base64_attach_type() {
        let data = "test_base64_str".to_owned().into_bytes();

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
        let attach_data = AttachmentData::new(content);

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
        let mut attach_data = AttachmentData::new(content);
        attach_data.jws = Some(jws);
        attach_data.sha256 = Some(sha256);

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
