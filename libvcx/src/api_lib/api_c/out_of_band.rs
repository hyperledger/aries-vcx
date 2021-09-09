use std::ptr;
use libc::c_char;

use aries_vcx::indy_sys::CommandHandle;

use crate::api_lib::api_handle::out_of_band;
use crate::api_lib::utils::cstring::CStringUtils;
use crate::api_lib::utils::runtime::execute;
use crate::error::prelude::*;
use aries_vcx::utils::error;

#[no_mangle]
pub extern fn vcx_out_of_band_create(command_handle: CommandHandle,
                                     source_id: *const c_char,
                                     config: *const c_char,
                                     cb: Option<extern fn(xcommand_handle: CommandHandle, err: u32, handle: u32)>) -> u32 {
    info!("vcx_out_of_band_create >>>");

    check_useful_c_str!(config, VcxErrorKind::InvalidOption);
    check_useful_c_str!(source_id, VcxErrorKind::InvalidOption);
    check_useful_c_callback!(cb, VcxErrorKind::InvalidOption);

    trace!("vcx_out_of_band_create(command_handle: {}, config: {}) source_id: {}", command_handle, config, source_id);

    execute(move || {
        match out_of_band::create_out_of_band_msg(&config) {
            Ok(handle) => {
                trace!("vcx_out_of_band_create_cb(command_handle: {}, rc: {}, handle: {}) source_id: {}",
                       command_handle, error::SUCCESS.message, handle, source_id);
                cb(command_handle, error::SUCCESS.code_num, handle);
            }
            Err(x) => {
                warn!("vcx_out_of_band_create_cb(command_handle: {}, rc: {}, handle: {}) source_id: {}",
                      command_handle, x, 0, source_id);
                cb(command_handle, x.into(), 0);
            }
        }
        Ok(())
    });

    error::SUCCESS.code_num
}

pub extern fn vcx_out_of_band_create_from_message(command_handle: CommandHandle,
                                                  source_id: *const c_char,
                                                  message: *const c_char,
                                                  cb: Option<extern fn(xcommand_handle: CommandHandle, err: u32, handle: u32)>) -> u32 {
    info!("vcx_out_of_band_create_from_message >>>");

    check_useful_c_str!(message, VcxErrorKind::InvalidOption);
    check_useful_c_str!(source_id, VcxErrorKind::InvalidOption);
    check_useful_c_callback!(cb, VcxErrorKind::InvalidOption);

    trace!("vcx_out_of_band_create_from_message(command_handle: {}, message: {}) source_id: {}", command_handle, message, source_id);

    execute(move || {
        match out_of_band::create_out_of_band_msg_from_msg(&message) {
            Ok(handle) => {
                trace!("vcx_out_of_band_create_from_message_cb(command_handle: {}, rc: {}, handle: {}) source_id: {}",
                       command_handle, error::SUCCESS.message, handle, source_id);
                cb(command_handle, error::SUCCESS.code_num, handle);
            }
            Err(x) => {
                warn!("vcx_out_of_band_create_from_message_cb(command_handle: {}, rc: {}, handle: {}) source_id: {}",
                      command_handle, x, 0, source_id);
                cb(command_handle, x.into(), 0);
            }
        }
        Ok(())
    });

    error::SUCCESS.code_num
}

#[no_mangle]
pub extern fn vcx_out_of_band_append_message(command_handle: CommandHandle,
                                             handle: u32,
                                             message: *const c_char,
                                             cb: Option<extern fn(xcommand_handle: CommandHandle, err: u32)>) -> u32 {
    info!("vcx_out_of_band_append_message >>>");

    check_useful_c_str!(message, VcxErrorKind::InvalidOption);
    check_useful_c_callback!(cb, VcxErrorKind::InvalidOption);

    if !out_of_band::is_valid_handle(handle) {
        return VcxError::from(VcxErrorKind::InvalidHandle).into();
    }

    trace!("vcx_out_of_band_append_message(command_handle: {}, handle: {}, message: {})", command_handle, handle, message);

    execute(move || {
        match out_of_band::append_message(handle, &message) {
            Ok(()) => {
                trace!("vcx_out_of_band_append_message_cb(command_handle: {}, rc: {})",
                       command_handle, error::SUCCESS.message);
                cb(command_handle, error::SUCCESS.code_num);
            }
            Err(x) => {
                warn!("vcx_out_of_band_append_message_cb(command_handle: {}, rc: {})",
                      command_handle, x);
                cb(command_handle, x.into());
            }
        }
        Ok(())
    });

    error::SUCCESS.code_num
}

#[no_mangle]
pub extern fn vcx_out_of_band_extract_message(command_handle: CommandHandle,
                                              handle: u32,
                                              cb: Option<extern fn(xcommand_handle: CommandHandle, err: u32, message: *const c_char)>) -> u32 {
    info!("vcx_out_of_band_extract_message >>>");

    check_useful_c_callback!(cb, VcxErrorKind::InvalidOption);

    if !out_of_band::is_valid_handle(handle) {
        return VcxError::from(VcxErrorKind::InvalidHandle).into();
    }

    trace!("vcx_out_of_band_extract_message(command_handle: {}, handle: {})", command_handle, handle);

    execute(move || {
        match out_of_band::extract_a2a_message(handle) {
            Ok(msg) => {
                trace!("vcx_out_of_band_append_message_cb(command_handle: {}, rc: {}, msg: {})",
                       command_handle, error::SUCCESS.message, msg);
                let msg = CStringUtils::string_to_cstring(msg);
                cb(command_handle, error::SUCCESS.code_num, msg.as_ptr());
            }
            Err(x) => {
                warn!("vcx_out_of_band_append_message_cb(command_handle: {}, rc: {}, msg: {})",
                      command_handle, x, "");
                cb(command_handle, x.into(), ptr::null());
            }
        }
        Ok(())
    });

    error::SUCCESS.code_num
}

#[no_mangle]
pub extern fn vcx_out_of_band_connection_exists(command_handle: CommandHandle,
                                                handle: u32,
                                                conn_handles: *const c_char,
                                                cb: Option<extern fn(xcommand_handle: CommandHandle, err: u32, conn_handle: u32, found_one: bool)>) -> u32 {
    info!("vcx_out_of_band_connection_exists >>>");

    check_useful_c_str!(conn_handles, VcxErrorKind::InvalidOption);
    check_useful_c_callback!(cb, VcxErrorKind::InvalidOption);

    if !out_of_band::is_valid_handle(handle) {
        return VcxError::from(VcxErrorKind::InvalidHandle).into();
    }

    trace!("vcx_out_of_band_connection_exists(command_handle: {}, handle: {}, conn_handles: {})", command_handle, handle, conn_handles);

    let conn_handles = match serde_json::from_str::<Vec<u32>>(&conn_handles) {
        Ok(conn_handles) => conn_handles,
        Err(err) => {
            error!("vcx_out_of_band_connection_exists >>> failed to parse connection handles: {}, err: {:?}", conn_handles, err);
            return error::INVALID_CONNECTION_HANDLE.code_num;
        }
    };

    execute(move || {
        match out_of_band::connection_exists(handle, conn_handles) {
            Ok((conn_handle, found_one)) => {
                trace!("vcx_out_of_band_connection_exists_cb(command_handle: {}, rc: {}, conn_handle: {}, found_one: {})",
                       command_handle, error::SUCCESS.message, conn_handle, found_one);
                cb(command_handle, error::SUCCESS.code_num, conn_handle, found_one);
            }
            Err(x) => {
                warn!("vcx_out_of_band_connection_exists_cb(command_handle: {}, rc: {}, conn_handle: {}, found_one: {})",
                      command_handle, x, 0, false);
                cb(command_handle, x.into(), 0, false);
            }
        }
        Ok(())
    });

    error::SUCCESS.code_num
}

#[no_mangle]
pub extern fn vcx_out_of_band_build_connection(command_handle: CommandHandle,
                                               handle: u32,
                                               cb: Option<extern fn(xcommand_handle: CommandHandle, err: u32, conn_handle: u32)>) -> u32 {
    info!("vcx_out_of_band_build_connection >>>");

    check_useful_c_callback!(cb, VcxErrorKind::InvalidOption);

    if !out_of_band::is_valid_handle(handle) {
        return VcxError::from(VcxErrorKind::InvalidHandle).into();
    }

    trace!("vcx_out_of_band_build_connection(command_handle: {}, handle: {})", command_handle, handle);

    execute(move || {
        match out_of_band::build_connection(handle) {
            Ok(conn_handle) => {
                trace!("vcx_out_of_band_build_connection_cb(command_handle: {}, rc: {}, conn_handle: {})",
                       command_handle, error::SUCCESS.message, conn_handle);
                cb(command_handle, error::SUCCESS.code_num, conn_handle);
            }
            Err(x) => {
                warn!("vcx_out_of_band_build_connection_cb(command_handle: {}, rc: {}, conn_handle: {})",
                      command_handle, x, 0);
                cb(command_handle, x.into(), 0);
            }
        }
        Ok(())
    });

    error::SUCCESS.code_num
}


#[no_mangle]
pub extern fn vcx_out_of_band_serialize(command_handle: CommandHandle,
                                        handle: u32,
                                        cb: Option<extern fn(xcommand_handle: CommandHandle, err: u32, oob_json: *const c_char)>) -> u32 {
    info!("vcx_out_of_band_serialize >>>");

    check_useful_c_callback!(cb, VcxErrorKind::InvalidOption);

    if !out_of_band::is_valid_handle(handle) {
        return VcxError::from(VcxErrorKind::InvalidHandle).into();
    }

    trace!("vcx_out_of_band_serialize(command_handle: {}, handle: {})", command_handle, handle);

    execute(move || {
        match out_of_band::to_string(handle) {
            Ok(oob_json) => {
                trace!("vcx_out_of_band_serialize_cb(command_handle: {}, rc: {}, oob_json: {})",
                       command_handle, error::SUCCESS.message, oob_json);
                let oob_json = CStringUtils::string_to_cstring(oob_json);
                cb(command_handle, error::SUCCESS.code_num, oob_json.as_ptr());
            }
            Err(x) => {
                warn!("vcx_out_of_band_serialize_cb(command_handle: {}, rc: {}, oob_json: {})",
                      command_handle, x, 0);
                cb(command_handle, x.into(), ptr::null());
            }
        }
        Ok(())
    });

    error::SUCCESS.code_num
}

#[no_mangle]
pub extern fn vcx_out_of_band_deserialize(command_handle: CommandHandle,
                                           oob_json: *const c_char,
                                           cb: Option<extern fn(xcommand_handle: CommandHandle, err: u32, handle: u32)>) -> u32 {
    info!("vcx_out_of_band_deserialize >>>");

    check_useful_c_callback!(cb, VcxErrorKind::InvalidOption);
    check_useful_c_str!(oob_json, VcxErrorKind::InvalidOption);

    trace!("vcx_out_of_band_deserialize(command_handle: {}, oob_json: {})", command_handle, oob_json);

    execute(move || {
        match out_of_band::from_string(&oob_json) {
            Ok(handle) => {
                trace!("vcx_out_of_band_deserialize_cb(command_handle: {}, rc: {}, handle: {})",
                       command_handle, error::SUCCESS.message, handle);
                cb(command_handle, error::SUCCESS.code_num, handle);
            }
            Err(x) => {
                warn!("vcx_out_of_band_deserialize_cb(command_handle: {}, rc: {}, handle: {})",
                      command_handle, x, 0);
                cb(command_handle, x.into(), 0);
            }
        }
        Ok(())
    });

    error::SUCCESS.code_num
}

#[no_mangle]
pub extern fn vcx_out_of_band_release(handle: u32) -> u32 {
    info!("vcx_out_of_band_release >>>");

    match out_of_band::release(handle) {
        Ok(()) => {
            trace!("vcx_out_of_band_release(handle: {}, rc: {})",
                   handle, error::SUCCESS.message);
            error::SUCCESS.code_num
        }
        Err(e) => {
            warn!("vcx_out_of_band_release(handle: {}), rc: {})",
                  handle, e);
            e.into()
        }
    }
}

