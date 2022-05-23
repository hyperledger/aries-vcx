use std::ptr;

use libc::c_char;
use futures::future::BoxFuture;

use aries_vcx::error::{VcxError, VcxErrorKind};
use aries_vcx::indy_sys::CommandHandle;
use aries_vcx::settings;
use aries_vcx::utils::error;

use crate::api_lib::api_handle::credential_def;
use crate::api_lib::utils::cstring::CStringUtils;
use crate::api_lib::utils::error::set_current_error_vcx;
use crate::api_lib::utils::runtime::{execute, execute_async};

#[no_mangle]
pub extern fn vcx_credentialdef_create_and_store(command_handle: CommandHandle,
                                                 source_id: *const c_char,
                                                 schema_id: *const c_char,
                                                 issuer_did: *const c_char,
                                                 tag: *const c_char,
                                                 revocation_details: *const c_char,
                                                 cb: Option<extern fn(xcommand_handle: CommandHandle, err: u32, credentialdef_handle: u32)>) -> u32 {
    info!("vcx_credentialdef_create_and_store >>>");

    check_useful_c_callback!(cb, VcxErrorKind::InvalidOption);
    check_useful_c_str!(source_id, VcxErrorKind::InvalidOption);
    check_useful_c_str!(schema_id, VcxErrorKind::InvalidOption);
    check_useful_c_str!(tag, VcxErrorKind::InvalidOption);
    check_useful_c_str!(revocation_details, VcxErrorKind::InvalidOption);

    let issuer_did: String = if !issuer_did.is_null() {
        check_useful_c_str!(issuer_did, VcxErrorKind::InvalidOption);
        issuer_did.to_owned()
    } else {
        match settings::get_config_value(settings::CONFIG_INSTITUTION_DID) {
            Ok(err) => err,
            Err(err) => return err.into(),
        }
    };

    trace!("vcx_credentialdef_create_and_store(command_handle: {}, source_id: {}, schema_id: {}, issuer_did: {}, tag: {}, revocation_details: {:?})",
           command_handle,
           source_id,
           schema_id,
           issuer_did,
           tag,
           revocation_details);

    execute_async::<BoxFuture<'static, Result<(), ()>>>(Box::pin(async move {
        let (rc, handle) = match credential_def::create_and_store(source_id,
                                                                  schema_id,
                                                                  issuer_did,
                                                                  tag,
                                                                  revocation_details).await {
            Ok(err) => {
                trace!("vcx_credentialdef_create_and_store_cb(command_handle: {}, rc: {}, credentialdef_handle: {}), source_id: {:?}",
                       command_handle, error::SUCCESS.message, err, credential_def::get_source_id(err).unwrap_or_default());
                (error::SUCCESS.code_num, err)
            }
            Err(err) => {
                set_current_error_vcx(&err);
                error!("vcx_credentialdef_create_and_store_cb(command_handle: {}, rc: {}, credentialdef_handle: {}), source_id: {:?}",
                      command_handle, err, 0, "");
                (err.into(), 0)
            }
        };
        cb(command_handle, rc, handle);

        Ok(())
    }));

    error::SUCCESS.code_num
}

#[no_mangle]
pub extern fn vcx_credentialdef_create_v2(command_handle: CommandHandle,
                                          source_id: *const c_char,
                                          schema_id: *const c_char,
                                          issuer_did: *const c_char,
                                          tag: *const c_char,
                                          support_revocation: bool,
                                          cb: Option<extern fn(xcommand_handle: CommandHandle, err: u32, credentialdef_handle: u32)>) -> u32 {
    info!("vcx_credentialdef_create_v2 >>>");

    check_useful_c_callback!(cb, VcxErrorKind::InvalidOption);
    check_useful_c_str!(source_id, VcxErrorKind::InvalidOption);
    check_useful_c_str!(schema_id, VcxErrorKind::InvalidOption);
    check_useful_c_str!(tag, VcxErrorKind::InvalidOption);

    let issuer_did: String = if !issuer_did.is_null() {
        check_useful_c_str!(issuer_did, VcxErrorKind::InvalidOption);
        issuer_did.to_owned()
    } else {
        match settings::get_config_value(settings::CONFIG_INSTITUTION_DID) {
            Ok(err) => err,
            Err(err) => return err.into(),
        }
    };

    trace!("vcx_credentialdef_create_v2(command_handle: {}, source_id: {}, schema_id: {}, issuer_did: {}, tag: {}, support_revocation: {:?})",
           command_handle,
           source_id,
           schema_id,
           issuer_did,
           tag,
           support_revocation);

    execute_async::<BoxFuture<'static, Result<(), ()>>>(Box::pin(async move {
        let (rc, handle) = match credential_def::create(source_id,
                                                        schema_id,
                                                        issuer_did,
                                                        tag,
                                                        support_revocation).await {
            Ok(err) => {
                trace!("vcx_credentialdef_create_v2_cb(command_handle: {}, rc: {}, credentialdef_handle: {}), source_id: {:?}",
                       command_handle, error::SUCCESS.message, err, credential_def::get_source_id(err).unwrap_or_default());
                (error::SUCCESS.code_num, err)
            }
            Err(err) => {
                set_current_error_vcx(&err);
                error!("vcx_credentialdef_create_v2_cb(command_handle: {}, rc: {}, credentialdef_handle: {}), source_id: {:?}",
                      command_handle, err, 0, "");
                (err.into(), 0)
            }
        };
        cb(command_handle, rc, handle);

        Ok(())
    }));

    error::SUCCESS.code_num
}

#[no_mangle]
pub extern fn vcx_credentialdef_publish(command_handle: CommandHandle,
                                        credentialdef_handle: u32,
                                        tails_url: *const c_char,
                                        cb: Option<extern fn(xcommand_handle: CommandHandle, err: u32)>) -> u32 {
    info!("vcx_credentialdef_publish >>>");

    check_useful_c_callback!(cb, VcxErrorKind::InvalidOption);
    check_useful_opt_c_str!(tails_url, VcxErrorKind::InvalidOption);

    if !credential_def::is_valid_handle(credentialdef_handle) {
        return VcxError::from(VcxErrorKind::InvalidCredDefHandle).into();
    };

    let source_id = credential_def::get_source_id(credentialdef_handle).unwrap_or_default();
    trace!("vcx_credentialdef_publish(command_handle: {}, credentialdef_handle: {}, tails_url: {:?}), source_id: {:?}",
           command_handle, credentialdef_handle, tails_url, source_id);

    execute_async::<BoxFuture<'static, Result<(), ()>>>(Box::pin(async move {
        match credential_def::publish(credentialdef_handle, tails_url).await {
            Ok(_) => {
                trace!("vcx_credentialdef_publish_cb(command_handle: {}, rc: {}, credentialdef_handle: {}), source_id: {:?}",
                       command_handle, error::SUCCESS.message, credentialdef_handle, source_id);
                cb(command_handle, error::SUCCESS.code_num);
            }
            Err(err) => {
                set_current_error_vcx(&err);
                error!("vcx_credentialdef_publish_cb(command_handle: {}, rc: {}, credentialdef_handle: {}), source_id: {:?}",
                      command_handle, err, credentialdef_handle, source_id);
                cb(command_handle, err.into());
            }
        };
        Ok(())
    }));

    error::SUCCESS.code_num
}

/// Takes the credentialdef object and returns a json string of all its attributes
///
/// #Params
/// command_handle: command handle to map callback to user context.
///
/// credentialdef_handle: Credentialdef handle that was provided during creation. Used to access credentialdef object
///
/// cb: Callback that provides json string of the credentialdef's attributes and provides error status
///
// #Returns
/// Error code as a u32
#[no_mangle]
pub extern fn vcx_credentialdef_serialize(command_handle: CommandHandle,
                                          credentialdef_handle: u32,
                                          cb: Option<extern fn(xcommand_handle: CommandHandle, err: u32, credentialdef_state: *const c_char)>) -> u32 {
    info!("vcx_credentialdef_serialize >>>");

    check_useful_c_callback!(cb, VcxErrorKind::InvalidOption);

    let source_id = credential_def::get_source_id(credentialdef_handle).unwrap_or_default();
    trace!("vcx_credentialdef_serialize(command_handle: {}, credentialdef_handle: {}), source_id: {:?}",
           command_handle, credentialdef_handle, source_id);

    if !credential_def::is_valid_handle(credentialdef_handle) {
        return VcxError::from(VcxErrorKind::InvalidCredDefHandle).into();
    };

    execute(move || {
        match credential_def::to_string(credentialdef_handle) {
            Ok(err) => {
                trace!("vcx_credentialdef_serialize_cb(command_handle: {}, credentialdef_handle: {}, rc: {}, state: {}), source_id: {:?}",
                       command_handle, credentialdef_handle, error::SUCCESS.message, err, source_id);
                let msg = CStringUtils::string_to_cstring(err);
                cb(command_handle, error::SUCCESS.code_num, msg.as_ptr());
            }
            Err(err) => {
                set_current_error_vcx(&err);
                error!("vcx_credentialdef_serialize_cb(command_handle: {}, credentialdef_handle: {}, rc: {}, state: {}), source_id: {:?}",
                      command_handle, credentialdef_handle, err, "null", source_id);
                cb(command_handle, err.into(), ptr::null_mut());
            }
        };

        Ok(())
    });

    error::SUCCESS.code_num
}

/// Takes a json string representing a credentialdef object and recreates an object matching the json
///
/// #Params
/// command_handle: command handle to map callback to user context.
///
/// credentialdef_data: json string representing a credentialdef object
///
/// cb: Callback that provides credentialdef handle and provides error status
///
/// #Returns
/// Error code as a u32
#[no_mangle]
pub extern fn vcx_credentialdef_deserialize(command_handle: CommandHandle,
                                            credentialdef_data: *const c_char,
                                            cb: Option<extern fn(xcommand_handle: CommandHandle, err: u32, credentialdef_handle: u32)>) -> u32 {
    info!("vcx_credentialdef_deserialize >>>");

    check_useful_c_callback!(cb, VcxErrorKind::InvalidOption);
    check_useful_c_str!(credentialdef_data, VcxErrorKind::InvalidOption);

    trace!("vcx_credentialdef_deserialize(command_handle: {}, credentialdef_data: {})", command_handle, credentialdef_data);

    execute(move || {
        let (rc, handle) = match credential_def::from_string(&credentialdef_data) {
            Ok(err) => {
                trace!("vcx_credentialdef_deserialize_cb(command_handle: {}, rc: {}, handle: {}), source_id: {}",
                       command_handle, error::SUCCESS.message, err, credential_def::get_source_id(err).unwrap_or_default());
                (error::SUCCESS.code_num, err)
            }
            Err(err) => {
                set_current_error_vcx(&err);
                error!("vcx_credentialdef_deserialize_cb(command_handle: {}, rc: {}, handle: {}), source_id: {}",
                      command_handle, err, 0, "");
                (err.into(), 0)
            }
        };
        cb(command_handle, rc, handle);

        Ok(())
    });

    error::SUCCESS.code_num
}

/// Retrieves credential definition's id
///
/// #Params
/// cred_def_handle: CredDef handle that was provided during creation. Used to access proof object
///
/// cb: Callback that provides credential definition id and provides error status
///
/// #Returns
/// Error code as a u32
#[no_mangle]
pub extern fn vcx_credentialdef_get_cred_def_id(command_handle: CommandHandle,
                                                cred_def_handle: u32,
                                                cb: Option<extern fn(xcommand_handle: CommandHandle, err: u32, cred_def_id: *const c_char)>) -> u32 {
    info!("vcx_credentialdef_get_cred_def_id >>>");

    check_useful_c_callback!(cb, VcxErrorKind::InvalidOption);

    let source_id = credential_def::get_source_id(cred_def_handle).unwrap_or_default();
    trace!("vcx_credentialdef_get_cred_def_id(command_handle: {}, cred_def_handle: {}) source_id: {}", command_handle, cred_def_handle, source_id);
    if !credential_def::is_valid_handle(cred_def_handle) {
        return VcxError::from(VcxErrorKind::InvalidCredDefHandle).into();
    }

    execute(move || {
        match credential_def::get_cred_def_id(cred_def_handle) {
            Ok(err) => {
                trace!("vcx_credentialdef_get_cred_def_id(command_handle: {}, cred_def_handle: {}, rc: {}, cred_def_id: {}) source_id: {}",
                       command_handle, cred_def_handle, error::SUCCESS.message, err, source_id);
                let msg = CStringUtils::string_to_cstring(err);
                cb(command_handle, error::SUCCESS.code_num, msg.as_ptr());
            }
            Err(err) => {
                set_current_error_vcx(&err);
                error!("vcx_credentialdef_get_cred_def_id(command_handle: {}, cred_def_handle: {}, rc: {}, cred_def_id: {}) source_id: {}",
                      command_handle, cred_def_handle, err, "", source_id);
                cb(command_handle, err.into(), ptr::null_mut());
            }
        };

        Ok(())
    });

    error::SUCCESS.code_num
}

/// Releases the credentialdef object by de-allocating memory
///
/// #Params
/// handle: Proof handle that was provided during creation. Used to access credential object
///
/// #Returns
/// Success
#[no_mangle]
pub extern fn vcx_credentialdef_release(credentialdef_handle: u32) -> u32 {
    info!("vcx_credentialdef_release >>>");

    let source_id = credential_def::get_source_id(credentialdef_handle).unwrap_or_default();
    match credential_def::release(credentialdef_handle) {
        Ok(()) => {
            trace!("vcx_credentialdef_release(credentialdef_handle: {}, rc: {}), source_id: {}",
                   credentialdef_handle, error::SUCCESS.message, source_id);
            error::SUCCESS.code_num
        }

        Err(err) => {
            set_current_error_vcx(&err);
            error!("vcx_credentialdef_release(credentialdef_handle: {}, rc: {}), source_id: {}",
                  credentialdef_handle, err, source_id);
            err.into()
        }
    }
}

/// Checks if credential definition is published on the Ledger and updates the state if it is.
///
/// #Params
/// command_handle: command handle to map callback to user context.
///
/// credentialdef_handle: Credentialdef handle that was provided during creation. Used to access credentialdef object
///
/// cb: Callback that provides most current state of the credential definition and error status of request
///     States:
///         0 = Built
///         1 = Published
///
/// #Returns
/// Error code as a u32
#[no_mangle]
pub extern fn vcx_credentialdef_update_state(command_handle: CommandHandle,
                                             credentialdef_handle: u32,
                                             cb: Option<extern fn(xcommand_handle: CommandHandle, err: u32, state: u32)>) -> u32 {
    info!("vcx_credentialdef_update_state >>>");

    check_useful_c_callback!(cb, VcxErrorKind::InvalidOption);

    let source_id = credential_def::get_source_id(credentialdef_handle).unwrap_or_default();
    trace!("vcx_credentialdef_update_state(command_handle: {}, credentialdef_handle: {}) source_id: {}",
           command_handle, credentialdef_handle, source_id);

    if !credential_def::is_valid_handle(credentialdef_handle) {
        return VcxError::from(VcxErrorKind::InvalidCredDefHandle).into();
    }

    execute_async::<BoxFuture<'static, Result<(), ()>>>(Box::pin(async move {
        match credential_def::update_state(credentialdef_handle).await {
            Ok(state) => {
                trace!("vcx_credentialdef_update_state(command_handle: {}, rc: {}, state: {})",
                       command_handle, error::SUCCESS.message, state);
                cb(command_handle, error::SUCCESS.code_num, state);
            }
            Err(err) => {
                set_current_error_vcx(&err);
                error!("vcx_credentialdef_update_state(command_handle: {}, rc: {}, state: {})",
                      command_handle, err, 0);
                cb(command_handle, err.into(), 0);
            }
        };

        Ok(())
    }));

    error::SUCCESS.code_num
}

/// Get the current state of the credential definition object
///
/// #Params
/// command_handle: command handle to map callback to user context.
///
/// credentialdef_handle: Credentialdef handle that was provided during creation. Used to access credentialdef object
///
/// cb: Callback that provides most current state of the credential definition and error status of request
///     States:
///         0 = Built
///         1 = Published
///
/// #Returns
/// Error code as a u32
#[no_mangle]
pub extern fn vcx_credentialdef_get_state(command_handle: CommandHandle,
                                          credentialdef_handle: u32,
                                          cb: Option<extern fn(xcommand_handle: CommandHandle, err: u32, state: u32)>) -> u32 {
    info!("vcx_credentialdef_get_state >>>");

    check_useful_c_callback!(cb, VcxErrorKind::InvalidOption);

    let source_id = credential_def::get_source_id(credentialdef_handle).unwrap_or_default();
    trace!("vcx_credentialdef_get_state(command_handle: {}, credentialdef_handle: {}) source_id: {}",
           command_handle, credentialdef_handle, source_id);

    if !credential_def::is_valid_handle(credentialdef_handle) {
        return VcxError::from(VcxErrorKind::InvalidCredDefHandle).into();
    }

    execute(move || {
        match credential_def::get_state(credentialdef_handle) {
            Ok(state) => {
                trace!("vcx_credentialdef_get_state(command_handle: {}, rc: {}, state: {})",
                       command_handle, error::SUCCESS.message, state);
                cb(command_handle, error::SUCCESS.code_num, state);
            }
            Err(err) => {
                set_current_error_vcx(&err);
                error!("vcx_credentialdef_get_state(command_handle: {}, rc: {}, state: {})",
                      command_handle, err, 0);
                cb(command_handle, err.into(), 0);
            }
        };

        Ok(())
    });

    error::SUCCESS.code_num
}

#[no_mangle]
pub extern fn vcx_credentialdef_rotate_rev_reg_def(command_handle: CommandHandle,
                                                   credentialdef_handle: u32,
                                                   revocation_details: *const c_char,
                                                   cb: Option<extern fn(xcommand_handle: CommandHandle, err: u32, credentialdef_state: *const c_char)>) -> u32 {
    info!("vcx_credentialdef_rotate_rev_reg_def >>>");

    check_useful_c_callback!(cb, VcxErrorKind::InvalidOption);
    check_useful_c_str!(revocation_details, VcxErrorKind::InvalidOption);

    let source_id = credential_def::get_source_id(credentialdef_handle).unwrap_or_default();
    trace!("vcx_credentialdef_rotate_rev_reg_def(command_handle: {}, credentialdef_handle: {}, revocation_details: {}) source_id: {}",
           command_handle, credentialdef_handle, revocation_details, source_id);

    if !credential_def::is_valid_handle(credentialdef_handle) {
        return VcxError::from(VcxErrorKind::InvalidCredDefHandle).into();
    }

    execute_async::<BoxFuture<'static, Result<(), ()>>>(Box::pin(async move {
        match credential_def::rotate_rev_reg_def(credentialdef_handle, &revocation_details).await {
            Ok(err) => {
                trace!("vcx_credentialdef_rotate_rev_reg_def(command_handle: {}, credentialdef_handle: {}, rc: {}, rev_reg_def: {}), source_id: {:?}",
                       command_handle, credentialdef_handle, error::SUCCESS.message, err, source_id);
                let msg = CStringUtils::string_to_cstring(err);
                cb(command_handle, error::SUCCESS.code_num, msg.as_ptr());
            }
            Err(err) => {
                set_current_error_vcx(&err);
                error!("vcx_credentialdef_rotate_rev_reg_def(command_handle: {}, credentialdef_handle: {}, rc: {}, rev_reg_def: {}), source_id: {:?}",
                      command_handle, credentialdef_handle, err, "null", source_id);
                cb(command_handle, err.into(), ptr::null_mut());
            }
        };

        Ok(())
    }));

    error::SUCCESS.code_num
}

#[no_mangle]
pub extern fn vcx_credentialdef_publish_revocations(command_handle: CommandHandle,
                                                    credentialdef_handle: u32,
                                                    cb: Option<extern fn(xcommand_handle: CommandHandle, err: u32)>) -> u32 {
    info!("vcx_credentialdef_publish_revocations >>>");

    check_useful_c_callback!(cb, VcxErrorKind::InvalidOption);

    let source_id = credential_def::get_source_id(credentialdef_handle).unwrap_or_default();

    trace!("vcx_credentialdef_get_state(command_handle: {}, credentialdef_handle: {}) source_id: {}",
           command_handle, credentialdef_handle, source_id);

    if !credential_def::is_valid_handle(credentialdef_handle) {
        return VcxError::from(VcxErrorKind::InvalidCredDefHandle).into();
    }

    execute_async::<BoxFuture<'static, Result<(), ()>>>(Box::pin(async move {
        match credential_def::publish_revocations(credentialdef_handle).await {
            Ok(()) => {
                trace!("vcx_credentialdef_publish_revocations(command_handle: {}, credentialdef_handle: {}, rc: {})",
                       command_handle, credentialdef_handle, error::SUCCESS.message);
                cb(command_handle, error::SUCCESS.code_num);
            }
            Err(err) => {
                set_current_error_vcx(&err);
                error!("vcx_credentialdef_publish_revocations(command_handle: {}, credentialdef_handle: {}, rc: {})",
                      command_handle, credentialdef_handle, err);
                cb(command_handle, err.into());
            }
        };

        Ok(())
    }));

    error::SUCCESS.code_num
}

#[no_mangle]
pub extern fn vcx_credentialdef_get_tails_hash(command_handle: CommandHandle,
                                               handle: u32,
                                               cb: Option<extern fn(xcommand_handle: CommandHandle, err: u32, hash: *const c_char)>) -> u32 {
    info!("vcx_credentialdef_get_tails_hash >>>");

    check_useful_c_callback!(cb, VcxErrorKind::InvalidOption);

    let source_id = credential_def::get_source_id(handle).unwrap_or_default();
    trace!("vcx_credentialdef_get_tails_hash(command_handle: {}) source_id: {}", command_handle, source_id);

    execute(move || {
        match credential_def::get_tails_hash(handle) {
            Ok(err) => {
                trace!("vcx_credentialdef_get_tails_hash_cb(command_handle: {}, rc: {}, hash: {}), source_id: {}",
                       command_handle, error::SUCCESS.message, err, credential_def::get_source_id(handle).unwrap_or_default());

                let hash = CStringUtils::string_to_cstring(err);
                cb(command_handle, 0, hash.as_ptr());
            }
            Err(err) => {
                set_current_error_vcx(&err);
                error!("vcx_credentialdef_get_tails_hash_cb(command_handle: {}, rc: {}, hash: {}), source_id: {}",
                       command_handle, err, "null", credential_def::get_source_id(handle).unwrap_or_default());
                cb(command_handle, err.into(), ptr::null());
            }
        };

        Ok(())
    });

    error::SUCCESS.code_num
}

#[no_mangle]
pub extern fn vcx_credentialdef_get_rev_reg_id(command_handle: CommandHandle,
                                               handle: u32,
                                               cb: Option<extern fn(xcommand_handle: CommandHandle, err: u32, rev_reg_id: *const c_char)>) -> u32 {
    info!("vcx_credentialdef_get_rev_reg_id >>>");

    check_useful_c_callback!(cb, VcxErrorKind::InvalidOption);

    let source_id = credential_def::get_source_id(handle).unwrap_or_default();
    trace!("vcx_credentialdef_get_rev_reg_id(command_handle: {}) source_id: {}", command_handle, source_id);

    execute(move || {
        match credential_def::get_rev_reg_id(handle) {
            Ok(err) => {
                trace!("vcx_credentialdef_get_rev_reg_id_cb(command_handle: {}, rc: {}, rev_reg_id: {}), source_id: {}",
                       command_handle, error::SUCCESS.message, err, credential_def::get_source_id(handle).unwrap_or_default());

                let rev_reg_id = CStringUtils::string_to_cstring(err);
                cb(command_handle, 0, rev_reg_id.as_ptr());
            }
            Err(err) => {
                set_current_error_vcx(&err);
                error!("vcx_credentialdef_get_rev_reg_id(command_handle: {}, rc: {}, rev_reg_id: {}), source_id: {}",
                       command_handle, err, "null", credential_def::get_source_id(handle).unwrap_or_default());
                cb(command_handle, err.into(), ptr::null());
            }
        };

        Ok(())
    });

    error::SUCCESS.code_num
}

#[cfg(test)]
mod tests {
    use std::ffi::CString;

    use aries_vcx::utils::constants::SCHEMA_ID;
    use aries_vcx::utils::devsetup::{SetupLibraryWallet, SetupMocks};

    use crate::api_lib::utils::return_types_u32;
    use crate::api_lib::utils::timeout::TimeoutUtils;

    use super::*;

    #[test]
    #[cfg(feature = "general_test")]
    fn test_vcx_create_credentialdef_success() {
        let _setup = SetupMocks::init();

        let cb = return_types_u32::Return_U32_U32::new().unwrap();
        assert_eq!(vcx_credentialdef_create_and_store(cb.command_handle,
                                                      CString::new("Test Source ID").unwrap().into_raw(),
                                                      CString::new(SCHEMA_ID).unwrap().into_raw(),
                                                      CString::new("6vkhW3L28AophhA68SSzRS").unwrap().into_raw(),
                                                      CString::new("tag").unwrap().into_raw(),
                                                      CString::new("{}").unwrap().into_raw(),
                                                      Some(cb.get_callback())), error::SUCCESS.code_num);
        cb.receive(TimeoutUtils::some_medium()).unwrap();
    }

    #[tokio::test]
    #[cfg(feature = "general_test")]
    async fn test_vcx_create_credentialdef_fails() {
        let _setup = SetupLibraryWallet::init().await;

        let cb = return_types_u32::Return_U32_U32::new().unwrap();
        assert_eq!(vcx_credentialdef_create_and_store(cb.command_handle,
                                                      CString::new("Test Source ID").unwrap().into_raw(),
                                                      CString::new(SCHEMA_ID).unwrap().into_raw(),
                                                      ptr::null(),
                                                      CString::new("tag").unwrap().into_raw(),
                                                      CString::new("{}").unwrap().into_raw(),
                                                      Some(cb.get_callback())), error::SUCCESS.code_num);
        assert!(cb.receive(TimeoutUtils::some_medium()).is_err());
    }

    #[test]
    #[cfg(feature = "general_test")]
    fn test_vcx_credentialdef_serialize() {
        let _setup = SetupMocks::init();

        let cb = return_types_u32::Return_U32_U32::new().unwrap();
        assert_eq!(vcx_credentialdef_create_and_store(cb.command_handle,
                                                      CString::new("Test Source ID").unwrap().into_raw(),
                                                      CString::new(SCHEMA_ID).unwrap().into_raw(),
                                                      ptr::null(),
                                                      CString::new("tag").unwrap().into_raw(),
                                                      CString::new("{}").unwrap().into_raw(),
                                                      Some(cb.get_callback())), error::SUCCESS.code_num);

        let handle = cb.receive(TimeoutUtils::some_medium()).unwrap();
        let cb = return_types_u32::Return_U32_STR::new().unwrap();
        assert_eq!(vcx_credentialdef_serialize(cb.command_handle, handle, Some(cb.get_callback())), error::SUCCESS.code_num);
        let cred = cb.receive(TimeoutUtils::some_medium()).unwrap();
        assert!(cred.is_some());
    }

    #[test]
    #[cfg(feature = "general_test")]
    fn test_vcx_credentialdef_deserialize_succeeds() {
        let _setup = SetupMocks::init();

        let cb = return_types_u32::Return_U32_U32::new().unwrap();

        let original = r#"{"version":"1.0", "data": {"cred_def_id":"2hoqvcwupRTUNkXn6ArYzs:3:CL:1697","issuer_did":"2hoqvcwupRTUNkXn6ArYzs","tag":"tag","rev_ref_def":null,"rev_reg_entry":null,"rev_reg_id":null,"source_id":"SourceId","cred_def_json":""}}"#;
        assert_eq!(vcx_credentialdef_deserialize(cb.command_handle,
                                                 CString::new(original).unwrap().into_raw(),
                                                 Some(cb.get_callback())), error::SUCCESS.code_num);

        let handle = cb.receive(TimeoutUtils::some_short()).unwrap();
        assert!(handle > 0);
    }

    #[test]
    #[cfg(feature = "general_test")]
    fn test_vcx_credentialdef_deserialize_succeeds_with_old_data() {
        let _setup = SetupMocks::init();

        let cb = return_types_u32::Return_U32_U32::new().unwrap();

        let original = r#"{"data":{"cred_def_id":"V4SGRU86Z58d6TV7PBUe6f:3:CL:912:tag1","payment_txn":null,"source_id":"1","tag":"tag1","issuer_did":"66Fh8yBzrpJQmNyZzgoTqB","cred_def_json":""},"version":"1.0"}"#;
        assert_eq!(vcx_credentialdef_deserialize(cb.command_handle,
                                                 CString::new(original).unwrap().into_raw(),
                                                 Some(cb.get_callback())), error::SUCCESS.code_num);

        let handle = cb.receive(TimeoutUtils::some_short()).unwrap();
        assert!(handle > 0);
    }


    #[test]
    #[cfg(feature = "general_test")]
    fn test_vcx_credentialdef_release() {
        let _setup = SetupMocks::init();

        let cb = return_types_u32::Return_U32_U32::new().unwrap();
        assert_eq!(vcx_credentialdef_create_and_store(cb.command_handle,
                                                      CString::new("Test Source ID Release Test").unwrap().into_raw(),
                                                      CString::new(SCHEMA_ID).unwrap().into_raw(),
                                                      ptr::null(),
                                                      CString::new("tag").unwrap().into_raw(),
                                                      CString::new("{}").unwrap().into_raw(),
                                                      Some(cb.get_callback())), error::SUCCESS.code_num);

        let handle = cb.receive(TimeoutUtils::some_medium()).unwrap();
        let unknown_handle = handle + 1;
        assert_eq!(vcx_credentialdef_release(unknown_handle), error::INVALID_CREDENTIAL_DEF_HANDLE.code_num);
    }


    #[test]
    #[cfg(feature = "general_test")]
    fn test_vcx_creddef_get_id() {
        let _setup = SetupMocks::init();

        let cb = return_types_u32::Return_U32_U32::new().unwrap();
        assert_eq!(vcx_credentialdef_create_and_store(cb.command_handle,
                                                      CString::new("Test Source ID").unwrap().into_raw(),
                                                      CString::new(SCHEMA_ID).unwrap().into_raw(),
                                                      CString::new("6vkhW3L28AophhA68SSzRS").unwrap().into_raw(),
                                                      CString::new("tag").unwrap().into_raw(),
                                                      CString::new("{}").unwrap().into_raw(),
                                                      Some(cb.get_callback())), error::SUCCESS.code_num);
        let handle = cb.receive(TimeoutUtils::some_medium()).unwrap();
        let cb = return_types_u32::Return_U32_STR::new().unwrap();
        assert_eq!(vcx_credentialdef_get_cred_def_id(cb.command_handle, handle, Some(cb.get_callback())), error::SUCCESS.code_num);
        cb.receive(TimeoutUtils::some_medium()).unwrap();
    }

    // TODO: Update to not use prepare_credentialdef_for_endorser if possible
    #[test]
    #[cfg(feature = "to_restore")]
    #[cfg(feature = "general_test")]
    fn test_vcx_cred_def_get_state() {
        let _setup = SetupMocks::init();

        let (handle, _, _, _) = credential_def::prepare_credentialdef_for_endorser("testid".to_string(),
                                                                                   "Test Credential Def".to_string(),
                                                                                   "6vkhW3L28AophhA68SSzRS".to_string(),
                                                                                   SCHEMA_ID.to_string(),
                                                                                   "tag".to_string(),
                                                                                   "{}".to_string(),
                                                                                   "V4SGRU86Z58d6TV7PBUe6f".to_string()).unwrap();
        {
            let cb = return_types_u32::Return_U32_U32::new().unwrap();
            let _rc = vcx_credentialdef_get_state(cb.command_handle, handle, Some(cb.get_callback()));
            assert_eq!(cb.receive(TimeoutUtils::some_medium()).unwrap(), PublicEntityStateType::Built as u32)
        }
        {
            let cb = return_types_u32::Return_U32_U32::new().unwrap();
            let _rc = vcx_credentialdef_update_state(cb.command_handle, handle, Some(cb.get_callback()));
            assert_eq!(cb.receive(TimeoutUtils::some_medium()).unwrap(), PublicEntityStateType::Published as u32);
        }
        {
            let cb = return_types_u32::Return_U32_U32::new().unwrap();
            let _rc = vcx_credentialdef_get_state(cb.command_handle, handle, Some(cb.get_callback()));
            assert_eq!(cb.receive(TimeoutUtils::some_medium()).unwrap(), PublicEntityStateType::Published as u32)
        }
    }
}
