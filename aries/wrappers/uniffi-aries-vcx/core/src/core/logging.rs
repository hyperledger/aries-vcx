use android_logger::Config;
use log::{debug, LevelFilter};

pub fn enable_logging() {
    android_logger::init_once(Config::default().with_max_level(LevelFilter::Trace));
    debug!("this is a debug {}", "message");
}
