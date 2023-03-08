use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use serde_json::value::RawValue;
use url::Url;

use crate::mime_type::MimeType;

#[derive(Debug, Clone, Serialize, Deserialize)]
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

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct AttachmentData {
    // There probably is a better type for this???
    pub jws: Option<String>,
    // Better type for this as well?
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

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum AttachmentType {
    Base64(String),
    Json(Box<RawValue>),
    Links(Vec<Url>),
}
