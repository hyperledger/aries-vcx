#[cfg(target_os = "android")]
extern crate android_logger;

#[cfg(feature = "ffi_api")]
use env_logger::Builder as EnvLoggerBuilder;
#[cfg(feature = "ffi_api")]
use log::{Level, LevelFilter, Metadata, Record};

#[cfg(feature = "ffi_api")]
use libc::{c_char, c_void};
#[cfg(feature = "ffi_api")]
use std::{env, ffi::CString, io::Write, ptr};

#[cfg(target_os = "android")]
use self::android_logger::Filter;

#[cfg(feature = "ffi_api")]
use indy_api_types::errors::prelude::*;
#[cfg(feature = "ffi_api")]
use indy_utils::ctypes;

#[cfg(feature = "ffi_api")]
use indy_api_types::errors::IndyErrorKind::InvalidStructure;

#[cfg(feature = "ffi_api")]
pub static mut LOGGER_STATE: LoggerState = LoggerState::Default;

#[cfg(feature = "ffi_api")]
pub enum LoggerState {
    Default,
    Custom,
}

#[cfg(feature = "ffi_api")]
impl LoggerState {
    pub fn get(
        &self,
    ) -> (
        *const c_void,
        Option<EnabledCB>,
        Option<LogCB>,
        Option<FlushCB>,
    ) {
        match self {
            LoggerState::Default => (
                ptr::null(),
                Some(LibvdrtoolsDefaultLogger::enabled),
                Some(LibvdrtoolsDefaultLogger::log),
                Some(LibvdrtoolsDefaultLogger::flush),
            ),
            LoggerState::Custom => unsafe { (CONTEXT, ENABLED_CB, LOG_CB, FLUSH_CB) },
        }
    }
}

#[cfg(feature = "ffi_api")]
pub type EnabledCB =
    extern "C" fn(context: *const c_void, level: u32, target: *const c_char) -> bool;

#[cfg(feature = "ffi_api")]
pub type LogCB = extern "C" fn(
    context: *const c_void,
    level: u32,
    target: *const c_char,
    message: *const c_char,
    module_path: *const c_char,
    file: *const c_char,
    line: u32,
);

#[cfg(feature = "ffi_api")]
pub type FlushCB = extern "C" fn(context: *const c_void);

#[cfg(feature = "ffi_api")]
static mut CONTEXT: *const c_void = ptr::null();
#[cfg(feature = "ffi_api")]
static mut ENABLED_CB: Option<EnabledCB> = None;
#[cfg(feature = "ffi_api")]
static mut LOG_CB: Option<LogCB> = None;
#[cfg(feature = "ffi_api")]
static mut FLUSH_CB: Option<FlushCB> = None;

#[cfg(feature = "ffi_api")]
#[cfg(debug_assertions)]
const DEFAULT_MAX_LEVEL: LevelFilter = LevelFilter::Trace;

#[cfg(feature = "ffi_api")]
#[cfg(not(debug_assertions))]
const DEFAULT_MAX_LEVEL: LevelFilter = LevelFilter::Info;

#[cfg(feature = "ffi_api")]
pub struct LibvdrtoolsLogger {
    context: *const c_void,
    enabled: Option<EnabledCB>,
    log: LogCB,
    flush: Option<FlushCB>,
}

#[cfg(feature = "ffi_api")]
impl LibvdrtoolsLogger {
    fn new(
        context: *const c_void,
        enabled: Option<EnabledCB>,
        log: LogCB,
        flush: Option<FlushCB>,
    ) -> Self {
        LibvdrtoolsLogger {
            context,
            enabled,
            log,
            flush,
        }
    }
}

#[cfg(feature = "ffi_api")]
impl log::Log for LibvdrtoolsLogger {
    fn enabled(&self, metadata: &Metadata) -> bool {
        if let Some(enabled_cb) = self.enabled {
            let level = metadata.level() as u32;
            let target = CString::new(metadata.target()).unwrap();

            enabled_cb(self.context, level, target.as_ptr())
        } else {
            true
        }
    }

    fn log(&self, record: &Record) {
        let log_cb = self.log;

        let level = record.level() as u32;
        let target = CString::new(record.target()).unwrap();
        let message = CString::new(record.args().to_string()).unwrap();

        let module_path = record.module_path().map(|a| CString::new(a).unwrap());
        let file = record.file().map(|a| CString::new(a).unwrap());
        let line = record.line().unwrap_or(0);

        log_cb(
            self.context,
            level,
            target.as_ptr(),
            message.as_ptr(),
            module_path
                .as_ref()
                .map(|p| p.as_ptr())
                .unwrap_or(ptr::null()),
            file.as_ref().map(|p| p.as_ptr()).unwrap_or(ptr::null()),
            line,
        )
    }

    fn flush(&self) {
        if let Some(flush_cb) = self.flush {
            flush_cb(self.context)
        }
    }
}

#[cfg(feature = "ffi_api")]
unsafe impl Sync for LibvdrtoolsLogger {}

#[cfg(feature = "ffi_api")]
unsafe impl Send for LibvdrtoolsLogger {}

#[cfg(feature = "ffi_api")]
impl LibvdrtoolsLogger {
    pub fn init(
        context: *const c_void,
        enabled: Option<EnabledCB>,
        log: LogCB,
        flush: Option<FlushCB>,
        max_lvl: Option<u32>,
    ) -> Result<(), IndyError> {
        let logger = LibvdrtoolsLogger::new(context, enabled, log, flush);

        log::set_boxed_logger(Box::new(logger))?;
        let max_lvl = match max_lvl {
            Some(max_lvl) => LibvdrtoolsLogger::map_u32_lvl_to_filter(max_lvl)?,
            None => DEFAULT_MAX_LEVEL,
        };
        log::set_max_level(max_lvl);

        unsafe {
            LOGGER_STATE = LoggerState::Custom;
            CONTEXT = context;
            ENABLED_CB = enabled;
            LOG_CB = Some(log);
            FLUSH_CB = flush
        };

        Ok(())
    }

    fn map_u32_lvl_to_filter(max_level: u32) -> IndyResult<LevelFilter> {
        let max_level = match max_level {
            0 => LevelFilter::Off,
            1 => LevelFilter::Error,
            2 => LevelFilter::Warn,
            3 => LevelFilter::Info,
            4 => LevelFilter::Debug,
            5 => LevelFilter::Trace,
            _ => return Err(IndyError::from(InvalidStructure)),
        };
        Ok(max_level)
    }

    pub fn set_max_level(max_level: u32) -> IndyResult<LevelFilter> {
        let max_level_filter = LibvdrtoolsLogger::map_u32_lvl_to_filter(max_level)?;

        log::set_max_level(max_level_filter);

        Ok(max_level_filter)
    }
}

#[cfg(feature = "ffi_api")]
pub struct LibvdrtoolsDefaultLogger;

#[cfg(feature = "ffi_api")]
impl LibvdrtoolsDefaultLogger {
    pub fn init(pattern: Option<String>) -> Result<(), IndyError> {
        let pattern = pattern.or_else(|| env::var("RUST_LOG").ok());

        log_panics::init(); // Logging of panics is essential for android. As android does not log to stdout for native code

        if cfg!(target_os = "android") {
            #[cfg(target_os = "android")]
            let log_filter = match pattern {
                Some(val) => match val.to_lowercase().as_ref() {
                    "error" => Filter::default().with_min_level(log::Level::Error),
                    "warn" => Filter::default().with_min_level(log::Level::Warn),
                    "info" => Filter::default().with_min_level(log::Level::Info),
                    "debug" => Filter::default().with_min_level(log::Level::Debug),
                    "trace" => Filter::default().with_min_level(log::Level::Trace),
                    _ => Filter::default().with_min_level(log::Level::Error),
                },
                None => Filter::default().with_min_level(log::Level::Error),
            };

            //Set logging to off when deploying production android app.
            #[cfg(target_os = "android")]
            android_logger::init_once(log_filter);
            info!("Logging for Android");
        } else {
            EnvLoggerBuilder::new()
                .format(|buf, record| {
                    writeln!(
                        buf,
                        "{:>5}|{:<30}|{:>35}:{:<4}| {}",
                        record.level(),
                        record.target(),
                        record.file().get_or_insert(""),
                        record.line().get_or_insert(0),
                        record.args()
                    )
                })
                .filter(None, LevelFilter::Off)
                .parse_filters(pattern.as_ref().map(String::as_str).unwrap_or(""))
                .try_init()?;
        }
        unsafe { LOGGER_STATE = LoggerState::Default };
        Ok(())
    }

    extern "C" fn enabled(_context: *const c_void, level: u32, target: *const c_char) -> bool {
        let level = get_level(level);
        let target = ctypes::c_str_to_string(target).unwrap().unwrap();

        let metadata: Metadata = Metadata::builder().level(level).target(&target).build();

        log::logger().enabled(&metadata)
    }

    extern "C" fn log(
        _context: *const c_void,
        level: u32,
        target: *const c_char,
        args: *const c_char,
        module_path: *const c_char,
        file: *const c_char,
        line: u32,
    ) {
        let target = ctypes::c_str_to_string(target).unwrap().unwrap();
        let args = ctypes::c_str_to_string(args).unwrap().unwrap();
        let module_path = ctypes::c_str_to_string(module_path).unwrap();
        let file = ctypes::c_str_to_string(file).unwrap();

        let level = get_level(level);

        log::logger().log(
            &Record::builder()
                .args(format_args!("{}", args))
                .level(level)
                .target(&target)
                .module_path(module_path)
                .file(file)
                .line(Some(line))
                .build(),
        );
    }

    extern "C" fn flush(_context: *const c_void) {
        log::logger().flush()
    }
}

#[cfg(feature = "ffi_api")]
fn get_level(level: u32) -> Level {
    match level {
        1 => Level::Error,
        2 => Level::Warn,
        3 => Level::Info,
        4 => Level::Debug,
        5 => Level::Trace,
        _ => unreachable!(),
    }
}

#[macro_export]
macro_rules! try_log {
    ($expr:expr) => {
        match $expr {
            Ok(val) => val,
            Err(err) => {
                error!("try_log! | {}", err);
                return Err(From::from(err));
            }
        }
    };
}

macro_rules! _map_err {
    ($lvl:expr, $expr:expr) => {
        |err| {
            log!($lvl, "{} - {}", $expr, err);
            err
        }
    };
    ($lvl:expr) => {
        |err| {
            log!($lvl, "{}", err);
            err
        }
    };
}

#[macro_export]
macro_rules! map_err_err {
    () => ( _map_err!(::log::Level::Error) );
    ($($arg:tt)*) => ( _map_err!(::log::Level::Error, $($arg)*) )
}

#[macro_export]
macro_rules! map_err_trace {
    () => ( _map_err!(::log::Level::Trace) );
    ($($arg:tt)*) => ( _map_err!(::log::Level::Trace, $($arg)*) )
}

#[macro_export]
macro_rules! map_err_info {
    () => ( _map_err!(::log::Level::Info) );
    ($($arg:tt)*) => ( _map_err!(::log::Level::Info, $($arg)*) )
}

#[cfg(debug_assertions)]
#[macro_export]
macro_rules! secret {
    ($val:expr) => {{
        $val
    }};
}

#[cfg(not(debug_assertions))]
#[macro_export]
macro_rules! secret {
    ($val:expr) => {{
        "_"
    }};
}
