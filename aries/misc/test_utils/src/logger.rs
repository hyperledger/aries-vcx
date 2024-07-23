use std::{env, io::Write, sync::Once};

use chrono::{
    format::{DelayedFormat, StrftimeItems},
    Local,
};
use env_logger::{fmt::Formatter, Builder as EnvLoggerBuilder};
use log::{LevelFilter, Record};

use crate::errors::error::{TestUtilsError, TestUtilsResult};

static TEST_LOGGING_INIT: Once = Once::new();

pub fn init_logger() {
    TEST_LOGGING_INIT.call_once(|| {
        LibvcxDefaultLogger::init_testing_logger();
    })
}

pub struct LibvcxDefaultLogger;

fn _get_timestamp<'a>() -> DelayedFormat<StrftimeItems<'a>> {
    Local::now().format("%Y-%m-%d %H:%M:%S.%f")
}

fn text_format(buf: &mut Formatter, record: &Record) -> std::io::Result<()> {
    let level = buf.default_level_style(record.level());
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

impl LibvcxDefaultLogger {
    pub fn init_testing_logger() {
        if let Ok(log_pattern) = env::var("RUST_LOG") {
            LibvcxDefaultLogger::init(Some(log_pattern))
                .expect("Failed to initialize LibvcxDefaultLogger for testing")
        }
    }

    pub fn init(pattern: Option<String>) -> TestUtilsResult<()> {
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
                TestUtilsError::LoggingError(format!("Cannot init logger: {:?}", err))
            })?;
        Ok(())
    }
}
