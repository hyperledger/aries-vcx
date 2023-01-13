use indy_api_types::{errors::prelude::*, CommandHandle, ErrorCode, IndyHandle};
use indy_utils::ctypes;
use libc::c_char;

use crate::services::CommandMetric;
use crate::Locator;

#[no_mangle]
pub extern "C" fn indy_open_blob_storage_reader(
    command_handle: CommandHandle,
    type_: *const c_char,
    config_json: *const c_char,
    cb: Option<extern "C" fn(command_handle_: CommandHandle, err: ErrorCode, handle: IndyHandle)>,
) -> ErrorCode {
    debug!(
        "indy_open_blob_storage_reader > type_ {:?} config_json {:?}",
        type_, config_json
    );

    check_useful_c_str!(type_, ErrorCode::CommonInvalidParam2);
    check_useful_c_str!(config_json, ErrorCode::CommonInvalidParam3);
    check_useful_c_callback!(cb, ErrorCode::CommonInvalidParam4);

    debug!(
        "indy_open_blob_storage_reader ? type_ {:?} config_json {:?}",
        type_, config_json
    );

    let locator = Locator::instance();

    let action = async move {
        let res = locator
            .blob_storage_controller
            .open_reader(type_, config_json)
            .await;
        res
    };

    let cb = move |res: IndyResult<_>| {
        let (err, handle) = prepare_result!(res, 0);

        debug!(
            "indy_open_blob_storage_reader ? err {:?} handle {:?}",
            err, handle
        );

        cb(command_handle, err, handle)
    };

    locator
        .executor
        .spawn_ok_instrumented(CommandMetric::BlobStorageCommandOpenReader, action, cb);

    let res = ErrorCode::Success;
    debug!("indy_open_blob_storage_reader < {:?}", res);
    res
}

#[no_mangle]
pub extern "C" fn indy_open_blob_storage_writer(
    command_handle: CommandHandle,
    type_: *const c_char,
    config_json: *const c_char,
    cb: Option<extern "C" fn(command_handle_: CommandHandle, err: ErrorCode, handle: IndyHandle)>,
) -> ErrorCode {
    debug!(
        "indy_open_blob_storage_writer > type_ {:?} config_json {:?}",
        type_, config_json
    );

    check_useful_c_str!(type_, ErrorCode::CommonInvalidParam2);
    check_useful_c_str!(config_json, ErrorCode::CommonInvalidParam3);
    check_useful_c_callback!(cb, ErrorCode::CommonInvalidParam4);

    debug!(
        "indy_open_blob_storage_writer ? type_ {:?} config_json {:?}",
        type_, config_json
    );

    let locator = Locator::instance();

    let action = async move {
        let res = locator
            .blob_storage_controller
            .open_writer(type_, config_json)
            .await;
        res
    };

    let cb = move |res: IndyResult<_>| {
        let (err, handle) = prepare_result!(res, 0);

        debug!(
            "indy_open_blob_storage_writer ? err {:?} handle {:?}",
            err, handle
        );

        cb(command_handle, err, handle)
    };

    locator
        .executor
        .spawn_ok_instrumented(CommandMetric::BlobStorageCommandOpenWriter, action, cb);

    let res = ErrorCode::Success;
    debug!("indy_open_blob_storage_writer < {:?}", res);
    res
}
