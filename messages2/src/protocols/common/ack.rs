use serde::{Deserialize, Serialize};

use crate::decorators::{Thread, Timing};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Ack {
    #[serde(rename = "@id")]
    pub id: String,
    pub status: AckStatus,
    #[serde(rename = "~thread")]
    pub thread: Thread,
    #[serde(rename = "~timing")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub timing: Option<Timing>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum AckStatus {
    #[serde(rename = "OK")]
    Ok,
    #[serde(rename = "FAIL")]
    Fail,
    #[serde(rename = "PENDING")]
    Pending,
}
