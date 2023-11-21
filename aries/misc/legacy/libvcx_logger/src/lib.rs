use std::{env, io::Write, sync::Once};

#[cfg(target_os = "android")]
use android_logger::Config;
use aries_vcx_core::errors::error::{AriesVcxCoreError, AriesVcxCoreErrorKind, VcxCoreResult};
use chrono::{
    format::{DelayedFormat, StrftimeItems},
    Local,
};
use env_logger::{fmt::Formatter, Builder as EnvLoggerBuilder};
use log::{info, LevelFilter, Record};

static TEST_LOGGING_INIT: Once = Once::new();

pub fn init_test_logging() {
    TEST_LOGGING_INIT.call_once(|| {
        LibvcxDefaultLogger::init_testing_logger();
    })
}

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

impl LibvcxDefaultLogger {
    pub fn init_testing_logger() {
        if let Ok(log_pattern) = env::var("RUST_LOG") {
            LibvcxDefaultLogger::init(Some(log_pattern))
                .expect("Failed to initialize LibvcxDefaultLogger for testing")
        }
    }

    pub fn init(pattern: Option<String>) -> VcxCoreResult<()> {
        let pattern = pattern.or(env::var("RUST_LOG").ok());
        if cfg!(target_os = "android") {
            #[cfg(target_os = "android")]
            let log_filter = match pattern.as_ref() {
                Some(val) => match val.to_lowercase().as_ref() {
                    "error" => Config::default().with_max_level(log::LevelFilter::Error),
                    "warn" => Config::default().with_max_level(log::LevelFilter::Warn),
                    "info" => Config::default().with_max_level(log::LevelFilter::Info),
                    "debug" => Config::default().with_max_level(log::LevelFilter::Debug),
                    "trace" => Config::default().with_max_level(log::LevelFilter::Trace),
                    _ => Config::default().with_max_level(log::LevelFilter::Error),
                },
                None => Config::default().with_max_level(log::LevelFilter::Error),
            };

            //Set logging to off when deploying production android app.
            #[cfg(target_os = "android")]
            android_logger::init_once(log_filter);
            info!("Logging for Android");
        } else {
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
                    AriesVcxCoreError::from_msg(
                        AriesVcxCoreErrorKind::LoggingError,
                        format!("Cannot init logger: {:?}", err),
                    )
                })?;
        }
        Ok(())
    }
}
