use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use serde_json::value::RawValue;
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

impl PartialEq for AttachmentType {
    fn eq(&self, other: &Self) -> bool {
        match (self, other) {
            (Self::Base64(l0), Self::Base64(r0)) => l0 == r0,
            (Self::Json(l0), Self::Json(r0)) => l0.get() == r0.get(),
            (Self::Links(l0), Self::Links(r0)) => l0 == r0,
            _ => false,
        }
    }
}
