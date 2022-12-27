use std::ptr;

use futures::future::BoxFuture;
use libc::c_char;

use crate::api_lib::api_c::types::CommandHandle;
use crate::api_lib::api_handle::agent;
use crate::api_lib::errors::error;
use crate::api_lib::errors::error::{LibvcxError, LibvcxErrorKind};
use crate::api_lib::utils::cstring::CStringUtils;
use crate::api_lib::utils::current_error::set_current_error_vcx;
use crate::api_lib::utils::runtime::{execute, execute_async};

#[no_mangle]
pub extern "C" fn vcx_public_agent_create(
    command_handle: CommandHandle,
    source_id: *const c_char,
    institution_did: *const c_char,
    cb: Option<extern "C" fn(xcommand_handle: CommandHandle, err: u32, agent_handle: u32)>,
) -> u32 {
    info!("vcx_public_agent_create >>>");

    check_useful_c_callback!(cb, LibvcxErrorKind::InvalidOption);
    check_useful_c_str!(source_id, LibvcxErrorKind::InvalidOption);
    check_useful_c_str!(institution_did, LibvcxErrorKind::InvalidOption);

    trace!(
        "vcx_public_agent_create(command_handle: {}, institution_did: {}) source_id: {}",
        command_handle,
        institution_did,
        source_id
    );

    execute_async::<BoxFuture<'static, Result<(), ()>>>(Box::pin(async move {
        match agent::create_public_agent(&source_id, &institution_did).await {
            Ok(handle) => {
                trace!(
                    "vcx_public_agent_create_cb(command_handle: {}, rc: {}, handle: {})",
                    command_handle,
                    error::SUCCESS_ERR_CODE,
                    handle
                );
                cb(command_handle, error::SUCCESS_ERR_CODE, handle);
            }
            Err(err) => {
                set_current_error_vcx(&err);
                error!(
                    "vcx_public_agent_create_cb(command_handle: {}, rc: {}, handle: {})",
                    command_handle, err, 0
                );
                cb(command_handle, err.into(), 0);
            }
        }
        Ok(())
    }));

    error::SUCCESS_ERR_CODE
}

#[no_mangle]
pub extern "C" fn vcx_public_agent_download_connection_requests(
    command_handle: CommandHandle,
    agent_handle: u32,
    uids: *const c_char,
    cb: Option<extern "C" fn(xcommand_handle: CommandHandle, err: u32, requests: *const c_char)>,
) -> u32 {
    info!("vcx_public_agent_download_connection_requests >>>");

    check_useful_c_callback!(cb, LibvcxErrorKind::InvalidOption);

    let uids = if !uids.is_null() {
        check_useful_c_str!(uids, LibvcxErrorKind::InvalidOption);
        let v: Vec<&str> = uids.split(',').collect();
        let v = v.iter().map(|s| s.to_string()).collect::<Vec<String>>();
        Some(v)
    } else {
        None
    };

    trace!(
        "vcx_public_agent_download_connection_requests(command_handle: {}, agent_handle: {}, uids: {:?})",
        command_handle,
        agent_handle,
        uids
    );

    execute_async::<BoxFuture<'static, Result<(), ()>>>(Box::pin(async move {
        match agent::download_connection_requests(agent_handle, uids.as_ref()).await {
            Ok(requests) => {
                trace!(
                    "vcx_public_agent_download_connection_requests_cb(command_handle: {}, rc: {}, requests: {})",
                    command_handle,
                    error::SUCCESS_ERR_CODE,
                    requests
                );
                let requests = CStringUtils::string_to_cstring(requests);
                cb(command_handle, error::SUCCESS_ERR_CODE, requests.as_ptr());
            }
            Err(err) => {
                set_current_error_vcx(&err);
                error!(
                    "vcx_public_agent_download_connection_requests_cb(command_handle: {}, rc: {}, requests: {})",
                    command_handle, err, 0
                );
                cb(command_handle, err.into(), ptr::null());
            }
        }
        Ok(())
    }));

    error::SUCCESS_ERR_CODE
}

#[no_mangle]
pub extern "C" fn vcx_public_agent_download_message(
    command_handle: CommandHandle,
    agent_handle: u32,
    uid: *const c_char,
    cb: Option<extern "C" fn(xcommand_handle: CommandHandle, err: u32, msg: *const c_char)>,
) -> u32 {
    info!("vcx_public_agent_download_message >>>");

    check_useful_c_callback!(cb, LibvcxErrorKind::InvalidOption);
    check_useful_c_str!(uid, LibvcxErrorKind::InvalidOption);

    trace!(
        "vcx_public_agent_download_message(command_handle: {}, agent_handle: {}, uids: {:?})",
        command_handle,
        agent_handle,
        uid
    );

    execute_async::<BoxFuture<'static, Result<(), ()>>>(Box::pin(async move {
        match agent::download_message(agent_handle, &uid).await {
            Ok(msg) => {
                trace!(
                    "vcx_public_agent_download_message_cb(command_handle: {}, rc: {}, msg: {})",
                    command_handle,
                    error::SUCCESS_ERR_CODE,
                    msg
                );
                let msg = CStringUtils::string_to_cstring(msg);
                cb(command_handle, error::SUCCESS_ERR_CODE, msg.as_ptr());
            }
            Err(err) => {
                set_current_error_vcx(&err);
                error!(
                    "vcx_public_agent_download_message_cb(command_handle: {}, rc: {}, msg: {})",
                    command_handle, err, 0
                );
                cb(command_handle, err.into(), ptr::null());
            }
        }
        Ok(())
    }));

    error::SUCCESS_ERR_CODE
}

#[no_mangle]
pub extern "C" fn vcx_public_agent_get_service(
    command_handle: CommandHandle,
    agent_handle: u32,
    cb: Option<extern "C" fn(xcommand_handle: CommandHandle, err: u32, service: *const c_char)>,
) -> u32 {
    info!("vcx_public_agent_get_service >>>");

    check_useful_c_callback!(cb, LibvcxErrorKind::InvalidOption);

    trace!(
        "vcx_public_agent_get_service(command_handle: {}, agent_handle: {})",
        command_handle,
        agent_handle
    );

    execute(move || {
        match agent::get_service(agent_handle) {
            Ok(service) => {
                trace!(
                    "vcx_public_agent_get_service_cb(command_handle: {}, rc: {}, service: {})",
                    command_handle,
                    error::SUCCESS_ERR_CODE,
                    service
                );
                let service = CStringUtils::string_to_cstring(service);
                cb(command_handle, error::SUCCESS_ERR_CODE, service.as_ptr());
            }
            Err(err) => {
                set_current_error_vcx(&err);
                error!(
                    "vcx_public_agent_get_service_cb(command_handle: {}, rc: {}, service: {})",
                    command_handle, err, 0
                );
                cb(command_handle, err.into(), ptr::null());
            }
        }
        Ok(())
    });

    error::SUCCESS_ERR_CODE
}

#[no_mangle]
pub extern "C" fn vcx_public_agent_serialize(
    command_handle: CommandHandle,
    agent_handle: u32,
    cb: Option<extern "C" fn(xcommand_handle: CommandHandle, err: u32, agent_json: *const c_char)>,
) -> u32 {
    info!("vcx_public_agent_serialize >>>");

    check_useful_c_callback!(cb, LibvcxErrorKind::InvalidOption);

    trace!(
        "vcx_public_agent_serialize(command_handle: {}, agent_handle: {})",
        command_handle,
        agent_handle
    );

    execute(move || {
        match agent::to_string(agent_handle) {
            Ok(agent_json) => {
                trace!(
                    "vcx_public_agent_serialize_cb(command_handle: {}, rc: {}, agent_json: {})",
                    command_handle,
                    error::SUCCESS_ERR_CODE,
                    agent_json
                );
                let agent_json = CStringUtils::string_to_cstring(agent_json);
                cb(command_handle, error::SUCCESS_ERR_CODE, agent_json.as_ptr());
            }
            Err(err) => {
                set_current_error_vcx(&err);
                error!(
                    "vcx_public_agent_serialize_cb(command_handle: {}, rc: {}, agent_json: {})",
                    command_handle, err, 0
                );
                cb(command_handle, err.into(), ptr::null());
            }
        }
        Ok(())
    });

    error::SUCCESS_ERR_CODE
}

#[no_mangle]
pub extern "C" fn vcx_public_agent_deserialize(
    command_handle: CommandHandle,
    agent_json: *const c_char,
    cb: Option<extern "C" fn(xcommand_handle: CommandHandle, err: u32, agent_handle: u32)>,
) -> u32 {
    info!("vcx_public_agent_deserialize >>>");

    check_useful_c_callback!(cb, LibvcxErrorKind::InvalidOption);
    check_useful_c_str!(agent_json, LibvcxErrorKind::InvalidOption);

    trace!(
        "vcx_public_agent_deserialize(command_handle: {}, agent_json: {})",
        command_handle,
        agent_json
    );

    execute(move || {
        match agent::from_string(&agent_json) {
            Ok(agent_handle) => {
                trace!(
                    "vcx_public_agent_deserialize_cb(command_handle: {}, rc: {}, agent_handle: {})",
                    command_handle,
                    error::SUCCESS_ERR_CODE,
                    agent_handle
                );
                cb(command_handle, error::SUCCESS_ERR_CODE, agent_handle);
            }
            Err(err) => {
                set_current_error_vcx(&err);
                error!(
                    "vcx_public_agent_deserialize_cb(command_handle: {}, rc: {}, agent_handle: {})",
                    command_handle, err, 0
                );
                cb(command_handle, err.into(), 0);
            }
        }
        Ok(())
    });

    error::SUCCESS_ERR_CODE
}

#[no_mangle]
pub extern "C" fn vcx_public_agent_release(agent_handle: u32) -> u32 {
    info!("vcx_public_agent_release >>>");

    match agent::release(agent_handle) {
        Ok(()) => {
            trace!(
                "vcx_public_agent_release(agent_handle: {}, rc: {})",
                agent_handle,
                error::SUCCESS_ERR_CODE
            );
            error::SUCCESS_ERR_CODE
        }
        Err(err) => {
            set_current_error_vcx(&err);
            error!("vcx_public_agent_release(agent_handle: {}), rc: {})", agent_handle, err);
            err.into()
        }
    }
}
