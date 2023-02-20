use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use serde_json::value::RawValue;
use url::Url;

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub struct Attachment {
    #[serde(skip_serializing_if = "Option::is_none")]
    #[serde(rename = "@id")]
    id: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    description: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    filename: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    #[serde(rename = "mime-type")]
    mime_type: Option<MimeType>,
    #[serde(skip_serializing_if = "Option::is_none")]
    lastmod_time: Option<DateTime<Utc>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    byte_count: Option<u64>,
    data: AttachmentData,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "mime-type")]
pub enum MimeType {
    #[serde(rename = "application/json")]
    Json,
    #[serde(rename = "image/jpg")]
    Jpg,
    #[serde(rename = "image/jpeg")]
    Jpeg,
    #[serde(rename = "image/png")]
    Png,
    #[serde(rename = "application/pdf")]
    Pdf,
    #[serde(rename = "text/plain")]
    Plain,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct AttachmentData {
    // There probably is a better type for this???
    jws: Option<String>,
    // Better type for this as well?
    sha256: Option<String>,
    #[serde(flatten)]
    content: AttachmentType,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum AttachmentType {
    Base64(String),
    Json(Box<RawValue>),
    Links(Vec<Url>),
}
