use std::ffi::CString;

use futures::future::{BoxFuture, FutureExt};
use libc::c_char;

use aries_vcx::agency_client::configuration::AgencyClientConfig;
use libvcx_core::api_vcx::api_global::agency_client::create_agency_client_for_main_wallet;
use libvcx_core::api_vcx::api_global::agency_client::update_webhook_url;
use libvcx_core::api_vcx::api_global::ledger::ledger_get_txn_author_agreement;
use libvcx_core::api_vcx::api_global::pool::{open_main_pool, LibvcxLedgerConfig};
use libvcx_core::api_vcx::api_global::settings::enable_mocks;
use libvcx_core::api_vcx::api_global::state::state_vcx_shutdown;
use libvcx_core::api_vcx::api_global::VERSION_STRING;
use libvcx_core::errors::error::{LibvcxError, LibvcxErrorKind};

use crate::api_c::cutils::cstring::CStringUtils;
use crate::api_c::cutils::current_error::{get_current_error_c_json, set_current_error, set_current_error_vcx};
use crate::api_c::cutils::runtime::{execute, execute_async, init_threadpool};
use crate::api_c::types::CommandHandle;
use crate::error::SUCCESS_ERR_CODE;

/// Only for Wrapper testing purposes, sets global library settings.
///
/// #Params
///
/// config: The agent provision configuration
///
/// #Returns
/// Error code as a u32
#[no_mangle]
pub extern "C" fn vcx_enable_mocks() -> u32 {
    info!("vcx_enable_mocks >>>");
    match enable_mocks() {
        Ok(_) => {}
        Err(_) => return LibvcxErrorKind::UnknownError.into(),
    };
    SUCCESS_ERR_CODE
}

/// Initializes threadpool.
///
/// #Params
/// command_handle: command handle to map callback to user context.
///
/// threadpool_config: Config of the threadpool
/// {
///    num_threads (optional) - number of threads in the threadpool (default: 8)
/// }
///
/// cb: Callback that provides error status
///
/// #Returns
/// Error code as a u32
#[no_mangle]
pub extern "C" fn vcx_init_threadpool(config: *const c_char) -> u32 {
    info!("vcx_init_threadpool >>>");

    check_useful_c_str!(config, LibvcxErrorKind::InvalidOption);

    match init_threadpool(&config) {
        Ok(_) => SUCCESS_ERR_CODE,
        Err(_) => LibvcxErrorKind::UnknownError.into(),
    }
}

/// Creates an instance of agency client used to communicate with the agency. Must be called after
/// wallet was created and opened.
///
/// #Params
/// command_handle: command handle to map callback to user context.
///
/// agency_config: Config of the agency client
/// {
///    agency_url - URL of the agency
///    agency_did - DID of the agency
///    agency_pwdid - pairwise DID of the agency created for the connection
///    agency_vk - verkey  of the agency created for the connection
///    agent_pwdid - pairwise DID of this client's agent in the agency
///    agent_vk - verkey of this client's agent in the agency
///    my_pwdid - pairwise DID of this client used to communicate with it's agent in the agency
///    my_vk - verkey of this client used to communicate with it's agent in the agency
/// }
///
/// cb: Callback that provides error status
///
/// #Returns
/// Error code as a u32
#[no_mangle]
pub extern "C" fn vcx_create_agency_client_for_main_wallet(
    command_handle: CommandHandle,
    config: *const c_char,
    cb: Option<extern "C" fn(xcommand_handle: CommandHandle, err: u32)>,
) -> u32 {
    info!("vcx_create_agency_client_for_main_wallet >>>");

    check_useful_c_str!(config, LibvcxErrorKind::InvalidOption);
    check_useful_c_callback!(cb, LibvcxErrorKind::InvalidOption);

    trace!("vcx_create_agency_client_for_main_wallet >>> config: {}", config);

    let agency_config = match serde_json::from_str::<AgencyClientConfig>(&config) {
        Ok(agency_config) => agency_config,
        Err(err) => {
            set_current_error(&err);
            error!(
                "vcx_create_agency_client_for_main_wallet >>> invalid configuration, err: {:?}",
                err
            );
            return LibvcxErrorKind::InvalidConfiguration.into();
        }
    };

    execute(move || {
        match create_agency_client_for_main_wallet(&agency_config) {
            Ok(()) => {
                info!(
                    "vcx_create_agency_client_for_main_wallet_cb >>> command_handle: {}, rc {}",
                    command_handle, SUCCESS_ERR_CODE
                );
                cb(command_handle, SUCCESS_ERR_CODE)
            }
            Err(err) => {
                set_current_error_vcx(&err);
                error!(
                    "vcx_create_agency_client_for_main_wallet_cb >>> command_handle: {}, error {}",
                    command_handle, err
                );
                cb(command_handle, err.into());
                return Ok(());
            }
        }
        Ok(())
    });
    SUCCESS_ERR_CODE
}

/// Opens pool based on vcx configuration passed as a parameter
///
/// #Params
/// command_handle: command handle to map callback to user context.
///
/// pool_config: Pool configuration
/// {
///     "genesis_path" - Path to the genesis file
///     "pool_name" (optional) - Name of the pool ledger configuration.
///     `pool_config` (optional) - Runtime pool configuration json as a string.
///                         if NULL, then default config will be used.
/// }
/// where pool config structure is as follows
/// {
///     "timeout": int (optional), timeout for network request (in sec).
///     "extended_timeout": int (optional), extended timeout for network request (in sec).
///     "preordered_nodes": array<string> -  (optional), names of nodes which will have a priority during request sending:
///         ["name_of_1st_prior_node",  "name_of_2nd_prior_node", .... ]
///         This can be useful if a user prefers querying specific nodes.
///         Assume that `Node1` and `Node2` nodes reply faster.
///         If you pass them Libindy always sends a read request to these nodes first and only then (if not enough) to others.
///         Note: Nodes not specified will be placed randomly.
///     "number_read_nodes": int (optional) - the number of nodes to send read requests (2 by default)
///         By default Libindy sends a read requests to 2 nodes in the pool.
///         If response isn't received or `state proof` is invalid Libindy sends the request again but to 2 (`number_read_nodes`) * 2 = 4 nodes and so far until completion.
/// }
///
/// cb: Callback that provides error status
///
/// #Returns
/// Error code as a u32
#[no_mangle]
pub extern "C" fn vcx_open_main_pool(
    command_handle: CommandHandle,
    pool_config: *const c_char,
    cb: extern "C" fn(xcommand_handle: CommandHandle, err: u32),
) -> u32 {
    info!("vcx_open_main_pool >>>");
    check_useful_c_str!(pool_config, LibvcxErrorKind::InvalidOption);

    let pool_config = match serde_json::from_str::<LibvcxLedgerConfig>(&pool_config) {
        Ok(pool_config) => pool_config,
        Err(err) => {
            set_current_error(&err);
            error!("vcx_open_main_pool >>> invalid wallet configuration; err: {:?}", err);
            return LibvcxErrorKind::InvalidConfiguration.into();
        }
    };

    execute_async::<BoxFuture<'static, Result<(), ()>>>(Box::pin(async move {
        match open_main_pool(&pool_config).await {
            Ok(()) => {
                info!("vcx_open_main_pool_cb :: Vcx Pool Init Successful");
                cb(command_handle, SUCCESS_ERR_CODE)
            }
            Err(err) => {
                set_current_error_vcx(&err);
                error!("vcx_open_main_pool_cb :: Vcx Pool Init Error {}.", err);
                cb(command_handle, err.into());
                return Ok(());
            }
        }
        Ok(())
    }));
    SUCCESS_ERR_CODE
}

lazy_static! {
    pub static ref VERSION_STRING_CSRING: CString =
        CString::new(VERSION_STRING.to_string()).expect("Unexpected error converting to CString");
}

#[no_mangle]
pub extern "C" fn vcx_version() -> *const c_char {
    VERSION_STRING_CSRING.as_ptr()
}

/// Reset libvcx to a pre-configured state, releasing/deleting any handles and freeing memory
///
/// libvcx will be inoperable and must be initialized again with vcx_init_with_config
///
/// #Params
/// delete: specify whether wallet/pool should be deleted
///
/// #Returns
/// Success
#[no_mangle]
pub extern "C" fn vcx_shutdown(_delete: bool) -> u32 {
    info!("vcx_shutdown >>>");
    state_vcx_shutdown();
    SUCCESS_ERR_CODE
}

/// Get the message corresponding to an error code
///
/// #Params
/// error_code: code of error
///
/// #Returns
/// Error message
#[no_mangle]
pub extern "C" fn vcx_error_c_message(error_code: u32) -> *const c_char {
    let kind_string = LibvcxErrorKind::from(error_code).to_string();
    CString::new(kind_string)
        .expect("Unexpected error converting to CString")
        .into_raw()
}

/// Update agency webhook url setting
///
/// #Params
///
/// command_handle: command handle to map callback to user context.
///
/// notification_webhook_url: URL to which the notifications should be sent
///
/// cb: Callback that provides error code of the result
///
/// #Returns
/// Error code as u32
#[no_mangle]
pub extern "C" fn vcx_update_webhook_url(
    command_handle: CommandHandle,
    notification_webhook_url: *const c_char,
    cb: Option<extern "C" fn(xcommand_handle: CommandHandle, err: u32)>,
) -> u32 {
    info!("vcx_update_webhook {:?} >>>", notification_webhook_url);

    check_useful_c_str!(notification_webhook_url, LibvcxErrorKind::InvalidOption);
    check_useful_c_callback!(cb, LibvcxErrorKind::InvalidOption);

    trace!("vcx_update_webhook(webhook_url: {})", notification_webhook_url);

    execute_async::<BoxFuture<'static, Result<(), ()>>>(
        async move {
            match update_webhook_url(&notification_webhook_url[..]).await {
                Ok(()) => {
                    trace!(
                        "vcx_update_webhook_url_cb(command_handle: {}, rc: {})",
                        command_handle,
                        SUCCESS_ERR_CODE
                    );

                    cb(command_handle, SUCCESS_ERR_CODE);
                }
                Err(err) => {
                    set_current_error_vcx(&err);
                    error!(
                        "vcx_update_webhook_url_cb(command_handle: {}, rc: {})",
                        command_handle, err
                    );

                    cb(command_handle, err.into());
                }
            };

            Ok(())
        }
        .boxed(),
    );

    SUCCESS_ERR_CODE
}

#[no_mangle]
pub extern "C" fn vcx_get_ledger_author_agreement(
    command_handle: CommandHandle,
    cb: Option<extern "C" fn(xcommand_handle: CommandHandle, err: u32, author_agreement: *const c_char)>,
) -> u32 {
    info!("vcx_get_ledger_author_agreement >>>");

    check_useful_c_callback!(cb, LibvcxErrorKind::InvalidOption);

    trace!("vcx_get_ledger_author_agreement(command_handle: {})", command_handle);

    execute_async::<BoxFuture<'static, Result<(), ()>>>(
        async move {
            match ledger_get_txn_author_agreement().await {
                Ok(err) => {
                    trace!(
                        "vcx_get_ledger_author_agreement(command_handle: {}, rc: {}, author_agreement: {})",
                        command_handle,
                        SUCCESS_ERR_CODE,
                        err
                    );

                    let msg = CStringUtils::string_to_cstring(err);
                    cb(command_handle, SUCCESS_ERR_CODE, msg.as_ptr());
                }
                Err(err) => {
                    set_current_error_vcx(&err);
                    error!(
                        "vcx_get_ledger_author_agreement(command_handle: {}, rc: {})",
                        command_handle, err
                    );
                    cb(command_handle, err.into(), std::ptr::null_mut());
                }
            };

            Ok(())
        }
        .boxed(),
    );

    SUCCESS_ERR_CODE
}

// todo: remove from FFI and wrappers
#[no_mangle]
pub extern "C" fn vcx_set_active_txn_author_agreement_meta(
    _text: *const c_char,
    _version: *const c_char,
    _hash: *const c_char,
    _acc_mech_type: *const c_char,
    _time_of_acceptance: u64,
) -> u32 {
    unimplemented!("Not expected to be called in mobile use-case")
}

/// Get details for last occurred error.
///
/// This function should be called in two places to handle both cases of error occurrence:
///     1) synchronous  - in the same application thread
///     2) asynchronous - inside of function callback
///
/// NOTE: Error is stored until the next one occurs in the same execution thread or until asynchronous callback finished.
///       Returning pointer has the same lifetime.
///
/// #Params
/// * `error_json_p` - Reference that will contain error details (if any error has occurred before)
///  in the format:
/// {
///     "backtrace": Optional<str> - error backtrace.
///         Collecting of backtrace can be enabled by setting environment variable `RUST_BACKTRACE=1`
///     "message": str - human-readable error description
/// }
///
#[no_mangle]
pub extern "C" fn vcx_get_current_error(error_json_p: *mut *const c_char) {
    trace!("vcx_get_current_error >>> error_json_p: {:?}", error_json_p);

    let error = get_current_error_c_json();
    unsafe { *error_json_p = error };

    trace!("vcx_get_current_error: <<<");
}

#[cfg(test)]
pub mod test_utils {
    use uuid;

    use aries_vcx::agency_client::testing::mocking::enable_agency_mocks;
    use aries_vcx::global::settings::{DEFAULT_WALLET_KEY, WALLET_KDF_RAW};
    use libvcx_core::errors;

    use crate::api_c::cutils::return_types_u32;
    use crate::api_c::cutils::timeout::TimeoutUtils;
    use crate::api_c::vcx::vcx_open_main_pool;
    use crate::api_c::wallet::{
        vcx_configure_issuer_wallet, vcx_create_wallet, vcx_open_main_wallet, vcx_wallet_add_record,
        vcx_wallet_get_record,
    };

    use super::*;

    pub fn _vcx_open_main_pool_c_closure(pool_config: &str) -> Result<(), u32> {
        let cb = return_types_u32::Return_U32::new().unwrap();

        let rc = vcx_open_main_pool(
            cb.command_handle,
            CString::new(pool_config).unwrap().into_raw(),
            cb.get_callback(),
        );
        if rc != SUCCESS_ERR_CODE {
            return Err(rc);
        }
        cb.receive(TimeoutUtils::some_medium())
    }

    pub fn _vcx_open_main_wallet_c_closure(wallet_config: &str) -> Result<i32, u32> {
        let cb = return_types_u32::Return_U32_I32::new().unwrap();

        let rc = vcx_open_main_wallet(
            cb.command_handle,
            CString::new(wallet_config).unwrap().into_raw(),
            Some(cb.get_callback()),
        );
        if rc != SUCCESS_ERR_CODE {
            return Err(rc);
        }
        cb.receive(TimeoutUtils::some_medium())
    }

    pub fn _vcx_init_threadpool_c_closure(config: &str) -> Result<(), u32> {
        let rc = vcx_init_threadpool(CString::new(config).unwrap().into_raw());
        if rc != SUCCESS_ERR_CODE {
            return Err(rc);
        }
        Ok(())
    }

    pub fn _vcx_init_threadpool(config_threadpool: &str) -> Result<(), u32> {
        info!("_vcx_init_threadpool >>>");
        let rc = vcx_init_threadpool(CString::new(config_threadpool).unwrap().into_raw());
        if rc != SUCCESS_ERR_CODE {
            error!("vcx_init_threadpool failed");
            return Err(rc);
        }
        Ok(())
    }

    pub fn _vcx_open_pool(config_pool: &str) -> Result<(), u32> {
        info!("_vcx_open_pool >>> going to open pool");
        let cb = return_types_u32::Return_U32::new().unwrap();
        let rc = vcx_open_main_pool(
            cb.command_handle,
            CString::new(config_pool).unwrap().into_raw(),
            cb.get_callback(),
        );
        if rc != SUCCESS_ERR_CODE {
            error!("vcx_open_pool failed");
            return Err(rc);
        }
        cb.receive(TimeoutUtils::some_short()).unwrap();
        Ok(())
    }

    pub fn _vcx_init_full(config_threadpool: &str, config_pool: &str, config_wallet: &str) -> Result<(), u32> {
        info!("_vcx_init_full >>>");
        let rc = vcx_init_threadpool(CString::new(config_threadpool).unwrap().into_raw());
        if rc != SUCCESS_ERR_CODE {
            error!("vcx_init_threadpool failed");
            return Err(rc);
        }
        // todo: possibly can be removed
        enable_agency_mocks();

        info!("_vcx_init_full >>> going to open pool");
        let cb = return_types_u32::Return_U32::new().unwrap();
        let rc = vcx_open_main_pool(
            cb.command_handle,
            CString::new(config_pool).unwrap().into_raw(),
            cb.get_callback(),
        );
        if rc != SUCCESS_ERR_CODE {
            error!("vcx_open_pool failed");
            return Err(rc);
        }
        cb.receive(TimeoutUtils::some_short()).unwrap();

        info!("_vcx_init_full >>> going to open wallet");
        let cb = return_types_u32::Return_U32_I32::new().unwrap();
        let rc = vcx_open_main_wallet(
            cb.command_handle,
            CString::new(config_wallet).unwrap().into_raw(),
            Some(cb.get_callback()),
        );
        if rc != SUCCESS_ERR_CODE {
            error!("vcx_open_wallet failed");
            return Err(rc);
        }
        cb.receive(TimeoutUtils::some_custom(3)).unwrap();
        Ok(())
    }
}

#[cfg(test)]
#[allow(unused_imports)] // TODO: remove it
mod tests {

    use std::ptr;

    use aries_vcx::aries_vcx_core::indy;
    use aries_vcx::aries_vcx_core::wallet::indy::wallet::import;
    use aries_vcx::aries_vcx_core::wallet::indy::{RestoreWalletConfigs, WalletConfig};
    use aries_vcx::global::settings::{
        set_config_value, CONFIG_GENESIS_PATH, CONFIG_TXN_AUTHOR_AGREEMENT, DEFAULT_WALLET_BACKUP_KEY,
        DEFAULT_WALLET_KEY, WALLET_KDF_RAW,
    };
    use aries_vcx::utils::constants::POOL1_TXN;
    use aries_vcx::utils::devsetup::{SetupDefaults, SetupEmpty, SetupMocks, TempFile};
    use aries_vcx::utils::mockdata::mockdata_credex::ARIES_CREDENTIAL_OFFER;
    use aries_vcx::utils::mockdata::mockdata_proof::ARIES_PROOF_REQUEST_PRESENTATION;
    use libvcx_core;
    use libvcx_core::api_vcx::api_global::settings;
    use libvcx_core::api_vcx::api_global::wallet::close_main_wallet;
    #[cfg(test)]
    use libvcx_core::api_vcx::api_global::wallet::test_utils::_create_main_wallet_and_its_backup;
    use libvcx_core::api_vcx::api_global::wallet::wallet_import;
    use libvcx_core::api_vcx::api_handle::{
        credential, credential_def, disclosed_proof, issuer_credential, mediated_connection, proof, schema,
    };
    use libvcx_core::errors;
    use libvcx_core::errors::error::{LibvcxErrorKind, LibvcxResult};

    use crate::api_c;
    use crate::api_c::cutils::current_error::reset_current_error;
    use crate::api_c::cutils::return_types_u32;
    use crate::api_c::cutils::timeout::TimeoutUtils;
    use crate::api_c::protocols::mediated_connection::vcx_connection_create;
    #[cfg(test)]
    use crate::api_c::vcx::test_utils::{
        _vcx_init_threadpool, _vcx_init_threadpool_c_closure, _vcx_open_main_pool_c_closure,
        _vcx_open_main_wallet_c_closure, _vcx_open_pool,
    };
    use crate::api_c::wallet::vcx_create_wallet;

    use super::*;

    #[test]
    fn test_error_c_message() {
        let _setup = SetupMocks::init();

        let c_message = CStringUtils::c_str_to_string(vcx_error_c_message(1001))
            .unwrap()
            .unwrap();
        assert_match!(c_message, "Unknown error");

        let c_message = CStringUtils::c_str_to_string(vcx_error_c_message(100100))
            .unwrap()
            .unwrap();
        assert_match!(c_message, "Unknown error");
        //
        let c_message = CStringUtils::c_str_to_string(vcx_error_c_message(1021))
            .unwrap()
            .unwrap();
        assert_eq!(
            c_message,
            "Attributes provided to Credential Offer are not correct, possibly malformed"
        );
    }

    #[test]
    fn test_vcx_init_threadpool_fails_with_null_ptr_config() {
        let _setup = SetupEmpty::init();
        assert_eq!(
            vcx_init_threadpool(ptr::null()),
            u32::from(LibvcxErrorKind::InvalidOption)
        )
    }

    #[test]
    fn test_shutdown_with_no_previous_config() {
        let _setup = SetupDefaults::init();

        vcx_shutdown(true);
        vcx_shutdown(false);
    }

    #[test]
    fn test_vcx_version() {
        let _setup = SetupDefaults::init();

        let return_version = CStringUtils::c_str_to_string(vcx_version()).unwrap().unwrap();
        assert!(return_version.len() > 5);
    }

    #[test]
    fn get_current_error_works_for_no_error() {
        let _setup = SetupDefaults::init();

        reset_current_error();

        let mut error_json_p: *const c_char = ptr::null();

        vcx_get_current_error(&mut error_json_p);
        assert_eq!(None, CStringUtils::c_str_to_string(error_json_p).unwrap());
    }

    #[test]
    fn get_current_error_works_for_sync_error() {
        let _setup = SetupDefaults::init();

        let cb = return_types_u32::Return_U32_STR::new().unwrap();
        api_c::utils::vcx_provision_cloud_agent(cb.command_handle, ptr::null(), Some(cb.get_callback()));

        let mut error_json_p: *const c_char = ptr::null();
        vcx_get_current_error(&mut error_json_p);
        assert!(CStringUtils::c_str_to_string(error_json_p).unwrap().is_some());
    }

    #[test]
    fn get_current_error_works_for_async_error() {
        let _setup = SetupDefaults::init();

        extern "C" fn cb(_storage_handle: i32, _err: u32, _config: *const c_char) {
            let mut error_json_p: *const c_char = ptr::null();
            vcx_get_current_error(&mut error_json_p);
            assert!(CStringUtils::c_str_to_string(error_json_p).unwrap().is_some());
        }

        let config = CString::new("{}").unwrap();
        api_c::utils::vcx_provision_cloud_agent(0, config.as_ptr(), Some(cb));
        std::thread::sleep(std::time::Duration::from_secs(1));
    }

    #[test]
    fn test_call_c_callable_api_without_threadpool() {
        let _setup = SetupMocks::init();

        let cb = return_types_u32::Return_U32_U32::new().unwrap();
        let _rc = vcx_connection_create(
            cb.command_handle,
            CString::new("test_create").unwrap().into_raw(),
            Some(cb.get_callback()),
        );

        assert!(cb.receive(TimeoutUtils::some_medium()).unwrap() > 0);
    }
}
