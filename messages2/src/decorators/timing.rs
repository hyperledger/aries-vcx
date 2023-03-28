use crate::misc::utils;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

/// Struct representing the `~timing` decorator from its [RFC](<https://github.com/hyperledger/aries-rfcs/blob/main/features/0032-message-timing/README.md>).
#[derive(Debug, Deserialize, Serialize, Clone, Default, PartialEq)]
pub struct Timing {
    #[serde(skip_serializing_if = "Option::is_none")]
    #[serde(serialize_with = "utils::serialize_opt_datetime")]
    pub in_time: Option<DateTime<Utc>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    #[serde(serialize_with = "utils::serialize_opt_datetime")]
    pub out_time: Option<DateTime<Utc>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    #[serde(serialize_with = "utils::serialize_opt_datetime")]
    pub stale_time: Option<DateTime<Utc>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    #[serde(serialize_with = "utils::serialize_opt_datetime")]
    pub expires_time: Option<DateTime<Utc>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub delay_milli: Option<u32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    #[serde(serialize_with = "utils::serialize_opt_datetime")]
    pub wait_until_time: Option<DateTime<Utc>>,
}

#[cfg(test)]
#[allow(clippy::unwrap_used)]
#[allow(clippy::field_reassign_with_default)]
pub mod tests {
    use serde_json::json;

    use super::*;
    use crate::misc::test_utils::{self, OptDateTimeRfc3339};

    pub fn make_minimal_timing() -> Timing {
        Timing::default()
    }

    pub fn make_extended_timing() -> Timing {
        let dt = DateTime::default();
        let delay_milli = 10;

        let mut timing = Timing::default();
        timing.in_time = Some(dt);
        timing.out_time = Some(dt);
        timing.stale_time = Some(dt);
        timing.expires_time = Some(dt);
        timing.delay_milli = Some(delay_milli);
        timing.wait_until_time = Some(dt);

        timing
    }

    #[test]
    fn test_minimal_timing() {
        let timing = make_minimal_timing();
        let expected = json!({});

        test_utils::test_serde(timing, expected);
    }

    #[test]
    fn test_extended_timing() {
        let timing = make_extended_timing();

        let expected = json!({
            "in_time": OptDateTimeRfc3339(&timing.in_time),
            "out_time": OptDateTimeRfc3339(&timing.out_time),
            "stale_time": OptDateTimeRfc3339(&timing.stale_time),
            "expires_time": OptDateTimeRfc3339(&timing.expires_time),
            "delay_milli": timing.delay_milli,
            "wait_until_time": OptDateTimeRfc3339(&timing.wait_until_time)
        });

        test_utils::test_serde(timing, expected);
    }
}
