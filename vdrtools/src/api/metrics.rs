use indy_api_types::{ErrorCode, CommandHandle};
use indy_utils::ctypes;
use libc::c_char;
use indy_api_types::errors::IndyResult;
use crate::services::CommandMetric;
use crate::Locator;

/// Collect metrics.
///
/// #Returns
/// Map in the JSON format. Where keys are names of metrics.
///
/// #Errors
/// Common*
#[no_mangle]
pub extern fn indy_collect_metrics(command_handle: CommandHandle,
                                   cb: Option<extern fn(command_handle_: CommandHandle,
                                                        err: ErrorCode,
                                                        metrics_json: *const c_char)>) -> ErrorCode {
    debug!("indy_collect_metrics: >>> command_handle: {:?}, cb: {:?}",
           command_handle, cb);

    check_useful_c_callback!(cb, ErrorCode::CommonInvalidParam3);

    let locator = Locator::instance();

    let action = async move {
        let res = locator.metrics_controller.collect().await;
        res
    };

    let cb = move |res: IndyResult<_>| {
        let (err, metrics) = prepare_result!(res, String::new());

        trace!("indy_collect_metrics ? err {:?} metrics {:?}", err, metrics);

        let did = ctypes::string_to_cstring(metrics);
        cb(command_handle, err, did.as_ptr())
    };

    locator.executor.spawn_ok_instrumented(CommandMetric::MetricsCommandCollectMetrics, action, cb);

    let res = ErrorCode::Success;
    debug!("indy_collect_metrics: <<< res: {:?}", res);
    res
}
