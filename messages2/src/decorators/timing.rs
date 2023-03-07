use serde::{Deserialize, Serialize};

#[derive(Default, Debug, Deserialize, Serialize, PartialEq, Clone)]
pub struct Timing {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub in_time: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub out_time: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub stale_time: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub expires_time: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub delay_milli: Option<u32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub wait_until_time: Option<String>,
}
