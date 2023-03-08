use std::collections::HashMap;

use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct Thread {
    pub thid: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub pthid: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub sender_order: Option<u32>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub received_orders: Option<HashMap<String, u32>>, // should get replaced with DID.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub goal_code: Option<ThreadGoalCode>,
}

impl Thread {
    pub fn new(thid: String) -> Self {
        Self {
            thid,
            pthid: None,
            sender_order: None,
            received_orders: None,
            goal_code: None,
        }
    }
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub enum ThreadGoalCode {
    #[serde(rename = "aries.vc")]
    AriesVc,
    #[serde(rename = "aries.vc.issue")]
    AriesVcIssue,
    #[serde(rename = "aries.vc.verify")]
    AriesVcVerify,
    #[serde(rename = "aries.vc.revoke")]
    AriesVcRevoke,
    #[serde(rename = "aries.rel")]
    AriesRel,
    #[serde(rename = "aries.rel.build")]
    AriesRelBuild,
    #[serde(deserialize_with = "String::deserialize")]
    #[serde(serialize_with = "String::serialize")]
    Other(String),
}
