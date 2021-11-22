use std::time::Duration;

pub struct TimeoutUtils {}

impl TimeoutUtils {
    pub fn long_timeout() -> Duration {
        Duration::from_secs(50)
    }
}
