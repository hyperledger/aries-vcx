use std::collections::HashMap;

use serde::{Deserialize, Serialize};

use crate::decorators::{Thread, Timing};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProblemReport {
    #[serde(rename = "@id")]
    pub id: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    #[serde(rename = "~thread")]
    pub thread: Option<Thread>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<Description>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub who_retries: Option<WhoRetries>,
    #[serde(rename = "tracking-uri")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tracking_uri: Option<String>,
    #[serde(rename = "escalation-uri")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub escalation_uri: Option<String>,
    #[serde(rename = "fix-hint")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub fix_hint: Option<FixHint>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub impact: Option<Impact>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub noticed_time: Option<String>,
    #[serde(rename = "where")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub location: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub problem_items: Option<HashMap<String, String>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub comment: Option<String>,
    #[serde(rename = "~timing")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub timing: Option<Timing>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Description {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub en: Option<String>,
    pub code: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum WhoRetries {
    Me,
    You,
    Both,
    None,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FixHint {
    en: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum Impact {
    Message,
    Thread,
    Connection,
}
