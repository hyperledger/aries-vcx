use std::ptr;

use futures::future::BoxFuture;
use libc::c_char;

use aries_vcx::error::{VcxError, VcxErrorKind};
use aries_vcx::indy_sys::CommandHandle;
use aries_vcx::utils::error;

use crate::api_lib::api_handle::out_of_band;
use crate::api_lib::utils::cstring::CStringUtils;
use crate::api_lib::utils::error::{set_current_error, set_current_error_vcx};
use crate::api_lib::utils::runtime::{execute_async, execute};

#[no_mangle]
pub extern fn vcx_out_of_band_sender_create(command_handle: CommandHandle,
                                            config: *const c_char,
                                            cb: Option<extern fn(xcommand_handle: CommandHandle, err: u32, handle: u32)>) -> u32 {
    info!("vcx_out_of_band_sender_create >>>");

    check_useful_c_str!(config, VcxErrorKind::InvalidOption);
    check_useful_c_callback!(cb, VcxErrorKind::InvalidOption);

    trace!("vcx_out_of_band_sender_create(command_handle: {}, config: {})", command_handle, config);

    execute_async::<BoxFuture<'static, Result<(), ()>>>(Box::pin(async move {
        match out_of_band::create_out_of_band(&config).await {
            Ok(handle) => {
                trace!("vcx_out_of_band_sender_create_cb(command_handle: {}, rc: {}, handle: {})",
                       command_handle, error::SUCCESS.message, handle);
                cb(command_handle, error::SUCCESS.code_num, handle);
            }
            Err(x) => {
                set_current_error_vcx(&x);
                warn!("vcx_out_of_band_sender_create_cb(command_handle: {}, rc: {}, handle: {})",
                      command_handle, x, 0);
                cb(command_handle, x.into(), 0);
            }
        }
        Ok(())
    }));

    error::SUCCESS.code_num
}

#[no_mangle]
pub extern fn vcx_out_of_band_receiver_create(command_handle: CommandHandle,
                                              message: *const c_char,
                                              cb: Option<extern fn(xcommand_handle: CommandHandle, err: u32, handle: u32)>) -> u32 {
    info!("vcx_out_of_band_receiver_create >>>");

    check_useful_c_str!(message, VcxErrorKind::InvalidOption);
    check_useful_c_callback!(cb, VcxErrorKind::InvalidOption);

    trace!("vcx_out_of_band_receiver_create(command_handle: {}, message: {})", command_handle, message);

    execute(move || {
        match out_of_band::create_out_of_band_msg_from_msg(&message) {
            Ok(handle) => {
                trace!("vcx_out_of_band_receiver_create_cb(command_handle: {}, rc: {}, handle: {})",
                       command_handle, error::SUCCESS.message, handle);
                cb(command_handle, error::SUCCESS.code_num, handle);
            }
            Err(x) => {
                set_current_error_vcx(&x);
                warn!("vcx_out_of_band_receiver_create_cb(command_handle: {}, rc: {}, handle: {}):",
                      command_handle, x, 0);
                cb(command_handle, x.into(), 0);
            }
        }
        Ok(())
    });

    error::SUCCESS.code_num
}

#[no_mangle]
pub extern fn vcx_out_of_band_sender_get_thread_id(command_handle: CommandHandle,
                                                   handle: u32,
                                                   cb: Option<extern fn(xcommand_handle: CommandHandle, err: u32, thid: *const c_char)>) -> u32 {
    info!("vcx_out_of_band_sender_get_thread_id >>>");

    check_useful_c_callback!(cb, VcxErrorKind::InvalidOption);

    trace!("vcx_out_of_band_sender_get_thread_id(command_handle: {}, handle: {})", command_handle, handle);

    execute(move || {
        match out_of_band::get_thread_id_sender(handle) {
            Ok(thid) => {
                trace!("vcx_out_of_band_sender_get_thread_id_cb(command_handle: {}, rc: {}, thid: {})",
                       command_handle, error::SUCCESS.message, thid);
                let thid = CStringUtils::string_to_cstring(thid);
                cb(command_handle, error::SUCCESS.code_num, thid.as_ptr());
            }
            Err(x) => {
                warn!("vcx_out_of_band_sender_get_thread_id_cb(command_handle: {}, rc: {}, thid: {})",
                      command_handle, x, 0);
                cb(command_handle, x.into(), ptr::null());
            }
        }
        Ok(())
    });

    error::SUCCESS.code_num
}

#[no_mangle]
pub extern fn vcx_out_of_band_receiver_get_thread_id(command_handle: CommandHandle,
                                                     handle: u32,
                                                     cb: Option<extern fn(xcommand_handle: CommandHandle, err: u32, thid: *const c_char)>) -> u32 {
    info!("vcx_out_of_band_receiver_get_thread_id >>>");

    check_useful_c_callback!(cb, VcxErrorKind::InvalidOption);

    trace!("vcx_out_of_band_receiver_get_thread_id(command_handle: {}, handle: {})", command_handle, handle);

    execute(move || {
        match out_of_band::get_thread_id_receiver(handle) {
            Ok(thid) => {
                trace!("vcx_out_of_band_receiver_get_thread_id_cb(command_handle: {}, rc: {}, thid: {})",
                       command_handle, error::SUCCESS.message, thid);
                let thid = CStringUtils::string_to_cstring(thid);
                cb(command_handle, error::SUCCESS.code_num, thid.as_ptr());
            }
            Err(x) => {
                warn!("vcx_out_of_band_receiver_get_thread_id_cb(command_handle: {}, rc: {}, thid: {})",
                      command_handle, x, 0);
                cb(command_handle, x.into(), ptr::null());
            }
        }
        Ok(())
    });

    error::SUCCESS.code_num
}

#[no_mangle]
pub extern fn vcx_out_of_band_sender_append_message(command_handle: CommandHandle,
                                                    handle: u32,
                                                    message: *const c_char,
                                                    cb: Option<extern fn(xcommand_handle: CommandHandle, err: u32)>) -> u32 {
    info!("vcx_out_of_band_sender_append_message >>>");

    check_useful_c_str!(message, VcxErrorKind::InvalidOption);
    check_useful_c_callback!(cb, VcxErrorKind::InvalidOption);

    trace!("vcx_out_of_band_sender_append_message(command_handle: {}, handle: {}, message: {})", command_handle, handle, message);

    execute(move || {
        match out_of_band::append_message(handle, &message) {
            Ok(()) => {
                trace!("vcx_out_of_band_sender_append_message_cb(command_handle: {}, rc: {})",
                       command_handle, error::SUCCESS.message);
                cb(command_handle, error::SUCCESS.code_num);
            }
            Err(x) => {
                set_current_error_vcx(&x);
                warn!("vcx_out_of_band_sender_append_message_cb(command_handle: {}, rc: {})",
                      command_handle, x);
                cb(command_handle, x.into());
            }
        }
        Ok(())
    });

    error::SUCCESS.code_num
}

#[no_mangle]
pub extern fn vcx_out_of_band_sender_append_service(command_handle: CommandHandle,
                                                    handle: u32,
                                                    service: *const c_char,
                                                    cb: Option<extern fn(xcommand_handle: CommandHandle, err: u32)>) -> u32 {
    info!("vcx_out_of_band_sender_append_service >>>");

    check_useful_c_str!(service, VcxErrorKind::InvalidOption);
    check_useful_c_callback!(cb, VcxErrorKind::InvalidOption);

    trace!("vcx_out_of_band_sender_append_service(command_handle: {}, handle: {}, service: {})", command_handle, handle, service);

    execute(move || {
        match out_of_band::append_service(handle, &service) {
            Ok(()) => {
                trace!("vcx_out_of_band_sender_append_service_cb(command_handle: {}, rc: {})",
                       command_handle, error::SUCCESS.message);
                cb(command_handle, error::SUCCESS.code_num);
            }
            Err(x) => {
                set_current_error_vcx(&x);
                warn!("vcx_out_of_band_sender_append_service_cb(command_handle: {}, rc: {})",
                      command_handle, x);
                cb(command_handle, x.into());
            }
        }
        Ok(())
    });

    error::SUCCESS.code_num
}

#[no_mangle]
pub extern fn vcx_out_of_band_sender_append_service_did(command_handle: CommandHandle,
                                                        handle: u32,
                                                        did: *const c_char,
                                                        cb: Option<extern fn(xcommand_handle: CommandHandle, err: u32)>) -> u32 {
    info!("vcx_out_of_band_sender_append_service_did >>>");

    check_useful_c_str!(did, VcxErrorKind::InvalidOption);
    check_useful_c_callback!(cb, VcxErrorKind::InvalidOption);

    trace!("vcx_out_of_band_sender_append_service_did(command_handle: {}, handle: {}, did: {})", command_handle, handle, did);

    execute(move || {
        match out_of_band::append_service_did(handle, &did) {
            Ok(()) => {
                trace!("vcx_out_of_band_sender_append_service_did_cb(command_handle: {}, rc: {})",
                       command_handle, error::SUCCESS.message);
                cb(command_handle, error::SUCCESS.code_num);
            }
            Err(x) => {
                set_current_error_vcx(&x);
                warn!("vcx_out_of_band_sender_append_service_did_cb(command_handle: {}, rc: {})",
                      command_handle, x);
                cb(command_handle, x.into());
            }
        }
        Ok(())
    });

    error::SUCCESS.code_num
}

#[no_mangle]
pub extern fn vcx_out_of_band_receiver_extract_message(command_handle: CommandHandle,
                                                       handle: u32,
                                                       cb: Option<extern fn(xcommand_handle: CommandHandle, err: u32, message: *const c_char)>) -> u32 {
    info!("vcx_out_of_band_receiver_extract_message >>>");

    check_useful_c_callback!(cb, VcxErrorKind::InvalidOption);

    trace!("vcx_out_of_band_receiver_extract_message(command_handle: {}, handle: {})", command_handle, handle);

    execute(move || {
        match out_of_band::extract_a2a_message(handle) {
            Ok(msg) => {
                trace!("vcx_out_of_band_receiver_extract_message_cb(command_handle: {}, rc: {}, msg: {})",
                       command_handle, error::SUCCESS.message, msg);
                let msg = CStringUtils::string_to_cstring(msg);
                cb(command_handle, error::SUCCESS.code_num, msg.as_ptr());
            }
            Err(x) => {
                set_current_error_vcx(&x);
                warn!("vcx_out_of_band_receiver_extract_message_cb(command_handle: {}, rc: {}, msg: {})",
                      command_handle, x, "");
                cb(command_handle, x.into(), ptr::null());
            }
        }
        Ok(())
    });

    error::SUCCESS.code_num
}

#[no_mangle]
pub extern fn vcx_out_of_band_to_message(command_handle: CommandHandle,
                                         handle: u32,
                                         cb: Option<extern fn(xcommand_handle: CommandHandle, err: u32, message: *const c_char)>) -> u32 {
    info!("vcx_out_of_band_to_message >>>");

    check_useful_c_callback!(cb, VcxErrorKind::InvalidOption);

    trace!("vcx_out_of_band_to_message(command_handle: {}, handle: {})", command_handle, handle);

    execute(move || {
        match out_of_band::to_a2a_message(handle) {
            Ok(msg) => {
                trace!("vcx_out_of_band_to_message_cb(command_handle: {}, rc: {}, msg: {})",
                       command_handle, error::SUCCESS.message, msg);
                let msg = CStringUtils::string_to_cstring(msg);
                cb(command_handle, error::SUCCESS.code_num, msg.as_ptr());
            }
            Err(x) => {
                set_current_error_vcx(&x);
                warn!("vcx_out_of_band_to_message_cb(command_handle: {}, rc: {}, msg: {})",
                      command_handle, x, "");
                cb(command_handle, x.into(), ptr::null());
            }
        }
        Ok(())
    });

    error::SUCCESS.code_num
}

#[no_mangle]
pub extern fn vcx_out_of_band_receiver_connection_exists(command_handle: CommandHandle,
                                                         handle: u32,
                                                         conn_handles: *const c_char,
                                                         cb: Option<extern fn(xcommand_handle: CommandHandle, err: u32, conn_handle: u32, found_one: bool)>) -> u32 {
    info!("vcx_out_of_band_receiver_connection_exists >>>");

    check_useful_c_str!(conn_handles, VcxErrorKind::InvalidOption);
    check_useful_c_callback!(cb, VcxErrorKind::InvalidOption);

    trace!("vcx_out_of_band_receiver_connection_exists(command_handle: {}, handle: {}, conn_handles: {})", command_handle, handle, conn_handles);

    let conn_handles = match serde_json::from_str::<Vec<u32>>(&conn_handles) {
        Ok(conn_handles) => conn_handles,
        Err(err) => {
            set_current_error(&err);
            error!("vcx_out_of_band_receiver_connection_exists >>> failed to parse connection handles: {}, err: {:?}", conn_handles, err);
            return error::INVALID_CONNECTION_HANDLE.code_num;
        }
    };

    execute(move || {
        match out_of_band::connection_exists(handle, &conn_handles) {
            Ok((conn_handle, found_one)) => {
                trace!("vcx_out_of_band_receiver_connection_exists_cb(command_handle: {}, rc: {}, conn_handle: {}, found_one: {})",
                       command_handle, error::SUCCESS.message, conn_handle, found_one);
                cb(command_handle, error::SUCCESS.code_num, conn_handle, found_one);
            }
            Err(x) => {
                set_current_error_vcx(&x);
                warn!("vcx_out_of_band_receiver_connection_exists_cb(command_handle: {}, rc: {}, conn_handle: {}, found_one: {})",
                      command_handle, x, 0, false);
                cb(command_handle, x.into(), 0, false);
            }
        }
        Ok(())
    });

    error::SUCCESS.code_num
}

#[no_mangle]
pub extern fn vcx_out_of_band_receiver_build_connection(command_handle: CommandHandle,
                                                        handle: u32,
                                                        cb: Option<extern fn(xcommand_handle: CommandHandle, err: u32, connection: *const c_char)>) -> u32 {
    info!("vcx_out_of_band_receiver_build_connection >>>");

    check_useful_c_callback!(cb, VcxErrorKind::InvalidOption);

    trace!("vcx_out_of_band_receiver_build_connection(command_handle: {}, handle: {})", command_handle, handle);

    execute_async::<BoxFuture<'static, Result<(), ()>>>(Box::pin(async move {
        match out_of_band::build_connection(handle).await {
            Ok(connection) => {
                trace!("vcx_out_of_band_receiver_build_connection_cb(command_handle: {}, rc: {}, connection: {})",
                       command_handle, error::SUCCESS.message, connection);
                let connection = CStringUtils::string_to_cstring(connection);
                cb(command_handle, error::SUCCESS.code_num, connection.as_ptr());
            }
            Err(x) => {
                set_current_error_vcx(&x);
                warn!("vcx_out_of_band_receiver_build_connection_cb(command_handle: {}, rc: {}, connection: {})",
                      command_handle, x, "");
                cb(command_handle, x.into(), ptr::null());
            }
        }
        Ok(())
    }));

    error::SUCCESS.code_num
}

#[no_mangle]
pub extern fn vcx_out_of_band_sender_serialize(command_handle: CommandHandle,
                                               handle: u32,
                                               cb: Option<extern fn(xcommand_handle: CommandHandle, err: u32, oob_json: *const c_char)>) -> u32 {
    info!("vcx_out_of_band_sender_serialize >>>");

    check_useful_c_callback!(cb, VcxErrorKind::InvalidOption);

    trace!("vcx_out_of_band_sender_serialize(command_handle: {}, handle: {})", command_handle, handle);

    execute(move || {
        match out_of_band::to_string_sender(handle) {
            Ok(oob_json) => {
                trace!("vcx_out_of_band_sender_serialize_cb(command_handle: {}, rc: {}, oob_json: {})",
                       command_handle, error::SUCCESS.message, oob_json);
                let oob_json = CStringUtils::string_to_cstring(oob_json);
                cb(command_handle, error::SUCCESS.code_num, oob_json.as_ptr());
            }
            Err(x) => {
                set_current_error_vcx(&x);
                warn!("vcx_out_of_band_sender_serialize_cb(command_handle: {}, rc: {}, oob_json: {})",
                      command_handle, x, 0);
                cb(command_handle, x.into(), ptr::null());
            }
        }
        Ok(())
    });

    error::SUCCESS.code_num
}

#[no_mangle]
pub extern fn vcx_out_of_band_receiver_serialize(command_handle: CommandHandle,
                                                 handle: u32,
                                                 cb: Option<extern fn(xcommand_handle: CommandHandle, err: u32, oob_json: *const c_char)>) -> u32 {
    info!("vcx_out_of_band_receiver_serialize >>>");

    check_useful_c_callback!(cb, VcxErrorKind::InvalidOption);

    trace!("vcx_out_of_band_receiver_serialize(command_handle: {}, handle: {})", command_handle, handle);

    execute(move || {
        match out_of_band::to_string_receiver(handle) {
            Ok(oob_json) => {
                trace!("vcx_out_of_band_receiver_serialize_cb(command_handle: {}, rc: {}, oob_json: {})",
                       command_handle, error::SUCCESS.message, oob_json);
                let oob_json = CStringUtils::string_to_cstring(oob_json);
                cb(command_handle, error::SUCCESS.code_num, oob_json.as_ptr());
            }
            Err(x) => {
                set_current_error_vcx(&x);
                warn!("vcx_out_of_band_receiver_serialize_cb(command_handle: {}, rc: {}, oob_json: {})",
                      command_handle, x, 0);
                cb(command_handle, x.into(), ptr::null());
            }
        }
        Ok(())
    });

    error::SUCCESS.code_num
}

#[no_mangle]
pub extern fn vcx_out_of_band_sender_deserialize(command_handle: CommandHandle,
                                                 oob_json: *const c_char,
                                                 cb: Option<extern fn(xcommand_handle: CommandHandle, err: u32, handle: u32)>) -> u32 {
    info!("vcx_out_of_band_sender_deserialize >>>");

    check_useful_c_callback!(cb, VcxErrorKind::InvalidOption);
    check_useful_c_str!(oob_json, VcxErrorKind::InvalidOption);

    trace!("vcx_out_of_band_sender_deserialize(command_handle: {}, oob_json: {})", command_handle, oob_json);

    execute(move || {
        match out_of_band::from_string_sender(&oob_json) {
            Ok(handle) => {
                trace!("vcx_out_of_band_sender_deserialize_cb(command_handle: {}, rc: {}, handle: {})",
                       command_handle, error::SUCCESS.message, handle);
                cb(command_handle, error::SUCCESS.code_num, handle);
            }
            Err(x) => {
                set_current_error_vcx(&x);
                warn!("vcx_out_of_band_sender_deserialize_cb(command_handle: {}, rc: {}, handle: {})",
                      command_handle, x, 0);
                cb(command_handle, x.into(), 0);
            }
        }
        Ok(())
    });

    error::SUCCESS.code_num
}

#[no_mangle]
pub extern fn vcx_out_of_band_receiver_deserialize(command_handle: CommandHandle,
                                                   oob_json: *const c_char,
                                                   cb: Option<extern fn(xcommand_handle: CommandHandle, err: u32, handle: u32)>) -> u32 {
    info!("vcx_out_of_band_receiver_deserialize >>>");

    check_useful_c_callback!(cb, VcxErrorKind::InvalidOption);
    check_useful_c_str!(oob_json, VcxErrorKind::InvalidOption);

    trace!("vcx_out_of_band_receiver_deserialize(command_handle: {}, oob_json: {})", command_handle, oob_json);

    execute(move || {
        match out_of_band::from_string_receiver(&oob_json) {
            Ok(handle) => {
                trace!("vcx_out_of_band_receiver_deserialize_cb(command_handle: {}, rc: {}, handle: {})",
                       command_handle, error::SUCCESS.message, handle);
                cb(command_handle, error::SUCCESS.code_num, handle);
            }
            Err(x) => {
                set_current_error_vcx(&x);
                warn!("vcx_out_of_band_receiver_deserialize_cb(command_handle: {}, rc: {}, handle: {})",
                      command_handle, x, 0);
                cb(command_handle, x.into(), 0);
            }
        }
        Ok(())
    });

    error::SUCCESS.code_num
}

#[no_mangle]
pub extern fn vcx_out_of_band_sender_release(handle: u32) -> u32 {
    info!("vcx_out_of_band_sender_release >>>");

    match out_of_band::release_sender(handle) {
        Ok(()) => {
            trace!("vcx_out_of_band_sender_release(handle: {}, rc: {})",
                   handle, error::SUCCESS.message);
            error::SUCCESS.code_num
        }
        Err(e) => {
            warn!("vcx_out_of_band_sender_release(handle: {}), rc: {})",
                  handle, e);
            e.into()
        }
    }
}

#[no_mangle]
pub extern fn vcx_out_of_band_receiver_release(handle: u32) -> u32 {
    info!("vcx_out_of_band_receiver_release >>>");

    match out_of_band::release_receiver(handle) {
        Ok(()) => {
            trace!("vcx_out_of_band_receiver_release(handle: {}, rc: {})",
                   handle, error::SUCCESS.message);
            error::SUCCESS.code_num
        }
        Err(e) => {
            warn!("vcx_out_of_band_receiver_release(handle: {}), rc: {})",
                  handle, e);
            e.into()
        }
    }
}
