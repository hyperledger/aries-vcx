use std::ptr;
use libc::c_char;

use aries_vcx::indy_sys::CommandHandle;

use crate::api_lib::api_handle::agent;
use crate::api_lib::utils::cstring::CStringUtils;
use crate::api_lib::utils::runtime::execute;
use crate::error::prelude::*;
use aries_vcx::utils::error;

#[no_mangle]
pub extern fn vcx_public_agent_create(command_handle: CommandHandle,
                                      institution_did: *const c_char,
                                      cb: Option<extern fn(xcommand_handle: CommandHandle, err: u32, agent_handle: u32)>) -> u32 {
    info!("vcx_public_agent_create >>>");

    check_useful_c_callback!(cb, VcxErrorKind::InvalidOption);
    check_useful_c_str!(institution_did, VcxErrorKind::InvalidOption);

    trace!("vcx_public_agent_create(command_handle: {}, institution_did: {})", command_handle, institution_did);

    execute(move || {
        match agent::create_public_agent(&institution_did) {
            Ok(handle) => {
                trace!("vcx_public_agent_create_cb(command_handle: {}, rc: {}, handle: {})",
                       command_handle, error::SUCCESS.message, handle);
                cb(command_handle, error::SUCCESS.code_num, handle);
            }
            Err(x) => {
                warn!("vcx_public_agent_create_cb(command_handle: {}, rc: {}, handle: {})",
                      command_handle, x, 0);
                cb(command_handle, x.into(), 0);
            }
        }
        Ok(())
    });

    error::SUCCESS.code_num
}

#[no_mangle]
pub extern fn vcx_public_agent_generate_public_invite(command_handle: CommandHandle,
                                                      agent_handle: u32,
                                                      label: *const c_char,
                                                      cb: Option<extern fn(xcommand_handle: CommandHandle, err: u32, public_invite: *const c_char)>) -> u32 {
    info!("vcx_public_agent_generate_public_invite >>>");

    check_useful_c_callback!(cb, VcxErrorKind::InvalidOption);
    check_useful_c_str!(label, VcxErrorKind::InvalidOption);

    trace!("vcx_public_agent_generate_public_invite(command_handle: {}, label: {})", command_handle, label);

    execute(move || {
        match agent::generate_public_invite(agent_handle, &label) {
            Ok(public_invite) => {
                trace!("generate_public_invite_cb(command_handle: {}, rc: {}, public_invite: {})",
                       command_handle, error::SUCCESS.message, public_invite);
                let public_invite = CStringUtils::string_to_cstring(public_invite);
                cb(command_handle, error::SUCCESS.code_num, public_invite.as_ptr());
            }
            Err(x) => {
                warn!("generate_public_invite_cb(command_handle: {}, rc: {}, public_invite: {})",
                      command_handle, x, 0);
                cb(command_handle, x.into(), ptr::null());
            }
        }
        Ok(())
    });

    error::SUCCESS.code_num
}
