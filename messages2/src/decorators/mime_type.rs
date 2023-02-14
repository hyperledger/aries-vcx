use serde::{Serialize, Deserialize};

#[derive(Debug, Serialize, Deserialize, PartialEq, Clone)]
pub enum MimeType {
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

impl Default for MimeType {
    fn default() -> MimeType {
        MimeType::Plain
    }
}
