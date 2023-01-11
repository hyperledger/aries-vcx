use serde::{Deserialize, Serialize};
use std::collections::HashMap;

const BUCKET_COUNT: usize = 16;
pub(super) struct MetricsFloatValue(f64);

impl ToString for MetricsFloatValue {
    fn to_string(&self) -> String {
        if self.0 == f64::MAX {
            "+Inf".to_owned()
        } else {
            self.0.to_string()
        }
    }
}

impl From<f64> for MetricsFloatValue {
    fn from(value: f64) -> Self {
        MetricsFloatValue(value)
    }
}

pub(super) const LIST_LE: [MetricsFloatValue; BUCKET_COUNT] = [MetricsFloatValue(0.5), MetricsFloatValue(1.0), MetricsFloatValue(2.0), MetricsFloatValue(5.0), MetricsFloatValue(10.0), MetricsFloatValue(20.0), MetricsFloatValue(50.0), MetricsFloatValue(100.0), MetricsFloatValue(200.0), MetricsFloatValue(500.0), MetricsFloatValue(1000.0), MetricsFloatValue(2000.0), MetricsFloatValue(5000.0), MetricsFloatValue(10000.0), MetricsFloatValue(20000.0), MetricsFloatValue(f64::MAX)];

#[derive(Serialize, Deserialize)]
pub struct MetricsValue {
    value: usize,
    tags: HashMap<String, String>,
}

impl MetricsValue {
    pub fn new(value: usize, tags: HashMap<String, String>) -> Self {
        MetricsValue { value, tags }
    }
}

#[derive(Serialize, Deserialize, Copy, Clone, Debug)]
pub struct CommandCounters {
    pub count: u128,
    pub duration_ms_sum: u128,
    pub duration_ms_bucket: [u128; BUCKET_COUNT],
}

impl CommandCounters {
    pub fn new() -> Self {
        CommandCounters {count: 0, duration_ms_sum: 0, duration_ms_bucket: [0; BUCKET_COUNT]}
    }

    pub fn add(&mut self, duration: u128) {
        self.count += 1;
        self.duration_ms_sum += duration;
        self.add_buckets(duration);
    }

    fn add_buckets(&mut self, duration: u128) {
        for (le_index, le_value) in LIST_LE.iter().enumerate() {
            if duration <= (le_value.0 as u128) {
                self.duration_ms_bucket[le_index] += 1;
            }
        }
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_counters_are_initialized_as_zeros() {
        let command_counters = CommandCounters::new();
        assert_eq!(command_counters.count, 0);
        assert_eq!(command_counters.duration_ms_sum, 0);
        assert_eq!(command_counters.duration_ms_bucket, [0; BUCKET_COUNT]);
    }
}
