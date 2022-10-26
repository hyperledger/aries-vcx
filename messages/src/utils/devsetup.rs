use std::sync::Once;

use crate::utils::test_logger::LibvcxDefaultLogger;
#[cfg(feature = "test_utils")]
use chrono::{DateTime, Duration, Utc};

lazy_static! {
    static ref TEST_LOGGING_INIT: Once = Once::new();
}

pub fn init_test_logging() {
    TEST_LOGGING_INIT.call_once(|| {
        LibvcxDefaultLogger::init_testing_logger();
    })
}

pub struct SetupEmpty;

impl SetupEmpty {
    pub fn init() -> SetupEmpty {
        init_test_logging();
        SetupEmpty {}
    }
}

#[cfg(feature = "test_utils")]
pub fn was_in_past(datetime_rfc3339: &str, threshold: Duration) -> chrono::ParseResult<bool> {
    let now = Utc::now();
    let datetime: DateTime<Utc> = DateTime::parse_from_rfc3339(datetime_rfc3339)?.into();
    let diff = now - datetime;
    Ok(threshold > diff)
}
