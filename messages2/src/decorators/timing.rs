use serde::{Deserialize, Serialize};

#[derive(Debug, Deserialize, Serialize, Clone, Default, PartialEq)]
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

#[cfg(test)]
#[allow(clippy::unwrap_used)]
#[allow(clippy::field_reassign_with_default)]
pub mod tests {
    use serde_json::json;

    use super::*;
    use crate::misc::test_utils;

    pub fn make_minimal_timing() -> Timing {
        Timing::default()
    }

    pub fn make_extended_timing() -> Timing {
        let in_time = "test_in_time".to_owned();
        let out_time = "test_out_time".to_owned();
        let stale_time = "test_stale_time".to_owned();
        let expires_time = "test_expires_time".to_owned();
        let delay_milli = 10;
        let wait_until_time = "test_wait_until_time".to_owned();

        let mut timing = Timing::default();
        timing.in_time = Some(in_time);
        timing.out_time = Some(out_time);
        timing.stale_time = Some(stale_time);
        timing.expires_time = Some(expires_time);
        timing.delay_milli = Some(delay_milli);
        timing.wait_until_time = Some(wait_until_time);

        timing
    }

    #[test]
    fn test_minimal_timing() {
        let timing = make_minimal_timing();
        let json = json!({});

        test_utils::test_serde(timing, json);
    }

    #[test]
    fn test_extensive_timing() {
        let timing = make_extended_timing();

        let json = json!({
            "in_time": timing.in_time,
            "out_time": timing.out_time,
            "stale_time": timing.stale_time,
            "expires_time": timing.expires_time,
            "delay_milli": timing.delay_milli,
            "wait_until_time": timing.wait_until_time
        });

        test_utils::test_serde(timing, json);
    }
}
