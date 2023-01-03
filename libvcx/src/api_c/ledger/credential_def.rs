use std::ptr;

use futures::future::BoxFuture;
use libc::c_char;

use crate::api_c::cutils::cstring::CStringUtils;
use crate::api_c::cutils::current_error::set_current_error_vcx;
use crate::api_c::cutils::runtime::{execute, execute_async};
use crate::api_c::types::CommandHandle;

use crate::api_vcx::api_handle::credential_def;
use crate::errors::error;
use crate::errors::error::{LibvcxError, LibvcxErrorKind};

#[no_mangle]
pub extern "C" fn vcx_credentialdef_create_v2(
    command_handle: CommandHandle,
    source_id: *const c_char,
    schema_id: *const c_char,
    tag: *const c_char,
    support_revocation: bool,
    cb: Option<extern "C" fn(xcommand_handle: CommandHandle, err: u32, credentialdef_handle: u32)>,
) -> u32 {
    info!("vcx_credentialdef_create_v2 >>>");

    check_useful_c_callback!(cb, LibvcxErrorKind::InvalidOption);
    check_useful_c_str!(source_id, LibvcxErrorKind::InvalidOption);
    check_useful_c_str!(schema_id, LibvcxErrorKind::InvalidOption);
    check_useful_c_str!(tag, LibvcxErrorKind::InvalidOption);

    trace!("vcx_credentialdef_create_v2(command_handle: {}, source_id: {}, schema_id: {}, tag: {}, support_revocation: {:?})", command_handle, source_id, schema_id, tag, support_revocation);
    execute_async::<BoxFuture<'static, Result<(), ()>>>(Box::pin(async move {
        let (rc, handle) = match credential_def::create(source_id, schema_id, tag, support_revocation).await {
            Ok(handle) => {
                trace!("vcx_credentialdef_create_v2_cb(command_handle: {}, rc: {}, credentialdef_handle: {}), source_id: {:?}", command_handle, error::SUCCESS_ERR_CODE, handle, credential_def::get_source_id(handle).unwrap_or_default());
                (error::SUCCESS_ERR_CODE, handle)
            }
            Err(err) => {
                set_current_error_vcx(&err);
                error!("vcx_credentialdef_create_v2_cb(command_handle: {}, rc: {}, credentialdef_handle: {}), source_id: {:?}", command_handle, err, 0, "");
                (err.into(), 0)
            }
        };
        cb(command_handle, rc, handle);

        Ok(())
    }));

    error::SUCCESS_ERR_CODE
}

#[no_mangle]
pub extern "C" fn vcx_credentialdef_publish(
    command_handle: CommandHandle,
    credentialdef_handle: u32,
    _tails_url: *const c_char,
    cb: Option<extern "C" fn(xcommand_handle: CommandHandle, err: u32)>,
) -> u32 {
    info!("vcx_credentialdef_publish >>>");

    check_useful_c_callback!(cb, LibvcxErrorKind::InvalidOption);

    if !credential_def::is_valid_handle(credentialdef_handle) {
        return LibvcxError::from_msg(
            LibvcxErrorKind::InvalidCredDefHandle,
            format!("Invalid creddef handle {}", credentialdef_handle),
        )
        .into();
    };

    let source_id = credential_def::get_source_id(credentialdef_handle).unwrap_or_default();
    trace!(
        "vcx_credentialdef_publish(command_handle: {}, credentialdef_handle: {}, source_id: {:?}",
        command_handle,
        credentialdef_handle,
        source_id
    );

    execute_async::<BoxFuture<'static, Result<(), ()>>>(Box::pin(async move {
        match credential_def::publish(credentialdef_handle).await {
            Ok(_) => {
                trace!("vcx_credentialdef_publish_cb(command_handle: {}, rc: {}, credentialdef_handle: {}), source_id: {:?}", command_handle, error::SUCCESS_ERR_CODE, credentialdef_handle, source_id);
                cb(command_handle, error::SUCCESS_ERR_CODE);
            }
            Err(err) => {
                set_current_error_vcx(&err);
                error!("vcx_credentialdef_publish_cb(command_handle: {}, rc: {}, credentialdef_handle: {}), source_id: {:?}", command_handle, err, credentialdef_handle, source_id);
                cb(command_handle, err.into());
            }
        };
        Ok(())
    }));

    error::SUCCESS_ERR_CODE
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
pub extern "C" fn vcx_credentialdef_serialize(
    command_handle: CommandHandle,
    credentialdef_handle: u32,
    cb: Option<extern "C" fn(xcommand_handle: CommandHandle, err: u32, credentialdef_state: *const c_char)>,
) -> u32 {
    info!("vcx_credentialdef_serialize >>>");

    check_useful_c_callback!(cb, LibvcxErrorKind::InvalidOption);

    let source_id = credential_def::get_source_id(credentialdef_handle).unwrap_or_default();
    trace!(
        "vcx_credentialdef_serialize(command_handle: {}, credentialdef_handle: {}), source_id: {:?}",
        command_handle,
        credentialdef_handle,
        source_id
    );

    if !credential_def::is_valid_handle(credentialdef_handle) {
        return LibvcxError::from_msg(
            LibvcxErrorKind::InvalidCredDefHandle,
            format!("Invalid creddef handle {}", credentialdef_handle),
        )
        .into();
    };

    execute(move || {
        match credential_def::to_string(credentialdef_handle) {
            Ok(err) => {
                trace!("vcx_credentialdef_serialize_cb(command_handle: {}, credentialdef_handle: {}, rc: {}, state: {}), source_id: {:?}", command_handle, credentialdef_handle, error::SUCCESS_ERR_CODE, err, source_id);
                let msg = CStringUtils::string_to_cstring(err);
                cb(command_handle, error::SUCCESS_ERR_CODE, msg.as_ptr());
            }
            Err(err) => {
                set_current_error_vcx(&err);
                error!("vcx_credentialdef_serialize_cb(command_handle: {}, credentialdef_handle: {}, rc: {}, state: {}), source_id: {:?}", command_handle, credentialdef_handle, err, "null", source_id);
                cb(command_handle, err.into(), ptr::null_mut());
            }
        };

        Ok(())
    });

    error::SUCCESS_ERR_CODE
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
pub extern "C" fn vcx_credentialdef_deserialize(
    command_handle: CommandHandle,
    credentialdef_data: *const c_char,
    cb: Option<extern "C" fn(xcommand_handle: CommandHandle, err: u32, credentialdef_handle: u32)>,
) -> u32 {
    info!("vcx_credentialdef_deserialize >>>");

    check_useful_c_callback!(cb, LibvcxErrorKind::InvalidOption);
    check_useful_c_str!(credentialdef_data, LibvcxErrorKind::InvalidOption);

    trace!(
        "vcx_credentialdef_deserialize(command_handle: {}, credentialdef_data: {})",
        command_handle,
        credentialdef_data
    );

    execute(move || {
        let (rc, handle) = match credential_def::from_string(&credentialdef_data) {
            Ok(err) => {
                trace!(
                    "vcx_credentialdef_deserialize_cb(command_handle: {}, rc: {}, handle: {}), source_id: {}",
                    command_handle,
                    error::SUCCESS_ERR_CODE,
                    err,
                    credential_def::get_source_id(err).unwrap_or_default()
                );
                (error::SUCCESS_ERR_CODE, err)
            }
            Err(err) => {
                set_current_error_vcx(&err);
                error!(
                    "vcx_credentialdef_deserialize_cb(command_handle: {}, rc: {}, handle: {}), source_id: {}",
                    command_handle, err, 0, ""
                );
                (err.into(), 0)
            }
        };
        cb(command_handle, rc, handle);

        Ok(())
    });

    error::SUCCESS_ERR_CODE
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
pub extern "C" fn vcx_credentialdef_get_cred_def_id(
    command_handle: CommandHandle,
    cred_def_handle: u32,
    cb: Option<extern "C" fn(xcommand_handle: CommandHandle, err: u32, cred_def_id: *const c_char)>,
) -> u32 {
    info!("vcx_credentialdef_get_cred_def_id >>>");

    check_useful_c_callback!(cb, LibvcxErrorKind::InvalidOption);

    let source_id = credential_def::get_source_id(cred_def_handle).unwrap_or_default();
    trace!(
        "vcx_credentialdef_get_cred_def_id(command_handle: {}, cred_def_handle: {}) source_id: {}",
        command_handle,
        cred_def_handle,
        source_id
    );
    if !credential_def::is_valid_handle(cred_def_handle) {
        return LibvcxError::from_msg(
            LibvcxErrorKind::InvalidCredDefHandle,
            format!("Invalid creddef handle {}", cred_def_handle),
        )
        .into();
    }

    execute(move || {
        match credential_def::get_cred_def_id(cred_def_handle) {
            Ok(err) => {
                trace!("vcx_credentialdef_get_cred_def_id(command_handle: {}, cred_def_handle: {}, rc: {}, cred_def_id: {}) source_id: {}", command_handle, cred_def_handle, error::SUCCESS_ERR_CODE, err, source_id);
                let msg = CStringUtils::string_to_cstring(err);
                cb(command_handle, error::SUCCESS_ERR_CODE, msg.as_ptr());
            }
            Err(err) => {
                set_current_error_vcx(&err);
                error!("vcx_credentialdef_get_cred_def_id(command_handle: {}, cred_def_handle: {}, rc: {}, cred_def_id: {}) source_id: {}", command_handle, cred_def_handle, err, "", source_id);
                cb(command_handle, err.into(), ptr::null_mut());
            }
        };

        Ok(())
    });

    error::SUCCESS_ERR_CODE
}

/// Releases the credentialdef object by de-allocating memory
///
/// #Params
/// handle: Proof handle that was provided during creation. Used to access credential object
///
/// #Returns
/// Success
#[no_mangle]
pub extern "C" fn vcx_credentialdef_release(credentialdef_handle: u32) -> u32 {
    info!("vcx_credentialdef_release >>>");

    let source_id = credential_def::get_source_id(credentialdef_handle).unwrap_or_default();
    match credential_def::release(credentialdef_handle) {
        Ok(()) => {
            trace!(
                "vcx_credentialdef_release(credentialdef_handle: {}, rc: {}), source_id: {}",
                credentialdef_handle,
                error::SUCCESS_ERR_CODE,
                source_id
            );
            error::SUCCESS_ERR_CODE
        }

        Err(err) => {
            set_current_error_vcx(&err);
            error!(
                "vcx_credentialdef_release(credentialdef_handle: {}, rc: {}), source_id: {}",
                credentialdef_handle, err, source_id
            );
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
pub extern "C" fn vcx_credentialdef_update_state(
    command_handle: CommandHandle,
    credentialdef_handle: u32,
    cb: Option<extern "C" fn(xcommand_handle: CommandHandle, err: u32, state: u32)>,
) -> u32 {
    info!("vcx_credentialdef_update_state >>>");

    check_useful_c_callback!(cb, LibvcxErrorKind::InvalidOption);

    let source_id = credential_def::get_source_id(credentialdef_handle).unwrap_or_default();
    trace!(
        "vcx_credentialdef_update_state(command_handle: {}, credentialdef_handle: {}) source_id: {}",
        command_handle,
        credentialdef_handle,
        source_id
    );

    if !credential_def::is_valid_handle(credentialdef_handle) {
        return LibvcxError::from_msg(
            LibvcxErrorKind::InvalidCredDefHandle,
            format!("Invalid creddef handle {}", credentialdef_handle),
        )
        .into();
    }

    execute_async::<BoxFuture<'static, Result<(), ()>>>(Box::pin(async move {
        match credential_def::update_state(credentialdef_handle).await {
            Ok(state) => {
                trace!(
                    "vcx_credentialdef_update_state(command_handle: {}, rc: {}, state: {})",
                    command_handle,
                    error::SUCCESS_ERR_CODE,
                    state
                );
                cb(command_handle, error::SUCCESS_ERR_CODE, state);
            }
            Err(err) => {
                set_current_error_vcx(&err);
                error!(
                    "vcx_credentialdef_update_state(command_handle: {}, rc: {}, state: {})",
                    command_handle, err, 0
                );
                cb(command_handle, err.into(), 0);
            }
        };

        Ok(())
    }));

    error::SUCCESS_ERR_CODE
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
pub extern "C" fn vcx_credentialdef_get_state(
    command_handle: CommandHandle,
    credentialdef_handle: u32,
    cb: Option<extern "C" fn(xcommand_handle: CommandHandle, err: u32, state: u32)>,
) -> u32 {
    info!("vcx_credentialdef_get_state >>>");

    check_useful_c_callback!(cb, LibvcxErrorKind::InvalidOption);

    let source_id = credential_def::get_source_id(credentialdef_handle).unwrap_or_default();
    trace!(
        "vcx_credentialdef_get_state(command_handle: {}, credentialdef_handle: {}) source_id: {}",
        command_handle,
        credentialdef_handle,
        source_id
    );

    if !credential_def::is_valid_handle(credentialdef_handle) {
        return LibvcxError::from_msg(
            LibvcxErrorKind::InvalidCredDefHandle,
            format!("Invalid creddef handle {}", credentialdef_handle),
        )
        .into();
    }

    execute(move || {
        match credential_def::get_state(credentialdef_handle) {
            Ok(state) => {
                trace!(
                    "vcx_credentialdef_get_state(command_handle: {}, rc: {}, state: {})",
                    command_handle,
                    error::SUCCESS_ERR_CODE,
                    state
                );
                cb(command_handle, error::SUCCESS_ERR_CODE, state);
            }
            Err(err) => {
                set_current_error_vcx(&err);
                error!(
                    "vcx_credentialdef_get_state(command_handle: {}, rc: {}, state: {})",
                    command_handle, err, 0
                );
                cb(command_handle, err.into(), 0);
            }
        };

        Ok(())
    });

    error::SUCCESS_ERR_CODE
}

#[cfg(feature = "general_test")]
#[cfg(test)]
mod tests {
    use std::ffi::CString;

    use aries_vcx::utils::constants::SCHEMA_ID;
    use aries_vcx::utils::devsetup::{SetupLibraryWallet, SetupMocks};

    use crate::api_c::cutils::return_types_u32;
    use crate::api_c::cutils::timeout::TimeoutUtils;
    use crate::errors::error;

    use super::*;

    #[test]
    #[cfg(feature = "general_test")]
    fn test_vcx_create_credentialdef_success() {
        let _setup = SetupMocks::init();

        let cb = return_types_u32::Return_U32_U32::new().unwrap();
        assert_eq!(
            vcx_credentialdef_create_v2(
                cb.command_handle,
                CString::new("Test Source ID").unwrap().into_raw(),
                CString::new(SCHEMA_ID).unwrap().into_raw(),
                CString::new("tag").unwrap().into_raw(),
                true,
                Some(cb.get_callback()),
            ),
            error::SUCCESS_ERR_CODE
        );
        cb.receive(TimeoutUtils::some_medium()).unwrap();
    }

    #[cfg(feature = "general_test")]
    #[tokio::test]
    async fn test_vcx_create_credentialdef_fails() {
        SetupLibraryWallet::run(|_setup| async {
            let cb = return_types_u32::Return_U32_U32::new().unwrap();
            assert_eq!(
                vcx_credentialdef_create_v2(
                    cb.command_handle,
                    CString::new("Test Source ID").unwrap().into_raw(),
                    CString::new(SCHEMA_ID).unwrap().into_raw(),
                    CString::new("tag").unwrap().into_raw(),
                    true,
                    Some(cb.get_callback()),
                ),
                error::SUCCESS_ERR_CODE
            );
            assert!(cb.receive(TimeoutUtils::some_medium()).is_err());
        })
        .await;
    }

    #[test]
    #[cfg(feature = "general_test")]
    fn test_vcx_credentialdef_serialize() {
        let _setup = SetupMocks::init();

        let cb = return_types_u32::Return_U32_U32::new().unwrap();
        assert_eq!(
            vcx_credentialdef_create_v2(
                cb.command_handle,
                CString::new("Test Source ID").unwrap().into_raw(),
                CString::new(SCHEMA_ID).unwrap().into_raw(),
                CString::new("tag").unwrap().into_raw(),
                true,
                Some(cb.get_callback()),
            ),
            error::SUCCESS_ERR_CODE
        );

        let handle = cb.receive(TimeoutUtils::some_medium()).unwrap();
        let cb = return_types_u32::Return_U32_STR::new().unwrap();
        assert_eq!(
            vcx_credentialdef_serialize(cb.command_handle, handle, Some(cb.get_callback())),
            error::SUCCESS_ERR_CODE
        );
        let cred = cb.receive(TimeoutUtils::some_medium()).unwrap();
        assert!(cred.is_some());
    }

    #[test]
    #[cfg(feature = "general_test")]
    fn test_vcx_credentialdef_deserialize_succeeds() {
        let _setup = SetupMocks::init();

        let cb = return_types_u32::Return_U32_U32::new().unwrap();

        let original = r#"{"version":"1.0", "data": {"cred_def_id":"2hoqvcwupRTUNkXn6ArYzs:3:CL:1697","issuer_did":"2hoqvcwupRTUNkXn6ArYzs","tag":"tag","rev_ref_def":null,"rev_reg_entry":null,"rev_reg_id":null,"source_id":"SourceId","cred_def_json":"","support_revocation": true}}"#;
        assert_eq!(
            vcx_credentialdef_deserialize(
                cb.command_handle,
                CString::new(original).unwrap().into_raw(),
                Some(cb.get_callback()),
            ),
            error::SUCCESS_ERR_CODE
        );

        let handle = cb.receive(TimeoutUtils::some_short()).unwrap();
        assert!(handle > 0);
    }

    #[test]
    #[cfg(feature = "general_test")]
    fn test_vcx_credentialdef_deserialize_succeeds_with_old_data() {
        let _setup = SetupMocks::init();

        let cb = return_types_u32::Return_U32_U32::new().unwrap();

        let original = r#"{"data":{"cred_def_id":"V4SGRU86Z58d6TV7PBUe6f:3:CL:912:tag1","payment_txn":null,"source_id":"1","tag":"tag1","issuer_did":"66Fh8yBzrpJQmNyZzgoTqB","cred_def_json":"","support_revocation": true},"version":"1.0"}"#;
        assert_eq!(
            vcx_credentialdef_deserialize(
                cb.command_handle,
                CString::new(original).unwrap().into_raw(),
                Some(cb.get_callback()),
            ),
            error::SUCCESS_ERR_CODE
        );

        let handle = cb.receive(TimeoutUtils::some_short()).unwrap();
        assert!(handle > 0);
    }

    #[test]
    #[cfg(feature = "general_test")]
    fn test_vcx_creddef_get_id() {
        let _setup = SetupMocks::init();

        let cb = return_types_u32::Return_U32_U32::new().unwrap();
        assert_eq!(
            vcx_credentialdef_create_v2(
                cb.command_handle,
                CString::new("Test Source ID").unwrap().into_raw(),
                CString::new(SCHEMA_ID).unwrap().into_raw(),
                CString::new("tag").unwrap().into_raw(),
                true,
                Some(cb.get_callback()),
            ),
            error::SUCCESS_ERR_CODE
        );
        let handle = cb.receive(TimeoutUtils::some_medium()).unwrap();
        let cb = return_types_u32::Return_U32_STR::new().unwrap();
        assert_eq!(
            vcx_credentialdef_get_cred_def_id(cb.command_handle, handle, Some(cb.get_callback())),
            error::SUCCESS_ERR_CODE
        );
        cb.receive(TimeoutUtils::some_medium()).unwrap();
    }
}
