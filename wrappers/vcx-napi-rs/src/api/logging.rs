use napi_derive::napi;

use crate::error::ariesvcx_to_napi_err;

use std::env;
use std::io::Write;

use chrono::format::{DelayedFormat, StrftimeItems};
use chrono::Local;

use env_logger::fmt::Formatter;
use env_logger::Builder as EnvLoggerBuilder;
use log::{LevelFilter, Record};

pub struct Logger;

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

impl Logger {
    pub fn init(pattern: Option<String>) -> napi::Result<()> {
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
            .map_err(|err| napi::Error::new(napi::Status::GenericFailure, format!("Cannot init logger: {:?}", err)))?;
        Ok(())
    }
}

#[napi]
pub fn init_default_logger(pattern: Option<String>) -> napi::Result<()> {
    Logger::init(pattern)
}
