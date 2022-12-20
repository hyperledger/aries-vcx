use std::ptr;

use futures::future::BoxFuture;
use libc::c_char;

use aries_vcx::global::settings;
use aries_vcx::vdrtools::CommandHandle;

use crate::api_lib::api_handle::{revocation_registry, revocation_registry::RevocationRegistryConfig, vcx_settings};
use crate::api_lib::errors::error_libvcx;
use crate::api_lib::errors::error_libvcx::{LibvcxError, LibvcxErrorKind};
use crate::api_lib::utils::cstring::CStringUtils;
use crate::api_lib::utils::current_error::{set_current_error, set_current_error_vcx};
use crate::api_lib::utils::libvcx_error;
use crate::api_lib::utils::runtime::{execute, execute_async};

#[no_mangle]
pub extern "C" fn vcx_revocation_registry_create(
    command_handle: CommandHandle,
    rev_reg_config: *const c_char,
    cb: Option<extern "C" fn(xcommand_handle: CommandHandle, err: u32, rev_reg_handle: u32)>,
) -> u32 {
    info!("vcx_revocation_registry_create >>>");

    check_useful_c_callback!(cb, LibvcxErrorKind::InvalidOption);
    check_useful_c_str!(rev_reg_config, LibvcxErrorKind::InvalidOption);

    trace!("vcx_revocation_registry_create(command_handle: {})", command_handle);

    let config = match serde_json::from_str::<RevocationRegistryConfig>(&rev_reg_config) {
        Ok(config) => config,
        Err(err) => {
            set_current_error(&err);
            error!(
                "vcx_revocation_registry_create >>> invalid revocation registry configuration; err: {:?}",
                err
            );
            return LibvcxErrorKind::InvalidConfiguration.into();
        }
    };

    execute_async::<BoxFuture<'static, Result<(), ()>>>(Box::pin(async move {
        let (rc, handle) = match revocation_registry::create(config).await {
            Ok(handle) => {
                trace!(
                    "vcx_revocation_registry_create_cb(command_handle: {}, rc: {}, handle: {})",
                    command_handle,
                    libvcx_error::SUCCESS_ERR_CODE,
                    handle
                );
                (error_libvcx::SUCCESS_ERR_CODE, handle)
            }
            Err(err) => {
                set_current_error_vcx(&err);
                error!(
                    "vcx_revocation_registry_create_cb(command_handle: {}, rc: {}, handle: {})",
                    command_handle, err, 0
                );
                (err.into(), 0)
            }
        };

        cb(command_handle, rc, handle);

        Ok(())
    }));

    error_libvcx::SUCCESS_ERR_CODE
}

#[no_mangle]
pub extern "C" fn vcx_revocation_registry_publish(
    command_handle: CommandHandle,
    rev_reg_handle: u32,
    tails_url: *const c_char,
    cb: Option<extern "C" fn(xcommand_handle: CommandHandle, err: u32, handle: u32)>,
) -> u32 {
    info!("vcx_revocation_registry_publish >>>");

    check_useful_c_callback!(cb, LibvcxErrorKind::InvalidOption);
    check_useful_c_str!(tails_url, LibvcxErrorKind::InvalidOption);

    trace!(
        "vcx_revocation_registry_publish(command_handle: {}, rev_reg_handle: {}, tails_url: {})",
        command_handle,
        rev_reg_handle,
        tails_url
    );

    execute_async::<BoxFuture<'static, Result<(), ()>>>(Box::pin(async move {
        match revocation_registry::publish(rev_reg_handle, &tails_url).await {
            Ok(handle) => {
                trace!(
                    "vcx_revocation_registry_publish_cb(command_handle: {}, rc: {}) handle: {}",
                    command_handle,
                    libvcx_error::SUCCESS_ERR_CODE,
                    handle
                );
                cb(command_handle, error_libvcx::SUCCESS_ERR_CODE, handle);
            }
            Err(err) => {
                set_current_error_vcx(&err);
                error!(
                    "vcx_revocation_registry_publish_cb(command_handle: {}, rc: {}) handle: {}",
                    command_handle, err, 0
                );
                cb(command_handle, err.into(), 0);
            }
        };
        Ok(())
    }));

    error_libvcx::SUCCESS_ERR_CODE
}

#[no_mangle]
pub extern "C" fn vcx_revocation_registry_publish_revocations(
    command_handle: CommandHandle,
    rev_reg_handle: u32,
    cb: Option<extern "C" fn(xcommand_handle: CommandHandle, err: u32)>,
) -> u32 {
    info!("vcx_revocation_registry_publish_revocations >>>");

    check_useful_c_callback!(cb, LibvcxErrorKind::InvalidOption);

    let issuer_did: String = match vcx_settings::get_config_value(settings::CONFIG_INSTITUTION_DID) {
        Ok(value) => value,
        Err(err) => return err.into(),
    };
    trace!(
        "vcx_revocation_registry_publish_revocations(command_handle: {}, rev_reg_handle: {}, issuer_did: {})",
        command_handle,
        rev_reg_handle,
        issuer_did
    );

    execute_async::<BoxFuture<'static, Result<(), ()>>>(Box::pin(async move {
        match revocation_registry::publish_revocations(rev_reg_handle, &issuer_did).await {
            Ok(()) => {
                trace!(
                    "vcx_revocation_registry_publish_revocations_cb(command_handle: {}, issuer_did: {}, rc: {})",
                    command_handle,
                    issuer_did,
                    libvcx_error::SUCCESS_ERR_CODE
                );
                cb(command_handle, error_libvcx::SUCCESS_ERR_CODE);
            }
            Err(err) => {
                set_current_error_vcx(&err);
                error!(
                    "vcx_revocation_registry_publish_revocations_cb(command_handle: {}, issuer_did: {}, rc: {})",
                    command_handle, issuer_did, err
                );
                cb(command_handle, err.into());
            }
        };
        Ok(())
    }));

    error_libvcx::SUCCESS_ERR_CODE
}

#[no_mangle]
pub extern "C" fn vcx_revocation_registry_get_rev_reg_id(
    command_handle: CommandHandle,
    handle: u32,
    cb: Option<extern "C" fn(xcommand_handle: CommandHandle, err: u32, rev_reg_id: *const c_char)>,
) -> u32 {
    info!("vcx_revocation_registry_get_rev_reg_id >>>");

    check_useful_c_callback!(cb, LibvcxErrorKind::InvalidOption);

    trace!(
        "vcx_revocation_registry_get_rev_reg_id(command_handle: {}, handle: {})",
        command_handle,
        handle
    );

    execute(move || {
        match revocation_registry::get_rev_reg_id(handle) {
            Ok(rev_reg_id) => {
                trace!(
                    "vcx_revocation_registry_get_rev_reg_id_cb(command_handle: {}, rc: {}, rev_reg_id: {})",
                    command_handle,
                    libvcx_error::SUCCESS_ERR_CODE,
                    rev_reg_id
                );
                let rev_reg_json = CStringUtils::string_to_cstring(rev_reg_id);
                cb(command_handle, error_libvcx::SUCCESS_ERR_CODE, rev_reg_json.as_ptr());
            }
            Err(err) => {
                set_current_error_vcx(&err);
                error!(
                    "vcx_revocation_registry_get_rev_reg_id_cb(command_handle: {}, rc: {}, rev_reg_id: {})",
                    command_handle, err, 0
                );
                cb(command_handle, err.into(), ptr::null());
            }
        }
        Ok(())
    });

    error_libvcx::SUCCESS_ERR_CODE
}

#[no_mangle]
pub extern "C" fn vcx_revocation_registry_get_tails_hash(
    command_handle: CommandHandle,
    handle: u32,
    cb: Option<extern "C" fn(xcommand_handle: CommandHandle, err: u32, tails_hash: *const c_char)>,
) -> u32 {
    info!("vcx_revocation_registry_get_tails_hash >>>");

    check_useful_c_callback!(cb, LibvcxErrorKind::InvalidOption);

    trace!(
        "vcx_revocation_registry_get_tails_hash(command_handle: {}, handle: {})",
        command_handle,
        handle
    );

    execute(move || {
        match revocation_registry::get_tails_hash(handle) {
            Ok(tails_hash) => {
                trace!(
                    "vcx_revocation_registry_get_tails_hash_cb(command_handle: {}, rc: {}, tails_hash: {})",
                    command_handle,
                    libvcx_error::SUCCESS_ERR_CODE,
                    tails_hash
                );
                let tails_hash = CStringUtils::string_to_cstring(tails_hash);
                cb(command_handle, error_libvcx::SUCCESS_ERR_CODE, tails_hash.as_ptr());
            }
            Err(err) => {
                set_current_error_vcx(&err);
                error!(
                    "vcx_revocation_registry_get_tails_hash_cb(command_handle: {}, rc: {}, tails_hash: {})",
                    command_handle, err, 0
                );
                cb(command_handle, err.into(), ptr::null());
            }
        }
        Ok(())
    });

    error_libvcx::SUCCESS_ERR_CODE
}

#[no_mangle]
pub extern "C" fn vcx_revocation_registry_serialize(
    command_handle: CommandHandle,
    handle: u32,
    cb: Option<extern "C" fn(xcommand_handle: CommandHandle, err: u32, rev_reg_json: *const c_char)>,
) -> u32 {
    info!("vcx_revocation_registry_serialize >>>");

    check_useful_c_callback!(cb, LibvcxErrorKind::InvalidOption);

    trace!(
        "vcx_revocation_registry_serialize(command_handle: {}, handle: {})",
        command_handle,
        handle
    );

    execute(move || {
        match revocation_registry::to_string(handle) {
            Ok(rev_reg_json) => {
                trace!(
                    "vcx_revocation_registry_serialize_cb(command_handle: {}, rc: {}, rev_reg_json: {})",
                    command_handle,
                    libvcx_error::SUCCESS_ERR_CODE,
                    rev_reg_json
                );
                let rev_reg_json = CStringUtils::string_to_cstring(rev_reg_json);
                cb(command_handle, error_libvcx::SUCCESS_ERR_CODE, rev_reg_json.as_ptr());
            }
            Err(err) => {
                set_current_error_vcx(&err);
                error!(
                    "vcx_revocation_registry_serialize_cb(command_handle: {}, rc: {}, rev_reg_json: {})",
                    command_handle, err, 0
                );
                cb(command_handle, err.into(), ptr::null());
            }
        }
        Ok(())
    });

    error_libvcx::SUCCESS_ERR_CODE
}

#[no_mangle]
pub extern "C" fn vcx_revocation_registry_deserialize(
    command_handle: CommandHandle,
    rev_reg_json: *const c_char,
    cb: Option<extern "C" fn(xcommand_handle: CommandHandle, err: u32, handle: u32)>,
) -> u32 {
    info!("vcx_revocation_registry_deserialize >>>");

    check_useful_c_callback!(cb, LibvcxErrorKind::InvalidOption);
    check_useful_c_str!(rev_reg_json, LibvcxErrorKind::InvalidOption);

    trace!(
        "vcx_revocation_registry_deserialize(command_handle: {}, rev_reg_json: {})",
        command_handle,
        rev_reg_json
    );

    execute(move || {
        match revocation_registry::from_string(&rev_reg_json) {
            Ok(handle) => {
                trace!(
                    "vcx_revocation_registry_deserialize_cb(command_handle: {}, rc: {}, handle: {})",
                    command_handle,
                    libvcx_error::SUCCESS_ERR_CODE,
                    handle
                );
                cb(command_handle, error_libvcx::SUCCESS_ERR_CODE, handle);
            }
            Err(err) => {
                set_current_error_vcx(&err);
                error!(
                    "vcx_revocation_registry_deserialize_cb(command_handle: {}, rc: {}, handle: {})",
                    command_handle, err, 0
                );
                cb(command_handle, err.into(), 0);
            }
        }
        Ok(())
    });

    error_libvcx::SUCCESS_ERR_CODE
}

#[no_mangle]
pub extern "C" fn vcx_revocation_registry_release(handle: u32) -> u32 {
    info!("vcx_revocation_registry_release >>>");

    match revocation_registry::release(handle) {
        Ok(()) => {
            trace!("vcx_revocation_registry_release_cb(rc: {})", libvcx_error::SUCCESS_ERR_CODE);
            error_libvcx::SUCCESS_ERR_CODE
        }
        Err(err) => {
            set_current_error_vcx(&err);
            error!("vcx_revocation_registry_release_cb(rc: {})", err);
            err.into()
        }
    }
}
