#[cfg(target_os = "android")]
extern crate android_logger;
extern crate env_logger;
extern crate indy_sys;
extern crate log;

use std::env;
use std::io::Write;

use crate::error::prelude::*;
use crate::libindy;

#[allow(unused_imports)]
#[cfg(target_os = "android")]
use self::android_logger::Filter;
use self::env_logger::Builder as EnvLoggerBuilder;
use self::env_logger::fmt::Formatter;
use self::log::{LevelFilter, Record};

pub struct LibvcxDefaultLogger;

fn standard_format(buf: &mut Formatter, record: &Record) -> std::io::Result<()> {
    writeln!(buf, "{:>5}|{:<30}|{:>35}:{:<4}| {}", record.level(), record.target(), record.file().get_or_insert(""), record.line().get_or_insert(0), record.args())
}

fn json_format(buf: &mut Formatter, record: &Record) -> std::io::Result<()> {
    writeln!(buf, "{{\"level\":\"{}\",\"filename\":\"{}\",message:\"{}\"}}", record.level(), record.file().get_or_insert(""), record.args())
}

impl LibvcxDefaultLogger {
    pub fn init_testing_logger() {
        trace!("LibvcxDefaultLogger::init_testing_logger >>>");

        env::var("RUST_LOG")
            .map_or((), |log_pattern| LibvcxDefaultLogger::init(Some(log_pattern)).unwrap())
    }

    pub fn init(pattern: Option<String>) -> VcxResult<()> {
        info!("LibvcxDefaultLogger::init >>> pattern: {:?}", pattern);

        let pattern = pattern.or(env::var("RUST_LOG").ok());
        if cfg!(target_os = "android") {
            #[cfg(target_os = "android")]
                let log_filter = match pattern.as_ref() {
                Some(val) => match val.to_lowercase().as_ref() {
                    "error" => Filter::default().with_min_level(log::Level::Error),
                    "warn" => Filter::default().with_min_level(log::Level::Warn),
                    "info" => Filter::default().with_min_level(log::Level::Info),
                    "debug" => Filter::default().with_min_level(log::Level::Debug),
                    "trace" => Filter::default().with_min_level(log::Level::Trace),
                    _ => Filter::default().with_min_level(log::Level::Error),
                },
                None => Filter::default().with_min_level(log::Level::Error)
            };

            //Set logging to off when deploying production android app.
            #[cfg(target_os = "android")]
                android_logger::init_once(log_filter);
            info!("Logging for Android");
        } else {
            let formatter = match env::var("RUST_LOG_FORMATTER") {
                Ok(val) => match val.as_str() {
                    "json" => json_format,
                    _ => standard_format
                }
                _ => standard_format
            };
            EnvLoggerBuilder::new()
                .format(formatter)
                .filter(None, LevelFilter::Off)
                .parse_filters(pattern.as_ref().map(String::as_str).unwrap_or("warn"))
                .try_init()
                .map_err(|err| VcxError::from_msg(VcxErrorKind::LoggingError, format!("Cannot init logger: {:?}", err)))?;
        }
        libindy::utils::logger::set_default_logger(pattern.as_ref().map(String::as_str))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    #[cfg(feature = "general_test")]
    fn test_logger_for_testing() {
        LibvcxDefaultLogger::init_testing_logger();
    }
}
