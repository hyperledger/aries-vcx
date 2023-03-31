use std::sync::Once;

#[cfg(feature = "test_utils")]
use chrono::{DateTime, Duration, Utc};

lazy_static! {
    static ref TEST_LOGGING_INIT: Once = Once::new();
}

#[cfg(feature = "test_utils")]
pub fn was_in_past(datetime_rfc3339: &str, threshold: Duration) -> chrono::ParseResult<bool> {
    let now = Utc::now();
    let datetime: DateTime<Utc> = DateTime::parse_from_rfc3339(datetime_rfc3339)?.into();
    let diff = now - datetime;
    Ok(threshold > diff)
}
