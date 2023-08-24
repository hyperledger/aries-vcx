use std::fmt::Display;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ThinState {
    #[serde(rename = "RequestSent")]
    RequestSent,
    #[serde(rename = "ResponseSent")]
    ResponseSent,
    #[serde(rename = "Completed")]
    Completed,
    #[serde(rename = "Abandoned")]
    Abandoned,
}
