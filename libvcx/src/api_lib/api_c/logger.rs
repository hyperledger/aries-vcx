use libc::c_char;

use aries_vcx::error::{VcxError, VcxErrorKind};
use aries_vcx::utils::error::SUCCESS;

use crate::api_lib::utils::cstring::CStringUtils;
use crate::api_lib::utils::error::set_current_error_vcx;
use crate::api_lib::utils::logger::{
    CVoid, EnabledCB, FlushCB, LibvcxDefaultLogger, LibvcxLogger, LogCB, LOGGER_STATE,
};

/// Set default logger implementation.
///
/// Allows library user use `env_logger` logger as default implementation.
/// More details about `env_logger` and its customization can be found here: https://crates.io/crates/env_logger
///
/// #Params
/// pattern: (optional) pattern that corresponds with the log messages to show.
///
/// NOTE: You should specify either `pattern` parameter or `RUST_LOG` environment variable to init logger.
///
/// #Returns
/// u32 error code
#[no_mangle]
pub extern "C" fn vcx_set_default_logger(pattern: *const c_char) -> u32 {
    info!("vcx_set_default_logger >>>");

    check_useful_opt_c_str!(pattern, VcxErrorKind::InvalidConfiguration);

    trace!("vcx_set_default_logger(pattern: {:?})", pattern);

    match LibvcxDefaultLogger::init(pattern.clone()) {
        Ok(()) => {
            info!("Logger Successfully Initialized with pattern {:?}", &pattern);
            SUCCESS.code_num
        }
        Err(err) => {
            set_current_error_vcx(&err);
            error!("Logger Failed To Initialize: {}", err);
            err.into()
        }
    }
}

/// Set custom logger implementation.
///
/// Allows library user to provide custom logger implementation as set of handlers.
///
/// #Params
/// context: pointer to some logger context that will be available in logger handlers.
/// enabled: (optional) "enabled" operation handler - calls to determines if a log record would be logged. (false positive if not specified)
/// log: "log" operation handler - calls to logs a record.
/// flush: (optional) "flush" operation handler - calls to flushes buffered records (in case of crash or signal).
///
/// #Returns
/// u32 Error Code
#[no_mangle]
pub extern "C" fn vcx_set_logger(
    context: *const CVoid,
    enabled: Option<EnabledCB>,
    log: Option<LogCB>,
    flush: Option<FlushCB>,
) -> u32 {
    info!("vcx_set_logger >>>");

    trace!(
        "vcx_set_logger( context: {:?}, enabled: {:?}, log: {:?}, flush: {:?}",
        context,
        enabled,
        log,
        flush
    );
    check_useful_c_callback!(log, VcxErrorKind::InvalidOption);

    let res = LibvcxLogger::init(context, enabled, log, flush);
    match res {
        Ok(()) => {
            debug!("Logger Successfully Initialized");
            SUCCESS.code_num
        }
        Err(err) => {
            set_current_error_vcx(&err);
            error!("Logger Failed To Initialize: {}", err);
            err.into()
        }
    }
}

/// Get the currently used logger.
///
/// NOTE: if logger is not set dummy implementation would be returned.
///
/// #Params
/// `context_p` - Reference that will contain logger context.
/// `enabled_cb_p` - Reference that will contain pointer to enable operation handler.
/// `log_cb_p` - Reference that will contain pointer to log operation handler.
/// `flush_cb_p` - Reference that will contain pointer to flush operation handler.
///
/// #Returns
/// Error code
///
/// This is tested in wrapper tests (python3)
#[no_mangle]
pub extern "C" fn vcx_get_logger(
    context_p: *mut *const CVoid,
    enabled_cb_p: *mut Option<EnabledCB>,
    log_cb_p: *mut Option<LogCB>,
    flush_cb_p: *mut Option<FlushCB>,
) -> u32 {
    info!("vcx_get_logger >>>");

    trace!(
        "vcx_get_logger >>> context_p: {:?}, enabled_cb_p: {:?}, log_cb_p: {:?}, flush_cb_p: {:?}",
        context_p,
        enabled_cb_p,
        log_cb_p,
        flush_cb_p
    );

    unsafe {
        let (context, enabled_cb, log_cb, flush_cb) = LOGGER_STATE.get();
        *context_p = context;
        *enabled_cb_p = enabled_cb;
        *log_cb_p = log_cb;
        *flush_cb_p = flush_cb;
    }

    let res = SUCCESS.code_num;
    trace!("vcx_get_logger: <<< res: {:?}", res);
    res
}
