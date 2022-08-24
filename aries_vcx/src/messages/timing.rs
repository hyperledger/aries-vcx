use std::collections::HashMap;

use chrono::{DateTime, NaiveDateTime, SecondsFormat, TimeZone, Utc};

#[derive(Debug, Deserialize, Serialize, PartialEq, Clone)]
pub struct Timing {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub out_time: Option<String>,
}

pub fn as_iso8601<Tz: TimeZone>(date_time: DateTime<Tz>) -> String
where
    Tz::Offset: std::fmt::Display,
{
    date_time.to_rfc3339_opts(SecondsFormat::Millis, true)
}

impl Timing {
    pub fn new() -> Timing {
        Timing::default()
    }

    pub fn set_time<Tz: TimeZone>(mut self, date_time: DateTime<Tz>) -> Timing
    where
        Tz::Offset: std::fmt::Display,
    {
        self.out_time = Some(as_iso8601(date_time));
        self
    }

    pub fn set_out_time_to_now(mut self) -> Timing {
        self.set_time(Utc::now())
    }

    pub fn get_time_as_iso8601_string(&self) -> Option<&str> {
        self.out_time.as_deref()
    }
}

impl Default for Timing {
    fn default() -> Timing {
        Timing { out_time: None }
    }
}

#[macro_export]
macro_rules! timing_optional (($type:ident) => (
    impl $type {
        pub fn set_out_time(mut self) -> Self {
            self.timing = Some(Timing::new().set_out_time_to_now());
            self
        }
    }
));

#[cfg(test)]
#[cfg(feature = "general_test")]
pub mod unit_tests {
    use crate::messages::connection::response::test_utils::*;
    use std::str::FromStr;
    use std::thread;
    use std::time::Duration;

    use super::*;

    #[test]
    fn test_get_time_as_iso8601_string_with_milliseconds() {
        let dt: DateTime<Utc> = DateTime::parse_from_rfc3339("2020-01-01T00:00:00Z").unwrap().into();
        let timing = Timing::new().set_time(dt);
        assert_eq!(timing.get_time_as_iso8601_string(), Some("2020-01-01T00:00:00.000Z"));
    }

    #[test]
    fn test_sets_gets_current_time() {
        let t1 = Utc::now();
        thread::sleep(Duration::from_millis(10));
        let timing = Timing::new().set_out_time_to_now();
        thread::sleep(Duration::from_millis(10));
        let t2 = Utc::now();

        let t: DateTime<Utc> = DateTime::parse_from_rfc3339(timing.get_time_as_iso8601_string().unwrap())
            .unwrap()
            .into();
        assert!(t1 < t);
        assert!(t < t2);
    }
}
