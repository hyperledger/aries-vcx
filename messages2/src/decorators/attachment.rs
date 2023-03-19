use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use url::Url;

use crate::misc::mime_type::MimeType;

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
    Base64(String),
    Json(Value),
    Links(Vec<Url>),
}

#[cfg(test)]
#[allow(clippy::unwrap_used)]
#[allow(clippy::field_reassign_with_default)]
mod tests {
    use serde_json::json;

    use super::*;
    use crate::misc::test_utils;

    #[test]
    fn test_base64_attach_type() {
        let data = "test_base64_str".to_owned();

        let json = json!({ "base64": data });
        let attach_type = AttachmentType::Base64(data);

        test_utils::test_serde(attach_type, json);
    }

    #[test]
    fn test_json_attach_type() {
        let data = json!({
            "field": "test_json_data"
        });

        let json = json!({ "json": data });
        let attach_type = AttachmentType::Json(data);

        test_utils::test_serde(attach_type, json);
    }

    #[test]
    fn test_links_attach_type() {
        let data = vec!["https://dummy.dummy/dummy".parse().unwrap()];

        let json = json!({ "links": data });
        let attach_type = AttachmentType::Links(data);

        test_utils::test_serde(attach_type, json);
    }

    #[test]
    fn test_minimal_attach_data() {
        let data = json!({
            "field": "test_json_data"
        });

        let json = json!({ "json": data });

        let content = AttachmentType::Json(data);
        let attach_data = AttachmentData::new(content);

        test_utils::test_serde(attach_data, json);
    }

    #[test]
    fn test_extensive_attach_data() {
        let jws = "test_jws".to_owned();
        let sha256 = "test_sha256".to_owned();

        let data = json!({
            "field": "test_json_data"
        });

        let json = json!({
            "json": data,
            "jws": jws,
            "sha256": sha256
        });

        let content = AttachmentType::Json(data);
        let mut attach_data = AttachmentData::new(content);
        attach_data.jws = Some(jws);
        attach_data.sha256 = Some(sha256);

        test_utils::test_serde(attach_data, json);
    }
}
