use std::ffi::CString;

use futures::future::{BoxFuture, FutureExt};
use libc::c_char;

use aries_vcx::agency_client::configuration::AgencyClientConfig;

use aries_vcx::indy::ledger::pool::PoolConfig;
use aries_vcx::indy::wallet::IssuerConfig;

use crate::api_c::types::CommandHandle;
use crate::api_vcx::api_global::agency_client::update_webhook_url;
use crate::api_vcx::api_global::ledger::{ledger_get_txn_author_agreement, ledger_set_txn_author_agreement};

use crate::api_vcx::api_global::agency_client::create_agency_client_for_main_wallet;
use crate::api_vcx::api_global::pool::open_main_pool;
use crate::api_vcx::api_global::settings::{enable_mocks, settings_init_issuer_config};
use crate::errors::error;
use crate::errors::error::{LibvcxError, LibvcxErrorKind};

use crate::api_vcx::api_global::state::state_vcx_shutdown;

use crate::api_c::cutils::cstring::CStringUtils;
use crate::api_c::cutils::current_error::{get_current_error_c_json, set_current_error, set_current_error_vcx};
use crate::api_c::cutils::runtime::{execute, execute_async, init_threadpool};
use crate::api_vcx::api_global::VERSION_STRING;

use crate::api_vcx::utils::version_constants;

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
    error::SUCCESS_ERR_CODE
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
        Ok(_) => error::SUCCESS_ERR_CODE,
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
                    command_handle,
                    error::SUCCESS_ERR_CODE
                );
                cb(command_handle, error::SUCCESS_ERR_CODE)
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
    error::SUCCESS_ERR_CODE
}

/// Stores institution did and verkey in memory.
///
/// #Params
/// command_handle: command handle to map callback to user context.
///
/// issuer_config: Issuer configuration
/// {
///     "institution_did" (optional) - Institution did obtained on vcx_configure_issuer_wallet
///     `institution_verkey` (optional) - Institution verkey obtained on vcx_configure_issuer_wallet
///                         If NULL, then value set on vcx_configure_issuer_wallet will be used.
/// }
///
/// cb: Callback that provides error status
///
/// #Returns
/// Error code as a u32
#[no_mangle]
pub extern "C" fn vcx_init_issuer_config(
    command_handle: CommandHandle,
    config: *const c_char,
    cb: Option<extern "C" fn(xcommand_handle: CommandHandle, err: u32)>,
) -> u32 {
    info!("vcx_init_issuer_config >>>");

    check_useful_c_str!(config, LibvcxErrorKind::InvalidOption);
    check_useful_c_callback!(cb, LibvcxErrorKind::InvalidOption);

    trace!("vcx_init_issuer_config >>> config: {}", config);

    let issuer_config = match serde_json::from_str::<IssuerConfig>(&config) {
        Ok(issuer_config) => issuer_config,
        Err(err) => {
            set_current_error(&err);
            error!("vcx_init_issuer_config >>> invalid configuration, err: {:?}", err);
            return LibvcxErrorKind::InvalidConfiguration.into();
        }
    };

    execute(move || {
        match settings_init_issuer_config(&issuer_config) {
            Ok(()) => {
                info!(
                    "vcx_init_issuer_config_cb >>> command_handle: {}, rc: {}",
                    command_handle,
                    error::SUCCESS_ERR_CODE
                );
                cb(command_handle, error::SUCCESS_ERR_CODE)
            }
            Err(err) => {
                set_current_error_vcx(&err);
                error!(
                    "vcx_init_issuer_config_cb >>> command_handle: {}, error {}",
                    command_handle, err
                );
                cb(command_handle, err.into());
                return Ok(());
            }
        }
        Ok(())
    });
    error::SUCCESS_ERR_CODE
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

    let pool_config = match serde_json::from_str::<PoolConfig>(&pool_config) {
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
                cb(command_handle, error::SUCCESS_ERR_CODE)
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
    error::SUCCESS_ERR_CODE
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
pub extern "C" fn vcx_shutdown(delete: bool) -> u32 {
    info!("vcx_shutdown >>>");
    trace!("vcx_shutdown(delete: {})", delete);
    state_vcx_shutdown(delete);
    error::SUCCESS_ERR_CODE
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
                        error::SUCCESS_ERR_CODE
                    );

                    cb(command_handle, error::SUCCESS_ERR_CODE);
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

    error::SUCCESS_ERR_CODE
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
                        error::SUCCESS_ERR_CODE,
                        err
                    );

                    let msg = CStringUtils::string_to_cstring(err);
                    cb(command_handle, error::SUCCESS_ERR_CODE, msg.as_ptr());
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

    error::SUCCESS_ERR_CODE
}

/// Set some accepted agreement as active.
///
/// As result of successful call of this function appropriate metadata will be appended to each write request.
///
/// #Params
/// text and version - (optional) raw data about TAA from ledger.
///     These parameters should be passed together.
///     These parameters are required if hash parameter is ommited.
/// hash - (optional) hash on text and version. This parameter is required if text and version parameters are ommited.
/// acc_mech_type - mechanism how user has accepted the TAA
/// time_of_acceptance - UTC timestamp when user has accepted the TAA
///
/// #Returns
/// Error code as a u32
#[no_mangle]
pub extern "C" fn vcx_set_active_txn_author_agreement_meta(
    text: *const c_char,
    version: *const c_char,
    hash: *const c_char,
    acc_mech_type: *const c_char,
    time_of_acceptance: u64,
) -> u32 {
    info!("vcx_set_active_txn_author_agreement_meta >>>");

    check_useful_opt_c_str!(text, LibvcxErrorKind::InvalidOption);
    check_useful_opt_c_str!(version, LibvcxErrorKind::InvalidOption);
    check_useful_opt_c_str!(hash, LibvcxErrorKind::InvalidOption);
    check_useful_c_str!(acc_mech_type, LibvcxErrorKind::InvalidOption);

    trace!("vcx_set_active_txn_author_agreement_meta(text: {:?}, version: {:?}, hash: {:?}, acc_mech_type: {:?}, time_of_acceptance: {:?})", text, version, hash, acc_mech_type, time_of_acceptance);

    match ledger_set_txn_author_agreement(text, version, hash, acc_mech_type, time_of_acceptance) {
        Ok(()) => error::SUCCESS_ERR_CODE,
        Err(err) => err.into(),
    }
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

#[cfg(feature = "test_utils")]
pub mod test_utils {
    use aries_vcx::agency_client::testing::mocking::enable_agency_mocks;
    use aries_vcx::global::settings::{DEFAULT_WALLET_KEY, WALLET_KDF_RAW};

    use crate::api_c::cutils::return_types_u32;
    use crate::api_c::cutils::timeout::TimeoutUtils;
    use crate::api_c::vcx::vcx_open_main_pool;
    use crate::api_c::wallet::{
        vcx_configure_issuer_wallet, vcx_create_wallet, vcx_open_main_wallet, vcx_wallet_add_record,
        vcx_wallet_get_record,
    };
    use crate::errors::error;

    use super::*;

    pub fn _vcx_open_main_pool_c_closure(pool_config: &str) -> Result<(), u32> {
        let cb = return_types_u32::Return_U32::new().unwrap();

        let rc = vcx_open_main_pool(
            cb.command_handle,
            CString::new(pool_config).unwrap().into_raw(),
            cb.get_callback(),
        );
        if rc != error::SUCCESS_ERR_CODE {
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
        if rc != error::SUCCESS_ERR_CODE {
            return Err(rc);
        }
        cb.receive(TimeoutUtils::some_medium())
    }

    pub fn _vcx_init_threadpool_c_closure(config: &str) -> Result<(), u32> {
        let rc = vcx_init_threadpool(CString::new(config).unwrap().into_raw());
        if rc != error::SUCCESS_ERR_CODE {
            return Err(rc);
        }
        Ok(())
    }

    pub fn _vcx_init_threadpool(config_threadpool: &str) -> Result<(), u32> {
        info!("_vcx_init_threadpool >>>");
        let rc = vcx_init_threadpool(CString::new(config_threadpool).unwrap().into_raw());
        if rc != error::SUCCESS_ERR_CODE {
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
        if rc != error::SUCCESS_ERR_CODE {
            error!("vcx_open_pool failed");
            return Err(rc);
        }
        cb.receive(TimeoutUtils::some_short()).unwrap();
        Ok(())
    }

    pub fn _vcx_init_full(config_threadpool: &str, config_pool: &str, config_wallet: &str) -> Result<(), u32> {
        info!("_vcx_init_full >>>");
        let rc = vcx_init_threadpool(CString::new(config_threadpool).unwrap().into_raw());
        if rc != error::SUCCESS_ERR_CODE {
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
        if rc != error::SUCCESS_ERR_CODE {
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
        if rc != error::SUCCESS_ERR_CODE {
            error!("vcx_open_wallet failed");
            return Err(rc);
        }
        cb.receive(TimeoutUtils::some_custom(3)).unwrap();
        Ok(())
    }

    pub fn _vcx_open_wallet(config_wallet: &str) -> Result<(), u32> {
        info!("_vcx_init_full >>> going to open wallet");
        let cb = return_types_u32::Return_U32_I32::new().unwrap();
        let rc = vcx_open_main_wallet(
            cb.command_handle,
            CString::new(config_wallet.clone()).unwrap().into_raw(),
            Some(cb.get_callback()),
        );
        if rc != error::SUCCESS_ERR_CODE {
            error!("vcx_open_wallet failed");
            return Err(rc);
        }
        cb.receive(TimeoutUtils::some_custom(3)).unwrap();
        Ok(())
    }

    pub fn _vcx_create_wallet() -> Result<String, u32> {
        let wallet_name = format!("test_create_wallet_{}", uuid::Uuid::new_v4().to_string());
        let config_wallet = json!({
            "wallet_name": wallet_name,
            "wallet_key": DEFAULT_WALLET_KEY,
            "wallet_key_derivation": WALLET_KDF_RAW
        })
        .to_string();

        info!("_vcx_create_and_open_wallet >>>");

        let cb = return_types_u32::Return_U32::new().unwrap();
        let err = vcx_create_wallet(
            cb.command_handle,
            CString::new(format!("{}", config_wallet.clone())).unwrap().into_raw(),
            Some(cb.get_callback()),
        );
        assert_eq!(err, error::SUCCESS_ERR_CODE);
        cb.receive(TimeoutUtils::some_custom(3)).unwrap();
        Ok(config_wallet)
    }

    pub fn _vcx_create_and_open_wallet() -> Result<String, u32> {
        let config_wallet = _vcx_create_wallet()?;
        _vcx_open_wallet(&config_wallet)?;
        Ok(config_wallet)
    }

    pub fn _test_add_and_get_wallet_record() {
        let xtype = CStringUtils::string_to_cstring("record_type".to_string());
        let id = CStringUtils::string_to_cstring("123".to_string());
        let value = CStringUtils::string_to_cstring("Record Value".to_string());
        let tags = CStringUtils::string_to_cstring("{}".to_string());
        let options = json!({
            "retrieveType": true,
            "retrieveValue": true,
            "retrieveTags": false
        })
        .to_string();
        let options = CStringUtils::string_to_cstring(options);

        let cb = return_types_u32::Return_U32::new().unwrap();
        assert_eq!(
            vcx_wallet_add_record(
                cb.command_handle,
                xtype.as_ptr(),
                id.as_ptr(),
                value.as_ptr(),
                tags.as_ptr(),
                Some(cb.get_callback()),
            ),
            error::SUCCESS_ERR_CODE
        );
        cb.receive(TimeoutUtils::some_custom(1)).unwrap();

        let cb = return_types_u32::Return_U32_STR::new().unwrap();
        assert_eq!(
            vcx_wallet_get_record(
                cb.command_handle,
                xtype.as_ptr(),
                id.as_ptr(),
                options.as_ptr(),
                Some(cb.get_callback()),
            ),
            error::SUCCESS_ERR_CODE
        );
        let record_value = cb.receive(TimeoutUtils::some_custom(1)).unwrap().unwrap();
        assert!(record_value.contains("Record Value"));
    }
}

#[cfg(test)]
#[allow(unused_imports)] // TODO: remove it
mod tests {
    use aries_vcx::global::settings::{
        set_config_value, set_test_configs, CONFIG_GENESIS_PATH, CONFIG_TXN_AUTHOR_AGREEMENT,
        DEFAULT_WALLET_BACKUP_KEY, DEFAULT_WALLET_KEY, WALLET_KDF_RAW,
    };
    #[cfg(feature = "general_test")]
    use std::ptr;

    use aries_vcx::indy;
    #[cfg(feature = "pool_tests")]
    use aries_vcx::indy::ledger::pool::{
        test_utils::{create_tmp_genesis_txn_file, delete_named_test_pool, delete_test_pool},
        PoolConfig,
    };
    use aries_vcx::indy::wallet::{import, RestoreWalletConfigs, WalletConfig};
    use aries_vcx::utils::constants::GENESIS_PATH;
    use aries_vcx::utils::devsetup::{
        SetupDefaults, SetupEmpty, SetupMocks, SetupPoolConfig, TempFile, TestSetupCreateWallet,
    };
    use aries_vcx::utils::mockdata::mockdata_credex::ARIES_CREDENTIAL_OFFER;
    use aries_vcx::utils::mockdata::mockdata_proof::ARIES_PROOF_REQUEST_PRESENTATION;
    use aries_vcx::vdrtools::INVALID_WALLET_HANDLE;

    use crate::api_c;
    use crate::api_c::cutils::current_error::reset_current_error;
    use crate::api_c::cutils::return_types_u32;
    use crate::api_c::cutils::timeout::TimeoutUtils;
    use crate::api_c::protocols::mediated_connection::vcx_connection_create;
    use crate::api_c::vcx::test_utils::{
        _test_add_and_get_wallet_record, _vcx_create_and_open_wallet, _vcx_create_wallet, _vcx_init_threadpool,
        _vcx_init_threadpool_c_closure, _vcx_open_main_pool_c_closure, _vcx_open_main_wallet_c_closure, _vcx_open_pool,
        _vcx_open_wallet,
    };
    use crate::api_vcx;
    #[cfg(feature = "pool_tests")]
    use crate::api_vcx::api_global::pool::get_main_pool_handle;
    use crate::api_vcx::api_global::pool::reset_main_pool_handle;
    use crate::api_vcx::api_global::settings;
    use crate::api_vcx::api_global::wallet::test_utils::_create_main_wallet_and_its_backup;
    use crate::api_vcx::api_global::wallet::wallet_import;
    use crate::api_vcx::api_global::wallet::{close_main_wallet, get_main_wallet_handle};
    use crate::api_vcx::api_handle::{
        credential, credential_def, disclosed_proof, issuer_credential, mediated_connection, proof, schema,
    };
    use crate::errors::error;
    use crate::errors::error::{LibvcxErrorKind, LibvcxResult};

    use super::*;

    #[cfg(feature = "pool_tests")]
    #[tokio::test]
    async fn test_open_pool_fails_if_genesis_file_is_invalid() {
        let _setup = SetupDefaults::init();
        let pool_name = format!("invalidpool_{}", uuid::Uuid::new_v4().to_string());

        // Write invalid genesis.txn
        let _genesis_transactions = TempFile::create_with_data(GENESIS_PATH, "{}");

        // FIXME: actually use result
        let _ = set_config_value(CONFIG_GENESIS_PATH, &_genesis_transactions.path);

        let pool_config = PoolConfig {
            genesis_path: _genesis_transactions.path.clone(),
            pool_name: Some(pool_name.clone()),
            pool_config: None,
        };
        let err = _vcx_open_main_pool_c_closure(&json!(pool_config).to_string()).unwrap_err();
        assert_eq!(err, u32::from(LibvcxErrorKind::PoolLedgerConnect));
        assert_eq!(get_main_pool_handle().unwrap_err().kind(), LibvcxErrorKind::NoPoolOpen);

        delete_named_test_pool(0, &pool_name).await;
        reset_main_pool_handle();
    }

    #[cfg(feature = "pool_tests")]
    #[test]
    fn test_open_pool_fails_if_genesis_path_is_invalid() {
        let _setup = SetupDefaults::init();
        let pool_name = format!("invalidpool_{}", uuid::Uuid::new_v4().to_string());

        let pool_config = PoolConfig {
            genesis_path: "invalid/txn/path".to_string(),
            pool_name: Some(pool_name.clone()),
            pool_config: None,
        };
        let err = _vcx_open_main_pool_c_closure(&json!(pool_config).to_string()).unwrap_err();
        assert_eq!(err, u32::from(LibvcxErrorKind::InvalidGenesisTxnPath));
        assert_eq!(get_main_pool_handle().unwrap_err().kind(), LibvcxErrorKind::NoPoolOpen);
    }

    #[cfg(feature = "pool_tests")]
    #[tokio::test]
    async fn test_vcx_init_called_twice_passes_after_shutdown() {
        let _setup_defaults = SetupDefaults::init();
        for _ in 0..2 {
            TestSetupCreateWallet::run(|_| async {
                let setup_pool = SetupPoolConfig::init().await;

                let wallet_config = _vcx_create_wallet().unwrap();
                _vcx_init_threadpool("{}").unwrap();
                _vcx_open_pool(&json!(setup_pool.pool_config).to_string()).unwrap();
                _vcx_open_wallet(&wallet_config).unwrap();

                //Assert config values were set correctly
                assert_ne!(get_main_wallet_handle(), INVALID_WALLET_HANDLE);

                //Verify shutdown was successful
                vcx_shutdown(true);
                assert_eq!(get_main_wallet_handle(), INVALID_WALLET_HANDLE);

                return true; // skip_cleanup in TestSetupCreateWallet
            })
            .await;
        }
    }

    #[tokio::test]
    #[cfg(feature = "general_test")]
    async fn test_open_wallet_of_imported_wallet_succeeds() {
        let _setup = SetupDefaults::init();

        let (export_wallet_path, wallet_name, wallet_config) = _create_main_wallet_and_its_backup().await;

        indy::wallet::delete_wallet(&wallet_config).await.unwrap();

        let import_config = RestoreWalletConfigs {
            wallet_name: wallet_name.clone(),
            wallet_key: DEFAULT_WALLET_KEY.into(),
            exported_wallet_path: export_wallet_path.path.clone(),
            backup_key: DEFAULT_WALLET_BACKUP_KEY.to_string(),
            wallet_key_derivation: Some(WALLET_KDF_RAW.into()),
        };
        import(&import_config).await.unwrap();

        let content = json!({
            "wallet_name": &wallet_name,
            "wallet_key": DEFAULT_WALLET_KEY,
            "wallet_key_derivation": WALLET_KDF_RAW,
        })
        .to_string();

        _vcx_init_threadpool_c_closure("{}").unwrap();
        _vcx_open_main_wallet_c_closure(&content).unwrap();

        vcx_shutdown(true);
    }

    #[tokio::test]
    #[cfg(feature = "general_test")]
    async fn test_open_wallet_with_wrong_name_fails() {
        let _setup = SetupDefaults::init();

        let (export_wallet_path, _wallet_name, wallet_config) = _create_main_wallet_and_its_backup().await;

        indy::wallet::delete_wallet(&wallet_config).await.unwrap();

        let wallet_name = &format!("export_test_wallet_{}", uuid::Uuid::new_v4());
        let wallet_config = WalletConfig {
            wallet_name: wallet_name.into(),
            wallet_key: DEFAULT_WALLET_KEY.into(),
            wallet_key_derivation: WALLET_KDF_RAW.into(),
            wallet_type: None,
            storage_config: None,
            storage_credentials: None,
            rekey: None,
            rekey_derivation_method: None,
        };

        let import_config = RestoreWalletConfigs {
            wallet_name: wallet_config.wallet_name.clone(),
            wallet_key: wallet_config.wallet_key.clone(),
            exported_wallet_path: export_wallet_path.path.clone(),
            backup_key: DEFAULT_WALLET_BACKUP_KEY.to_string(),
            wallet_key_derivation: Some(wallet_config.wallet_key_derivation.clone()),
        };
        import(&import_config).await.unwrap();

        let content = json!({
            "wallet_name": "different_wallet_name",
            "wallet_key": DEFAULT_WALLET_KEY,
            "wallet_key_derivation": WALLET_KDF_RAW,
        })
        .to_string();

        _vcx_init_threadpool_c_closure("{}").unwrap();
        let err = _vcx_open_main_wallet_c_closure(&content).unwrap_err();
        assert_eq!(err, u32::from(LibvcxErrorKind::WalletNotFound));

        indy::wallet::delete_wallet(&wallet_config).await.unwrap();
    }

    #[test]
    #[cfg(feature = "general_test")]
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

    #[tokio::test]
    #[cfg(feature = "general_test")]
    async fn test_import_of_opened_wallet_fails() {
        let _setup = SetupDefaults::init();

        let (export_wallet_path, wallet_name, wallet_config) = _create_main_wallet_and_its_backup().await;

        _vcx_init_threadpool_c_closure("{}").unwrap();
        _vcx_open_main_wallet_c_closure(&serde_json::to_string(&wallet_config).unwrap()).unwrap();

        let import_config = RestoreWalletConfigs {
            wallet_name: wallet_name.into(),
            wallet_key: DEFAULT_WALLET_KEY.into(),
            exported_wallet_path: export_wallet_path.path.clone(),
            backup_key: DEFAULT_WALLET_BACKUP_KEY.to_string(),
            wallet_key_derivation: None,
        };
        assert_eq!(
            wallet_import(&import_config).await.unwrap_err().kind(),
            LibvcxErrorKind::DuplicationWallet
        );

        vcx_shutdown(true);
    }

    #[test]
    #[cfg(feature = "general_test")]
    fn test_init_no_config_path() {
        let _setup = SetupEmpty::init();
        assert_eq!(
            vcx_init_threadpool(ptr::null()),
            u32::from(LibvcxErrorKind::InvalidOption)
        )
    }

    #[test]
    #[cfg(feature = "general_test")]
    fn test_shutdown_with_no_previous_config() {
        let _setup = SetupDefaults::init();

        vcx_shutdown(true);
        vcx_shutdown(false);
    }

    #[tokio::test]
    #[cfg(feature = "general_test")]
    async fn test_shutdown() {
        let _setup = SetupMocks::init();

        let data = r#"["name","male"]"#;
        let connection = mediated_connection::tests::build_test_connection_inviter_invited().await;
        let credentialdef = credential_def::create("SID".to_string(), "id".to_string(), "tag".to_string(), false)
            .await
            .unwrap();
        let issuer_credential = issuer_credential::issuer_credential_create("1".to_string()).unwrap();
        let proof = proof::create_proof(
            "1".to_string(),
            "[]".to_string(),
            "[]".to_string(),
            r#"{"support_revocation":false}"#.to_string(),
            "Optional".to_owned(),
        )
        .await
        .unwrap();
        let schema = schema::create_and_publish_schema("5", "name".to_string(), "0.1".to_string(), data.to_string())
            .await
            .unwrap();
        let disclosed_proof =
            disclosed_proof::create_with_proof_request("id", ARIES_PROOF_REQUEST_PRESENTATION).unwrap();
        let credential = credential::credential_create_with_offer("name", ARIES_CREDENTIAL_OFFER).unwrap();

        vcx_shutdown(true);
        assert_eq!(mediated_connection::is_valid_handle(connection), false);
        assert_eq!(issuer_credential::is_valid_handle(issuer_credential), false);
        assert_eq!(schema::is_valid_handle(schema), false);
        assert_eq!(proof::is_valid_handle(proof), false);
        assert_eq!(credential_def::is_valid_handle(credentialdef), false);
        assert_eq!(credential::is_valid_handle(credential), false);
        assert_eq!(disclosed_proof::is_valid_handle(disclosed_proof), false);
        assert_eq!(get_main_wallet_handle(), INVALID_WALLET_HANDLE);
    }

    #[test]
    #[cfg(feature = "general_test")]
    fn test_vcx_version() {
        let _setup = SetupDefaults::init();

        let return_version = CStringUtils::c_str_to_string(vcx_version()).unwrap().unwrap();
        assert!(return_version.len() > 5);
    }

    #[test]
    #[cfg(feature = "general_test")]
    fn test_vcx_update_institution_webhook() {
        let _setup = SetupDefaults::init();

        let webhook_url = "https://example.com";
        let cb = return_types_u32::Return_U32::new().unwrap();
        assert_eq!(
            error::SUCCESS_ERR_CODE,
            vcx_update_webhook_url(
                cb.command_handle,
                CString::new(webhook_url.to_string()).unwrap().into_raw(),
                Some(cb.get_callback()),
            )
        );
    }

    #[test]
    #[cfg(feature = "general_test")]
    fn get_current_error_works_for_no_error() {
        let _setup = SetupDefaults::init();

        reset_current_error();

        let mut error_json_p: *const c_char = ptr::null();

        vcx_get_current_error(&mut error_json_p);
        assert_eq!(None, CStringUtils::c_str_to_string(error_json_p).unwrap());
    }

    #[test]
    #[cfg(feature = "general_test")]
    fn get_current_error_works_for_sync_error() {
        let _setup = SetupDefaults::init();

        let cb = return_types_u32::Return_U32_STR::new().unwrap();
        api_c::utils::vcx_provision_cloud_agent(cb.command_handle, ptr::null(), Some(cb.get_callback()));

        let mut error_json_p: *const c_char = ptr::null();
        vcx_get_current_error(&mut error_json_p);
        assert!(CStringUtils::c_str_to_string(error_json_p).unwrap().is_some());
    }

    #[test]
    #[cfg(feature = "general_test")]
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
    #[cfg(feature = "general_test")]
    fn test_vcx_set_active_txn_author_agreement_meta() {
        let _setup = SetupMocks::init();

        assert!(&settings::get_config_value(CONFIG_TXN_AUTHOR_AGREEMENT).is_err());

        let text = "text";
        let version = "1.0.0";
        let acc_mech_type = "type 1";
        let time_of_acceptance = 123456789;

        assert_eq!(
            error::SUCCESS_ERR_CODE,
            vcx_set_active_txn_author_agreement_meta(
                CString::new(text.to_string()).unwrap().into_raw(),
                CString::new(version.to_string()).unwrap().into_raw(),
                std::ptr::null(),
                CString::new(acc_mech_type.to_string()).unwrap().into_raw(),
                time_of_acceptance,
            )
        );

        let expected = json!({
            "text": text,
            "version": version,
            "acceptanceMechanismType": acc_mech_type,
            "timeOfAcceptance": time_of_acceptance,
        });

        let auth_agreement = settings::get_config_value(CONFIG_TXN_AUTHOR_AGREEMENT).unwrap();
        let auth_agreement = serde_json::from_str::<::serde_json::Value>(&auth_agreement).unwrap();

        assert_eq!(expected, auth_agreement);

        set_test_configs();
    }

    #[test]
    #[cfg(feature = "general_test")]
    fn test_vcx_get_ledger_author_agreement() {
        let _setup = SetupMocks::init();

        let cb = return_types_u32::Return_U32_STR::new().unwrap();
        assert_eq!(
            vcx_get_ledger_author_agreement(cb.command_handle, Some(cb.get_callback())),
            error::SUCCESS_ERR_CODE
        );
        let agreement = cb.receive(TimeoutUtils::some_short()).unwrap();
        assert_eq!(
            aries_vcx::utils::constants::DEFAULT_AUTHOR_AGREEMENT,
            agreement.unwrap()
        );
    }

    #[test]
    #[cfg(feature = "general_test")]
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

    #[tokio::test]
    #[cfg(feature = "pool_tests")]
    async fn test_open_pool() {
        let _setup = SetupEmpty::init();

        let genesis_path = create_tmp_genesis_txn_file();
        let config = PoolConfig {
            genesis_path,
            pool_name: None,
            pool_config: None,
        };
        _vcx_open_main_pool_c_closure(&json!(config).to_string()).unwrap();

        delete_test_pool(get_main_pool_handle().unwrap()).await;
        reset_main_pool_handle();
    }

    #[tokio::test]
    #[cfg(feature = "general_test")]
    async fn test_open_wallet() -> LibvcxResult<()> {
        let _ = _vcx_create_and_open_wallet();
        _test_add_and_get_wallet_record();
        close_main_wallet().await?;
        Ok(())
    }

    #[cfg(feature = "pool_tests")]
    #[tokio::test]
    async fn test_full_init() {
        let _setup_defaults = SetupDefaults::init();
        let setup_pool = SetupPoolConfig::init().await;

        let wallet_config = _vcx_create_wallet().unwrap();
        _vcx_init_threadpool("{}").unwrap();
        _vcx_open_pool(&json!(setup_pool.pool_config).to_string()).unwrap();
        _vcx_open_wallet(&wallet_config).unwrap();

        // Assert pool was initialized
        assert_ne!(get_main_pool_handle().unwrap(), 0);
        delete_test_pool(get_main_pool_handle().unwrap()).await;
        reset_main_pool_handle();
    }
}
