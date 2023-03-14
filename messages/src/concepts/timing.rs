use chrono::{DateTime, SecondsFormat, TimeZone, Utc};

#[derive(Default, Debug, Deserialize, Serialize, PartialEq, Clone)]
pub struct Timing {
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

    pub fn set_out_time<Tz: TimeZone>(mut self, date_time: DateTime<Tz>) -> Timing
    where
        Tz::Offset: std::fmt::Display,
    {
        self.out_time = Some(as_iso8601(date_time));
        self
    }

    pub fn set_stale_time<Tz: TimeZone>(mut self, date_time: DateTime<Tz>) -> Timing
    where
        Tz::Offset: std::fmt::Display,
    {
        self.stale_time = Some(as_iso8601(date_time));
        self
    }

    pub fn set_expires_time<Tz: TimeZone>(mut self, date_time: DateTime<Tz>) -> Timing
    where
        Tz::Offset: std::fmt::Display,
    {
        self.expires_time = Some(as_iso8601(date_time));
        self
    }

    pub fn set_delay_milli(mut self, delay_milli: u32) -> Timing {
        self.delay_milli = Some(delay_milli);
        self
    }

    pub fn set_wait_until_time<Tz: TimeZone>(mut self, date_time: DateTime<Tz>) -> Timing
    where
        Tz::Offset: std::fmt::Display,
    {
        self.wait_until_time = Some(as_iso8601(date_time));
        self
    }

    pub fn set_out_time_to_now(self) -> Timing {
        self.set_out_time(Utc::now())
    }

    pub fn get_out_time(&self) -> Option<&str> {
        self.out_time.as_deref()
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

    use std::{thread, time::Duration};

    use super::*;

    #[test]
    fn test_timing_serialize_deserialize() {
        let dt1: DateTime<Utc> = DateTime::parse_from_rfc3339("2020-01-01T00:01:00Z").unwrap().into();
        let dt2: DateTime<Utc> = DateTime::parse_from_rfc3339("2020-02-01T05:01:11Z").unwrap().into();
        let dt3: DateTime<Utc> = DateTime::parse_from_rfc3339("2020-03-01T05:01:22Z").unwrap().into();
        let dt4: DateTime<Utc> = DateTime::parse_from_rfc3339("2020-04-01T05:01:33Z").unwrap().into();
        let timing = Timing::new()
            .set_out_time(dt1)
            .set_stale_time(dt2)
            .set_expires_time(dt3)
            .set_delay_milli(2000 as u32)
            .set_wait_until_time(dt4);
        let serialized = serde_json::to_string(&timing).unwrap();
        let deserialized: Timing = serde_json::from_str(&serialized).unwrap();
        assert_eq!(timing, deserialized);
    }

    #[test]
    fn test_get_time_as_iso8601_string_with_milliseconds() {
        let dt: DateTime<Utc> = DateTime::parse_from_rfc3339("2020-01-01T00:00:00Z").unwrap().into();
        let timing = Timing::new().set_out_time(dt);
        assert_eq!(timing.get_out_time(), Some("2020-01-01T00:00:00.000Z"));
    }

    #[test]
    fn test_sets_gets_current_time() {
        let t1 = Utc::now();
        thread::sleep(Duration::from_millis(10));
        let timing = Timing::new().set_out_time_to_now();
        thread::sleep(Duration::from_millis(10));
        let t2 = Utc::now();

        let t: DateTime<Utc> = DateTime::parse_from_rfc3339(timing.get_out_time().unwrap())
            .unwrap()
            .into();
        assert!(t1 < t);
        assert!(t < t2);
    }
}
