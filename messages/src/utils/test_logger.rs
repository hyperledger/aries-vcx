extern crate env_logger;
extern crate log;

use std::env;
use std::io::Write;

use chrono::format::{DelayedFormat, StrftimeItems};

use crate::chrono::Local;
use crate::error::prelude::*;
use vdrtoolsrs::logger;

use self::env_logger::fmt::Formatter;
use self::env_logger::Builder as EnvLoggerBuilder;
use self::log::{LevelFilter, Record};

pub struct LibvcxDefaultLogger;

fn _get_timestamp<'a>() -> DelayedFormat<StrftimeItems<'a>> {
    Local::now().format("%Y-%m-%d %H:%M:%S.%f")
}

fn text_format(buf: &mut Formatter, record: &Record) -> std::io::Result<()> {
    let level = buf.default_styled_level(record.level());
    writeln!(
        buf,
        "{}|{:>5}|{:<30}|{:>35}:{:<4}| {}",
        _get_timestamp(),
        level,
        record.target(),
        record.file().get_or_insert(""),
        record.line().get_or_insert(0),
        record.args()
    )
}

fn text_no_color_format(buf: &mut Formatter, record: &Record) -> std::io::Result<()> {
    let level = record.level();
    writeln!(
        buf,
        "{}|{:>5}|{:<30}|{:>35}:{:<4}| {}",
        _get_timestamp(),
        level,
        record.target(),
        record.file().get_or_insert(""),
        record.line().get_or_insert(0),
        record.args()
    )
}

fn set_default_logger(patter: Option<&str>) -> MessagesResult<()> {
    logger::set_default_logger(patter).map_err(MessagesError::from)
}

impl LibvcxDefaultLogger {
    pub fn init_testing_logger() {
        trace!("LibvcxDefaultLogger::init_testing_logger >>>");

        env::var("RUST_LOG").map_or((), |log_pattern| LibvcxDefaultLogger::init(Some(log_pattern)).unwrap())
    }

    pub fn init(pattern: Option<String>) -> MessagesResult<()> {
        info!("LibvcxDefaultLogger::init >>> pattern: {:?}", pattern);
        let pattern = pattern.or(env::var("RUST_LOG").ok());
        let formatter = match env::var("RUST_LOG_FORMATTER") {
            Ok(val) => match val.as_str() {
                "text_no_color" => text_no_color_format,
                _ => text_format,
            },
            _ => text_format,
        };
        EnvLoggerBuilder::new()
            .format(formatter)
            .filter(None, LevelFilter::Off)
            .parse_filters(pattern.as_deref().unwrap_or("warn"))
            .try_init()
            .map_err(|err| {
                MessagesError::from_msg(MesssagesErrorKind::LoggingError, format!("Cannot init logger: {:?}", err))
            })?;
        set_default_logger(pattern.as_deref())
    }
}

#[cfg(test)]
#[cfg(feature = "general_test")]
mod unit_tests {
    use super::*;

    #[test]
    fn test_logger_for_testing() {
        LibvcxDefaultLogger::init_testing_logger();
    }
}
