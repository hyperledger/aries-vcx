// todo: remove text representations, and should definitely not be driven by AATH expectations
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ThinState {
    #[serde(rename = "request-sent")]
    RequestSent,
    #[serde(rename = "response-sent")]
    ResponseSent,
    #[serde(rename = "completed")]
    Completed,
    #[serde(rename = "abandoned")]
    Abandoned,
}
