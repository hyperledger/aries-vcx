use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
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
    #[serde(rename = "text/string")]
    String,
    #[serde(rename = "didcomm/aip1")]
    Aip1,
    #[serde(rename = "didcomm/aip2;env=rfc19")]
    Aip2Rfc19,
    #[serde(rename = "didcomm/aip2;env=rfc587")]
    Aip2Rfc587,
    #[serde(rename = "didcomm/v2")]
    DidcommV2,
}
