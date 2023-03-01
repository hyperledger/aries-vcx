use serde::{Deserialize, Serialize};

use super::EmptyDecorator;

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

impl EmptyDecorator for Timing {
    fn is_empty(&self) -> bool {
        self.in_time.is_none()
            && self.out_time.is_none()
            && self.stale_time.is_none()
            && self.expires_time.is_none()
            && self.delay_milli.is_none()
            && self.wait_until_time.is_none()
    }
}
