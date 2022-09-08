use std::ffi::CString;

use futures::future::{BoxFuture, FutureExt};
use libc::c_char;

use aries_vcx::agency_client::configuration::AgencyClientConfig;
use aries_vcx::agency_client::testing::mocking::enable_agency_mocks;
use aries_vcx::error::{VcxError, VcxErrorKind};
use aries_vcx::global::pool::{is_main_pool_open, open_main_pool, get_main_pool_handle, close_main_pool};
use aries_vcx::global::settings;
use aries_vcx::global::settings::{enable_indy_mocks, init_issuer_config};
use aries_vcx::indy::CommandHandle;
use aries_vcx::libindy::utils::pool::PoolConfig;
use aries_vcx::libindy::utils::wallet::{IssuerConfig, WalletConfig};
use aries_vcx::libindy::utils::{ledger, pool, wallet};
use aries_vcx::utils;
use aries_vcx::utils::error;
use aries_vcx::utils::version_constants;

use crate::api_lib;
use crate::api_lib::api_handle::utils::agency_update_agent_webhook;
use crate::api_lib::global::agency_client::create_agency_client_for_main_wallet;
use crate::api_lib::global::wallet::close_main_wallet;
use crate::api_lib::utils::cstring::CStringUtils;
use crate::api_lib::utils::error::{get_current_error_c_json, set_current_error, set_current_error_vcx};
use crate::api_lib::utils::runtime::{execute, execute_async, init_threadpool};

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
    match enable_indy_mocks() {
        Ok(_) => {}
        Err(_) => return error::UNKNOWN_ERROR.code_num,
    };
    enable_agency_mocks();
    return error::SUCCESS.code_num;
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

    check_useful_c_str!(config, VcxErrorKind::InvalidOption);

    match init_threadpool(&config) {
        Ok(_) => error::SUCCESS.code_num,
        Err(_) => error::UNKNOWN_ERROR.code_num,
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

    check_useful_c_str!(config, VcxErrorKind::InvalidOption);
    check_useful_c_callback!(cb, VcxErrorKind::InvalidOption);

    trace!("vcx_create_agency_client_for_main_wallet >>> config: {}", config);

    let agency_config = match serde_json::from_str::<AgencyClientConfig>(&config) {
        Ok(agency_config) => agency_config,
        Err(err) => {
            set_current_error(&err);
            error!(
                "vcx_create_agency_client_for_main_wallet >>> invalid configuration, err: {:?}",
                err
            );
            return error::INVALID_CONFIGURATION.code_num;
        }
    };

    execute(move || {
        match create_agency_client_for_main_wallet(&agency_config) {
            Ok(()) => {
                info!(
                    "vcx_create_agency_client_for_main_wallet_cb >>> command_handle: {}, rc {}",
                    command_handle,
                    error::SUCCESS.code_num
                );
                cb(command_handle, error::SUCCESS.code_num)
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
    error::SUCCESS.code_num
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

    check_useful_c_str!(config, VcxErrorKind::InvalidOption);
    check_useful_c_callback!(cb, VcxErrorKind::InvalidOption);

    trace!("vcx_init_issuer_config >>> config: {}", config);

    let issuer_config = match serde_json::from_str::<IssuerConfig>(&config) {
        Ok(issuer_config) => issuer_config,
        Err(err) => {
            set_current_error(&err);
            error!("vcx_init_issuer_config >>> invalid configuration, err: {:?}", err);
            return error::INVALID_CONFIGURATION.code_num;
        }
    };

    execute(move || {
        match init_issuer_config(&issuer_config) {
            Ok(()) => {
                info!(
                    "vcx_init_issuer_config_cb >>> command_handle: {}, rc: {}",
                    command_handle,
                    error::SUCCESS.code_num
                );
                cb(command_handle, error::SUCCESS.code_num)
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
    error::SUCCESS.code_num
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
    check_useful_c_str!(pool_config, VcxErrorKind::InvalidOption);
    if is_main_pool_open() {
        error!("vcx_open_main_pool :: Pool connection is already open.");
        return VcxError::from_msg(VcxErrorKind::AlreadyInitialized, "Pool connection is already open.").into();
    }

    let pool_config = match serde_json::from_str::<PoolConfig>(&pool_config) {
        Ok(pool_config) => pool_config,
        Err(err) => {
            set_current_error(&err);
            error!("vcx_open_main_pool >>> invalid wallet configuration; err: {:?}", err);
            return error::INVALID_CONFIGURATION.code_num;
        }
    };

    execute_async::<BoxFuture<'static, Result<(), ()>>>(Box::pin(async move {
        match open_main_pool(&pool_config).await {
            Ok(()) => {
                info!("vcx_open_main_pool_cb :: Vcx Pool Init Successful");
                cb(command_handle, error::SUCCESS.code_num)
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
    error::SUCCESS.code_num
}

lazy_static! {
    pub static ref VERSION_STRING: CString =
        CString::new(format!("{}{}", version_constants::VERSION, version_constants::REVISION)).unwrap();
}

#[no_mangle]
pub extern "C" fn vcx_version() -> *const c_char {
    VERSION_STRING.as_ptr()
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

    match futures::executor::block_on(api_lib::global::wallet::close_main_wallet()) {
        Ok(()) => {}
        Err(_) => {}
    };

    match futures::executor::block_on(close_main_pool()) {
        Ok(()) => {}
        Err(_) => {}
    };

    crate::api_lib::api_handle::schema::release_all();
    crate::api_lib::api_handle::connection::release_all();
    crate::api_lib::api_handle::issuer_credential::release_all();
    crate::api_lib::api_handle::credential_def::release_all();
    crate::api_lib::api_handle::proof::release_all();
    crate::api_lib::api_handle::disclosed_proof::release_all();
    crate::api_lib::api_handle::credential::release_all();

    if delete {
        let pool_name =
            settings::get_config_value(settings::CONFIG_POOL_NAME).unwrap_or(settings::DEFAULT_POOL_NAME.to_string());
        let wallet_name = settings::get_config_value(settings::CONFIG_WALLET_NAME)
            .unwrap_or(settings::DEFAULT_WALLET_NAME.to_string());
        let wallet_type = settings::get_config_value(settings::CONFIG_WALLET_TYPE).ok();
        let wallet_key = settings::get_config_value(settings::CONFIG_WALLET_KEY)
            .unwrap_or(settings::UNINITIALIZED_WALLET_KEY.into());
        let wallet_key_derivation = settings::get_config_value(settings::CONFIG_WALLET_KEY_DERIVATION)
            .unwrap_or(settings::WALLET_KDF_DEFAULT.into());

        let _res = futures::executor::block_on(close_main_wallet());

        let wallet_config = WalletConfig {
            wallet_name,
            wallet_key,
            wallet_key_derivation,
            wallet_type,
            storage_config: None,
            storage_credentials: None,
            rekey: None,
            rekey_derivation_method: None,
        };

        match futures::executor::block_on(wallet::delete_wallet(&wallet_config)) {
            Ok(()) => (),
            Err(_) => (),
        };

        match futures::executor::block_on(pool::delete(&pool_name)) {
            Ok(()) => (),
            Err(_) => (),
        };
    }

    settings::reset_config_values();
    api_lib::global::agency_client::reset_main_agency_client();
    trace!("vcx_shutdown(delete: {})", delete);

    error::SUCCESS.code_num
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
    info!("vcx_error_c_message >>>");
    trace!("vcx_error_message(error_code: {})", error_code);
    error::error_c_message(&error_code).as_ptr()
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

    check_useful_c_str!(notification_webhook_url, VcxErrorKind::InvalidOption);
    check_useful_c_callback!(cb, VcxErrorKind::InvalidOption);

    trace!("vcx_update_webhook(webhook_url: {})", notification_webhook_url);

    execute_async::<BoxFuture<'static, Result<(), ()>>>(
        async move {
            match agency_update_agent_webhook(&notification_webhook_url[..]).await {
                Ok(()) => {
                    trace!(
                        "vcx_update_webhook_url_cb(command_handle: {}, rc: {})",
                        command_handle,
                        error::SUCCESS.message
                    );

                    cb(command_handle, error::SUCCESS.code_num);
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

    error::SUCCESS.code_num
}

#[no_mangle]
pub extern "C" fn vcx_get_ledger_author_agreement(
    command_handle: CommandHandle,
    cb: Option<extern "C" fn(xcommand_handle: CommandHandle, err: u32, author_agreement: *const c_char)>,
) -> u32 {
    info!("vcx_get_ledger_author_agreement >>>");

    check_useful_c_callback!(cb, VcxErrorKind::InvalidOption);

    trace!("vcx_get_ledger_author_agreement(command_handle: {})", command_handle);

    let pool_handle = match get_main_pool_handle() {
        Ok(handle) => handle,
        Err(err) => return err.into(),
    };

    execute_async::<BoxFuture<'static, Result<(), ()>>>(
        async move {
            match ledger::libindy_get_txn_author_agreement(pool_handle).await {
                Ok(err) => {
                    trace!(
                        "vcx_get_ledger_author_agreement(command_handle: {}, rc: {}, author_agreement: {})",
                        command_handle,
                        error::SUCCESS.message,
                        err
                    );

                    let msg = CStringUtils::string_to_cstring(err);
                    cb(command_handle, error::SUCCESS.code_num, msg.as_ptr());
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

    error::SUCCESS.code_num
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

    check_useful_opt_c_str!(text, VcxErrorKind::InvalidOption);
    check_useful_opt_c_str!(version, VcxErrorKind::InvalidOption);
    check_useful_opt_c_str!(hash, VcxErrorKind::InvalidOption);
    check_useful_c_str!(acc_mech_type, VcxErrorKind::InvalidOption);

    trace!("vcx_set_active_txn_author_agreement_meta(text: {:?}, version: {:?}, hash: {:?}, acc_mech_type: {:?}, time_of_acceptance: {:?})", text, version, hash, acc_mech_type, time_of_acceptance);

    match utils::author_agreement::set_txn_author_agreement(text, version, hash, acc_mech_type, time_of_acceptance) {
        Ok(()) => error::SUCCESS.code_num,
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
#[allow(unused_imports)]
pub mod test_utils {
    use aries_vcx::agency_client::testing::mocking::enable_agency_mocks;
    use aries_vcx::error::VcxResult;

    use crate::api_lib::api_c::vcx::vcx_open_main_pool;
    use crate::api_lib::api_c::wallet::{
        vcx_configure_issuer_wallet, vcx_create_wallet, vcx_open_main_wallet, vcx_wallet_add_record,
        vcx_wallet_get_record,
    };
    use crate::api_lib::utils::return_types_u32;
    use crate::api_lib::utils::timeout::TimeoutUtils;

    use super::*;

    pub fn _vcx_open_main_pool_c_closure(pool_config: &str) -> Result<(), u32> {
        let cb = return_types_u32::Return_U32::new().unwrap();

        let rc = vcx_open_main_pool(
            cb.command_handle,
            CString::new(pool_config).unwrap().into_raw(),
            cb.get_callback(),
        );
        if rc != error::SUCCESS.code_num {
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
        if rc != error::SUCCESS.code_num {
            return Err(rc);
        }
        cb.receive(TimeoutUtils::some_medium())
    }

    pub fn _vcx_init_threadpool_c_closure(config: &str) -> Result<(), u32> {
        let rc = vcx_init_threadpool(CString::new(config).unwrap().into_raw());
        if rc != error::SUCCESS.code_num {
            return Err(rc);
        }
        Ok(())
    }

    pub fn _vcx_init_threadpool(config_threadpool: &str) -> Result<(), u32> {
        info!("_vcx_init_threadpool >>>");
        let rc = vcx_init_threadpool(CString::new(config_threadpool).unwrap().into_raw());
        if rc != error::SUCCESS.code_num {
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
        if rc != error::SUCCESS.code_num {
            error!("vcx_open_pool failed");
            return Err(rc);
        }
        cb.receive(TimeoutUtils::some_short()).unwrap();
        Ok(())
    }

    pub fn _vcx_init_full(config_threadpool: &str, config_pool: &str, config_wallet: &str) -> Result<(), u32> {
        info!("_vcx_init_full >>>");
        let rc = vcx_init_threadpool(CString::new(config_threadpool).unwrap().into_raw());
        if rc != error::SUCCESS.code_num {
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
        if rc != error::SUCCESS.code_num {
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
        if rc != error::SUCCESS.code_num {
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
        if rc != error::SUCCESS.code_num {
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
            "wallet_key": settings::DEFAULT_WALLET_KEY,
            "wallet_key_derivation": settings::WALLET_KDF_RAW
        })
        .to_string();

        info!("_vcx_create_and_open_wallet >>>");

        let cb = return_types_u32::Return_U32::new().unwrap();
        let err = vcx_create_wallet(
            cb.command_handle,
            CString::new(format!("{}", config_wallet.clone())).unwrap().into_raw(),
            Some(cb.get_callback()),
        );
        assert_eq!(err, error::SUCCESS.code_num);
        cb.receive(TimeoutUtils::some_custom(3)).unwrap();
        Ok(config_wallet)
    }

    pub fn _vcx_configure_issuer_wallet(seed: &str) -> String {
        let cb = return_types_u32::Return_U32_STR::new().unwrap();
        vcx_configure_issuer_wallet(
            cb.command_handle,
            CString::new(String::from(seed)).unwrap().into_raw(),
            Some(cb.get_callback()),
        );
        let issuer_config = cb.receive(TimeoutUtils::some_custom(1)).unwrap().unwrap();
        return issuer_config;
    }

    pub fn _vcx_configure_issuer(config: &str) -> VcxResult<()> {
        let cb = return_types_u32::Return_U32::new().unwrap();
        vcx_init_issuer_config(
            cb.command_handle,
            CString::new(String::from(config)).unwrap().into_raw(),
            Some(cb.get_callback()),
        );
        cb.receive(TimeoutUtils::some_custom(1)).unwrap();
        Ok(())
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
                Some(cb.get_callback())
            ),
            error::SUCCESS.code_num
        );
        cb.receive(TimeoutUtils::some_custom(1)).unwrap();

        let cb = return_types_u32::Return_U32_STR::new().unwrap();
        assert_eq!(
            vcx_wallet_get_record(
                cb.command_handle,
                xtype.as_ptr(),
                id.as_ptr(),
                options.as_ptr(),
                Some(cb.get_callback())
            ),
            error::SUCCESS.code_num
        );
        let record_value = cb.receive(TimeoutUtils::some_custom(1)).unwrap().unwrap();
        assert!(record_value.contains("Record Value"));
    }
}

#[cfg(test)]
#[allow(unused_imports)]
mod tests {
    use std::ptr;

    use aries_vcx::agency_client::configuration::AgentProvisionConfig;
    use aries_vcx::global;
    use aries_vcx::global::pool::get_main_pool_handle;
    use aries_vcx::global::settings;
    use aries_vcx::indy::INVALID_WALLET_HANDLE;
    use aries_vcx::libindy::utils::anoncreds::test_utils::create_and_store_credential_def;
    use aries_vcx::libindy::utils::pool::test_utils::{
        create_tmp_genesis_txn_file, delete_named_test_pool, delete_test_pool
    };
    use aries_vcx::libindy::utils::pool::PoolConfig;
    use aries_vcx::libindy::utils::wallet::{import, RestoreWalletConfigs, WalletConfig};
    use aries_vcx::utils::constants;
    use aries_vcx::utils::devsetup::{
        SetupDefaults, SetupEmpty, SetupMocks, SetupPoolConfig, TempFile, TestSetupCreateWallet, AGENCY_DID,
        AGENCY_ENDPOINT, AGENCY_VERKEY,
    };

    use crate::api_lib;
    use crate::api_lib::api_c;
    use crate::api_lib::api_c::connection::vcx_connection_create;
    use crate::api_lib::api_c::utils::vcx_provision_cloud_agent;
    use crate::api_lib::api_c::vcx::test_utils::{
        _test_add_and_get_wallet_record, _vcx_configure_issuer, _vcx_configure_issuer_wallet,
        _vcx_create_and_open_wallet, _vcx_create_wallet, _vcx_init_full, _vcx_init_threadpool,
        _vcx_init_threadpool_c_closure, _vcx_open_main_pool_c_closure, _vcx_open_main_wallet_c_closure, _vcx_open_pool,
        _vcx_open_wallet,
    };
    use crate::api_lib::api_c::wallet::{
        vcx_close_main_wallet, vcx_configure_issuer_wallet, vcx_create_wallet, vcx_open_main_wallet,
    };
    use crate::api_lib::api_handle::{
        connection, credential, credential_def, disclosed_proof, issuer_credential, proof, schema,
    };
    #[cfg(feature = "pool_tests")]
    use crate::api_lib::global::wallet::get_main_wallet_handle;
    use crate::api_lib::global::wallet::test_utils::_create_main_wallet_and_its_backup;
    use crate::api_lib::utils::error::reset_current_error;
    use crate::api_lib::utils::return_types_u32;
    use crate::api_lib::utils::timeout::TimeoutUtils;

    use super::*;

    #[cfg(feature = "pool_tests")]
    #[tokio::test]
    async fn test_open_pool_fails_if_genesis_file_is_invalid() {
        let _setup = SetupDefaults::init();
        let pool_name = format!("invalidpool_{}", uuid::Uuid::new_v4().to_string());

        // Write invalid genesis.txn
        let _genesis_transactions = TempFile::create_with_data(utils::constants::GENESIS_PATH, "{}");
        settings::set_config_value(settings::CONFIG_GENESIS_PATH, &_genesis_transactions.path);

        let pool_config = PoolConfig {
            genesis_path: _genesis_transactions.path.clone(),
            pool_name: Some(pool_name.clone()),
            pool_config: None,
        };
        let err = _vcx_open_main_pool_c_closure(&json!(pool_config).to_string()).unwrap_err();
        assert_eq!(err, error::POOL_LEDGER_CONNECT.code_num);
        assert_eq!(
            get_main_pool_handle().unwrap_err().kind(),
            aries_vcx::error::VcxErrorKind::NoPoolOpen
        );

        delete_named_test_pool(0, &pool_name).await;
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
        assert_eq!(err, error::INVALID_GENESIS_TXN_PATH.code_num);
        assert_eq!(
            get_main_pool_handle().unwrap_err().kind(),
            aries_vcx::error::VcxErrorKind::NoPoolOpen
        );
    }

    #[cfg(feature = "pool_tests")]
    #[tokio::test]
    async fn test_vcx_init_called_twice_passes_after_shutdown() {
        let _setup_defaults = SetupDefaults::init();
        for _ in 0..2 {
            let setup_wallet = TestSetupCreateWallet::init().await.skip_cleanup();
            let setup_pool = SetupPoolConfig::init().await.skip_cleanup();

            let wallet_config = _vcx_create_wallet().unwrap();
            _vcx_init_threadpool("{}").unwrap();
            _vcx_open_pool(&json!(setup_pool.pool_config).to_string()).unwrap();
            _vcx_open_wallet(&wallet_config).unwrap();

            //Assert config values were set correctly
            assert_ne!(get_main_wallet_handle(), INVALID_WALLET_HANDLE);

            //Verify shutdown was successful
            vcx_shutdown(true);
            assert_eq!(get_main_wallet_handle(), INVALID_WALLET_HANDLE);
        }
    }

    #[tokio::test]
    #[cfg(feature = "general_test")]
    async fn test_open_wallet_of_imported_wallet_succeeds() {
        let _setup = SetupDefaults::init();

        let (export_wallet_path, wallet_name, wallet_config) = _create_main_wallet_and_its_backup().await;

        wallet::delete_wallet(&wallet_config).await.unwrap();

        let import_config = RestoreWalletConfigs {
            wallet_name: wallet_name.clone(),
            wallet_key: settings::DEFAULT_WALLET_KEY.into(),
            exported_wallet_path: export_wallet_path.path.clone(),
            backup_key: settings::DEFAULT_WALLET_BACKUP_KEY.to_string(),
            wallet_key_derivation: Some(settings::WALLET_KDF_RAW.into()),
        };
        import(&import_config).await.unwrap();

        let content = json!({
            "wallet_name": &wallet_name,
            "wallet_key": settings::DEFAULT_WALLET_KEY,
            "wallet_key_derivation": settings::WALLET_KDF_RAW,
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

        wallet::delete_wallet(&wallet_config).await.unwrap();

        let wallet_name = &format!("export_test_wallet_{}", uuid::Uuid::new_v4());
        let wallet_config = WalletConfig {
            wallet_name: wallet_name.into(),
            wallet_key: settings::DEFAULT_WALLET_KEY.into(),
            wallet_key_derivation: settings::WALLET_KDF_RAW.into(),
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
            backup_key: settings::DEFAULT_WALLET_BACKUP_KEY.to_string(),
            wallet_key_derivation: Some(wallet_config.wallet_key_derivation.clone()),
        };
        import(&import_config).await.unwrap();

        let content = json!({
            "wallet_name": "different_wallet_name",
            "wallet_key": settings::DEFAULT_WALLET_KEY,
            "wallet_key_derivation": settings::WALLET_KDF_RAW,
        })
        .to_string();

        _vcx_init_threadpool_c_closure("{}").unwrap();
        let err = _vcx_open_main_wallet_c_closure(&content).unwrap_err();
        assert_eq!(err, error::WALLET_NOT_FOUND.code_num);

        wallet::delete_wallet(&wallet_config).await.unwrap();
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
            wallet_key: settings::DEFAULT_WALLET_KEY.into(),
            exported_wallet_path: export_wallet_path.path.clone(),
            backup_key: settings::DEFAULT_WALLET_BACKUP_KEY.to_string(),
            wallet_key_derivation: None,
        };
        assert_eq!(
            import(&import_config).await.unwrap_err().kind(),
            aries_vcx::error::VcxErrorKind::DuplicationWallet
        );

        vcx_shutdown(true);
    }

    #[test]
    #[cfg(feature = "general_test")]
    fn test_init_no_config_path() {
        let _setup = SetupEmpty::init();
        assert_eq!(vcx_init_threadpool(ptr::null()), error::INVALID_OPTION.code_num)
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
        let connection = connection::tests::build_test_connection_inviter_invited().await;
        let credentialdef = credential_def::create(
            "SID".to_string(),
            "4fUDR9R7fjwELRvH9JT6HH".to_string(),
            "id".to_string(),
            "tag".to_string(),
            false,
        )
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
        let schema = schema::create_and_publish_schema(
            "5",
            "VsKV7grR1BUE29mG2Fm2kX".to_string(),
            "name".to_string(),
            "0.1".to_string(),
            data.to_string(),
        )
        .await
        .unwrap();
        let disclosed_proof =
            disclosed_proof::create_proof("id", utils::mockdata::mockdata_proof::ARIES_PROOF_REQUEST_PRESENTATION)
                .unwrap();
        let credential =
            credential::credential_create_with_offer("name", utils::mockdata::mockdata_credex::ARIES_CREDENTIAL_OFFER)
                .unwrap();

        vcx_shutdown(true);
        assert_eq!(
            connection::release(connection).unwrap_err().kind(),
            VcxErrorKind::InvalidConnectionHandle
        );
        assert_eq!(
            issuer_credential::release(issuer_credential).unwrap_err().kind(),
            VcxErrorKind::InvalidIssuerCredentialHandle
        );
        assert_eq!(
            schema::release(schema).unwrap_err().kind(),
            VcxErrorKind::InvalidSchemaHandle
        );
        assert_eq!(
            proof::release(proof).unwrap_err().kind(),
            VcxErrorKind::InvalidProofHandle
        );
        assert_eq!(
            credential_def::release(credentialdef).unwrap_err().kind(),
            VcxErrorKind::InvalidCredDefHandle
        );
        assert_eq!(
            credential::release(credential).unwrap_err().kind(),
            VcxErrorKind::InvalidCredentialHandle
        );
        assert_eq!(
            disclosed_proof::release(disclosed_proof).unwrap_err().kind(),
            VcxErrorKind::InvalidDisclosedProofHandle
        );
        assert_eq!(api_lib::global::wallet::get_main_wallet_handle(), INVALID_WALLET_HANDLE);
    }

    #[test]
    #[cfg(feature = "general_test")]
    fn test_error_c_message() {
        let _setup = SetupMocks::init();

        let c_message = CStringUtils::c_str_to_string(vcx_error_c_message(0)).unwrap().unwrap();
        assert_eq!(c_message, error::SUCCESS.message);

        let c_message = CStringUtils::c_str_to_string(vcx_error_c_message(1001))
            .unwrap()
            .unwrap();
        assert_eq!(c_message, error::UNKNOWN_ERROR.message);

        let c_message = CStringUtils::c_str_to_string(vcx_error_c_message(100100))
            .unwrap()
            .unwrap();
        assert_eq!(c_message, error::UNKNOWN_ERROR.message);

        let c_message = CStringUtils::c_str_to_string(vcx_error_c_message(1021))
            .unwrap()
            .unwrap();
        assert_eq!(c_message, error::INVALID_ATTRIBUTES_STRUCTURE.message);
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
            error::SUCCESS.code_num,
            vcx_update_webhook_url(
                cb.command_handle,
                CString::new(webhook_url.to_string()).unwrap().into_raw(),
                Some(cb.get_callback())
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

        assert!(&settings::get_config_value(settings::CONFIG_TXN_AUTHOR_AGREEMENT).is_err());

        let text = "text";
        let version = "1.0.0";
        let acc_mech_type = "type 1";
        let time_of_acceptance = 123456789;

        assert_eq!(
            error::SUCCESS.code_num,
            vcx_set_active_txn_author_agreement_meta(
                CString::new(text.to_string()).unwrap().into_raw(),
                CString::new(version.to_string()).unwrap().into_raw(),
                std::ptr::null(),
                CString::new(acc_mech_type.to_string()).unwrap().into_raw(),
                time_of_acceptance
            )
        );

        let expected = json!({
            "text": text,
            "version": version,
            "acceptanceMechanismType": acc_mech_type,
            "timeOfAcceptance": time_of_acceptance,
        });

        let auth_agreement = settings::get_config_value(settings::CONFIG_TXN_AUTHOR_AGREEMENT).unwrap();
        let auth_agreement = serde_json::from_str::<::serde_json::Value>(&auth_agreement).unwrap();

        assert_eq!(expected, auth_agreement);

        settings::set_test_configs();
    }

    #[test]
    #[cfg(feature = "general_test")]
    fn test_vcx_get_ledger_author_agreement() {
        let _setup = SetupMocks::init();

        let cb = return_types_u32::Return_U32_STR::new().unwrap();
        assert_eq!(
            vcx_get_ledger_author_agreement(cb.command_handle, Some(cb.get_callback())),
            error::SUCCESS.code_num
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

        delete_test_pool(0).await;
        settings::set_test_configs();
    }

    #[tokio::test]
    #[cfg(feature = "general_test")]
    async fn test_open_wallet() {
        _vcx_create_and_open_wallet();
        _test_add_and_get_wallet_record();
        close_main_wallet().await;
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
    }

    #[cfg(feature = "agency_tests")]
    #[tokio::test]
    async fn test_provision_cloud_agent() {
        let _setup_defaults = SetupDefaults::init();

        let rc = vcx_init_threadpool(CString::new("{}").unwrap().into_raw());
        assert_eq!(rc, error::SUCCESS.code_num);

        let config_wallet = _vcx_create_and_open_wallet().unwrap();

        let config_provision_agent = AgentProvisionConfig {
            agency_did: AGENCY_DID.to_string(),
            agency_verkey: AGENCY_VERKEY.to_string(),
            agency_endpoint: AGENCY_ENDPOINT.to_string(),
            agent_seed: None,
        };
        let config_provision_agent: &str = &json!(config_provision_agent).to_string();
        let cb = return_types_u32::Return_U32_STR::new().unwrap();
        let rc = vcx_provision_cloud_agent(
            cb.command_handle,
            CString::new(config_provision_agent).unwrap().into_raw(),
            Some(cb.get_callback()),
        );
        assert_eq!(rc, error::SUCCESS.code_num);
        let agency_client_config = cb.receive(TimeoutUtils::some_custom(3)).unwrap().unwrap();

        let cb = return_types_u32::Return_U32::new().unwrap();
        vcx_create_agency_client_for_main_wallet(
            cb.command_handle,
            CString::new(agency_client_config).unwrap().into_raw(),
            Some(cb.get_callback()),
        );
        let webhook_url = "https://example.com";
        cb.receive(TimeoutUtils::some_custom(1)).unwrap();

        let cb = return_types_u32::Return_U32::new().unwrap();
        assert_eq!(
            error::SUCCESS.code_num,
            vcx_update_webhook_url(
                cb.command_handle,
                CString::new(webhook_url.to_string()).unwrap().into_raw(),
                Some(cb.get_callback())
            )
        );
        cb.receive(TimeoutUtils::some_custom(2)).unwrap();
    }
}
