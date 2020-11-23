use std::ffi::CString;

use indy::{CommandHandle, INVALID_WALLET_HANDLE};
use indy_sys::WalletHandle;
use libc::c_char;

use crate::{libindy, settings, utils};
use crate::error::prelude::*;
use crate::init::{init_core, open_as_main_wallet, open_pool};
use crate::libindy::utils::{ledger, pool, wallet};
use crate::libindy::utils::pool::is_pool_open;
use crate::libindy::utils::wallet::{close_main_wallet, get_wallet_handle, set_wallet_handle};
use crate::utils::cstring::CStringUtils;
use crate::utils::error;
use crate::utils::threadpool::spawn;
use crate::utils::version_constants;

/// Initializes VCX with config settings
///
/// #Params
/// command_handle: command handle to map callback to user context.
///
/// config: The agent provision configuration. You can produce this by provisioning agent using function vcx_provision_agent
///
/// cb: Callback that provides error status of initialization
///
/// #Returns
/// Error code as a u32
#[no_mangle]
pub extern fn vcx_init_core(config: *const c_char) -> u32 {
    info!("vcx_init_core >>>");
    info!("libvcx version: {}{}", version_constants::VERSION, version_constants::REVISION);
    check_useful_c_str!(config, VcxErrorKind::InvalidOption);
    match init_core(&config) {
        Ok(_) => error::SUCCESS.code_num,
        Err(_) => error::INVALID_CONFIGURATION.code_num
    }
}

/// Opens pool based on vcx configuration previously set via vcx_init_core
///
/// #Params
/// command_handle: command handle to map callback to user context.
///
/// cb: Callback that provides error status of initialization
///
/// #Returns
/// Error code as a u32
#[no_mangle]
pub extern fn vcx_open_pool(command_handle: CommandHandle, cb: extern fn(xcommand_handle: CommandHandle, err: u32)) -> u32 {
    info!("vcx_open_pool >>>");
    if is_pool_open() {
        error!("vcx_open_pool :: Pool connection is already open.");
        return VcxError::from_msg(VcxErrorKind::AlreadyInitialized, "Pool connection is already open.").into();
    }
    let path = match settings::get_config_value(settings::CONFIG_GENESIS_PATH) {
        Ok(result) => result,
        Err(_) => {
            error!("vcx_open_pool :: Failed to init pool because CONFIG_GENESIS_PATH was not set");
            return error::INVALID_CONFIGURATION.code_num;
        }
    };
    let pool_name = settings::get_config_value(settings::CONFIG_POOL_NAME).unwrap_or(settings::DEFAULT_POOL_NAME.to_string());
    let pool_config = settings::get_config_value(settings::CONFIG_POOL_CONFIG).ok();

    spawn(move || {
        match open_pool(&pool_name, &path, pool_config.as_ref().map(String::as_str)) {
            Ok(()) => {
                info!("vcx_open_pool :: Vcx Pool Init Successful");
                cb(command_handle, error::SUCCESS.code_num)
            }
            Err(e) => {
                error!("vcx_open_pool :: Vcx Pool Init Error {}.", e);
                cb(command_handle, e.into());
                return Ok(());
            }
        }
        Ok(())
    });
    error::SUCCESS.code_num
}

/// Opens wallet based on vcx configuration previously set via vcx_init_core
///
/// #Params
/// command_handle: command handle to map callback to user context.
///
/// cb: Callback that provides error status of initialization
///
/// #Returns
/// Error code as a u32
#[no_mangle]
pub extern fn vcx_open_wallet(command_handle: CommandHandle, cb: extern fn(xcommand_handle: CommandHandle, err: u32)) -> u32 {
    info!("vcx_open_wallet >>>");
    if get_wallet_handle() != INVALID_WALLET_HANDLE {
        error!("vcx_open_wallet :: Wallet was already initialized.");
        return VcxError::from_msg(VcxErrorKind::AlreadyInitialized, "Wallet was already initialized").into();
    }
    let wallet_name = match settings::get_config_value(settings::CONFIG_WALLET_NAME) {
        Ok(x) => x,
        Err(_) => {
            error!("vcx_open_wallet :: Value of setting {} was not set.", settings::CONFIG_WALLET_NAME);
            return error::INVALID_CONFIGURATION.code_num;
        }
    };

    let wallet_key = match settings::get_config_value(settings::CONFIG_WALLET_KEY) {
        Ok(wallet_key) => wallet_key,
        Err(_) => {
            error!("vcx_open_wallet :: Value of setting {} was not set.", settings::CONFIG_WALLET_KEY);
            return error::MISSING_WALLET_KEY.code_num;
        }
    };
    let wallet_kdf = settings::get_config_value(settings::CONFIG_WALLET_KEY_DERIVATION).unwrap_or(settings::WALLET_KDF_DEFAULT.into());
    let wallet_type = settings::get_config_value(settings::CONFIG_WALLET_TYPE).ok();
    let storage_config = settings::get_config_value(settings::CONFIG_WALLET_STORAGE_CONFIG).ok();
    let storage_creds = settings::get_config_value(settings::CONFIG_WALLET_STORAGE_CREDS).ok();

    spawn(move || {
        if settings::indy_mocks_enabled() {
            set_wallet_handle(WalletHandle(1));
            info!("vcx_open_wallet :: Mocked Success");
            cb(command_handle, error::SUCCESS.code_num)
        } else {
            match open_as_main_wallet(&wallet_name,
                                      &wallet_key,
                                      &wallet_kdf,
                                      wallet_type.as_ref().map(String::as_str),
                                      storage_config.as_ref().map(String::as_str),
                                      storage_creds.as_ref().map(String::as_str),
            ) {
                Ok(_) => {
                    info!("vcx_open_wallet :: Success");
                    cb(command_handle, error::SUCCESS.code_num)
                }
                Err(e) => {
                    error!("vcx_open_wallet :: Error {}.", e);
                    cb(command_handle, e.into());
                    return Ok(());
                }
            }
        }
        Ok(())
    });
    error::SUCCESS.code_num
}

lazy_static! {
    pub static ref VERSION_STRING: CString = CString::new(format!("{}{}", version_constants::VERSION, version_constants::REVISION)).unwrap();
}

#[no_mangle]
pub extern fn vcx_version() -> *const c_char {
    info!("vcx_version >>>");
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
pub extern fn vcx_shutdown(delete: bool) -> u32 {
    info!("vcx_shutdown >>>");
    trace!("vcx_shutdown(delete: {})", delete);

    match wallet::close_main_wallet() {
        Ok(()) => {}
        Err(_) => {}
    };

    match pool::close() {
        Ok(()) => {}
        Err(_) => {}
    };

    crate::schema::release_all();
    crate::connection::release_all();
    crate::issuer_credential::release_all();
    crate::credential_def::release_all();
    crate::proof::release_all();
    crate::disclosed_proof::release_all();
    crate::credential::release_all();

    if delete {
        let pool_name = settings::get_config_value(settings::CONFIG_POOL_NAME)
            .unwrap_or(settings::DEFAULT_POOL_NAME.to_string());
        let wallet_name = settings::get_config_value(settings::CONFIG_WALLET_NAME)
            .unwrap_or(settings::DEFAULT_WALLET_NAME.to_string());
        let wallet_type = settings::get_config_value(settings::CONFIG_WALLET_TYPE).ok();
        let wallet_key = settings::get_config_value(settings::CONFIG_WALLET_KEY)
            .unwrap_or(settings::UNINITIALIZED_WALLET_KEY.into());
        let key_derivation = settings::get_config_value(settings::CONFIG_WALLET_KEY_DERIVATION)
            .unwrap_or(settings::WALLET_KDF_DEFAULT.into());

        let _res = close_main_wallet();
        match wallet::delete_wallet(&wallet_name, &wallet_key, &key_derivation, wallet_type.as_deref(), None, None) {
            Ok(()) => (),
            Err(_) => (),
        };

        match pool::delete(&pool_name) {
            Ok(()) => (),
            Err(_) => (),
        };
    }

    settings::clear_config();
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
pub extern fn vcx_error_c_message(error_code: u32) -> *const c_char {
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
pub extern fn vcx_update_webhook_url(command_handle: CommandHandle,
                                     notification_webhook_url: *const c_char,
                                     cb: Option<extern fn(xcommand_handle: CommandHandle, err: u32)>) -> u32 {
    info!("vcx_update_webhook {:?} >>>", notification_webhook_url);

    check_useful_c_str!(notification_webhook_url, VcxErrorKind::InvalidOption);
    check_useful_c_callback!(cb, VcxErrorKind::InvalidOption);

    trace!("vcx_update_webhook(webhook_url: {})", notification_webhook_url);

    settings::set_config_value(settings::CONFIG_WEBHOOK_URL, &notification_webhook_url);

    spawn(move || {
        match agency_client::utils::agent_utils::update_agent_webhook(&notification_webhook_url[..]) {
            Ok(()) => {
                trace!("vcx_update_webhook_url_cb(command_handle: {}, rc: {})",
                       command_handle, error::SUCCESS.message);

                cb(command_handle, error::SUCCESS.code_num);
            }
            Err(err) => {
                warn!("vcx_update_webhook_url_cb(command_handle: {}, rc: {})",
                      command_handle, err);

                cb(command_handle, err.into());
            }
        };

        Ok(())
    });

    error::SUCCESS.code_num
}

/// Retrieve author agreement and acceptance mechanisms set on the Ledger
///
/// #params
///
/// command_handle: command handle to map callback to user context.
///
/// cb: Callback that provides array of matching messages retrieved
///
/// # Example author_agreement -> "{"text":"Default agreement", "version":"1.0.0", "aml": {"label1": "description"}}"
///
/// #Returns
/// Error code as a u32
#[no_mangle]
pub extern fn vcx_get_ledger_author_agreement(command_handle: CommandHandle,
                                              cb: Option<extern fn(xcommand_handle: CommandHandle, err: u32, author_agreement: *const c_char)>) -> u32 {
    info!("vcx_get_ledger_author_agreement >>>");

    check_useful_c_callback!(cb, VcxErrorKind::InvalidOption);

    trace!("vcx_get_ledger_author_agreement(command_handle: {})",
           command_handle);

    spawn(move || {
        match ledger::libindy_get_txn_author_agreement() {
            Ok(x) => {
                trace!("vcx_ledger_get_fees_cb(command_handle: {}, rc: {}, author_agreement: {})",
                       command_handle, error::SUCCESS.message, x);

                let msg = CStringUtils::string_to_cstring(x);
                cb(command_handle, error::SUCCESS.code_num, msg.as_ptr());
            }
            Err(e) => {
                error!("vcx_get_ledger_author_agreement(command_handle: {}, rc: {})",
                       command_handle, e);
                cb(command_handle, e.into(), std::ptr::null_mut());
            }
        };

        Ok(())
    });

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
pub extern fn vcx_set_active_txn_author_agreement_meta(text: *const c_char,
                                                       version: *const c_char,
                                                       hash: *const c_char,
                                                       acc_mech_type: *const c_char,
                                                       time_of_acceptance: u64) -> u32 {
    info!("vcx_set_active_txn_author_agreement_meta >>>");

    check_useful_opt_c_str!(text, VcxErrorKind::InvalidOption);
    check_useful_opt_c_str!(version, VcxErrorKind::InvalidOption);
    check_useful_opt_c_str!(hash, VcxErrorKind::InvalidOption);
    check_useful_c_str!(acc_mech_type, VcxErrorKind::InvalidOption);

    trace!("vcx_set_active_txn_author_agreement_meta(text: {:?}, version: {:?}, hash: {:?}, acc_mech_type: {:?}, time_of_acceptance: {:?})",
           text, version, hash, acc_mech_type, time_of_acceptance);

    match utils::author_agreement::set_txn_author_agreement(text, version, hash, acc_mech_type, time_of_acceptance) {
        Ok(()) => error::SUCCESS.code_num,
        Err(err) => err.into()
    }
}

#[no_mangle]
pub extern fn vcx_mint_tokens(seed: *const c_char, fees: *const c_char) {
    info!("vcx_mint_tokens >>>");

    // TODO: CHEC
    let seed = if !seed.is_null() {
        match CStringUtils::c_str_to_string(seed) {
            Ok(opt_val) => opt_val.map(String::from),
            Err(_) => return ()
        }
    } else {
        None
    };

    let fees = if !fees.is_null() {
        match CStringUtils::c_str_to_string(fees) {
            Ok(opt_val) => opt_val.map(String::from),
            Err(_) => return ()
        }
    } else {
        None
    };
    trace!("vcx_mint_tokens(seed: {:?}, fees: {:?})", seed, fees);

    libindy::utils::payments::mint_tokens_and_set_fees(None, None, fees, seed).unwrap_or_default();
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
pub extern fn vcx_get_current_error(error_json_p: *mut *const c_char) {
    trace!("vcx_get_current_error >>> error_json_p: {:?}", error_json_p);

    let error = get_current_error_c_json();
    unsafe { *error_json_p = error };

    trace!("vcx_get_current_error: <<<");
}

#[cfg(test)]
mod tests {
    use std::ptr;

    use crate::{api, connection, credential, credential_def, disclosed_proof, issuer_credential, proof, schema};
    use crate::api::return_types_u32;
    use crate::api::wallet::tests::_test_add_and_get_wallet_record;
    use crate::libindy::utils::pool::get_pool_handle;
    use crate::libindy::utils::pool::tests::create_tmp_genesis_txn_file;
    #[cfg(feature = "pool_tests")]
    use crate::libindy::utils::pool::tests::delete_test_pool;
    use crate::libindy::utils::wallet::import;
    #[cfg(feature = "pool_tests")]
    use crate::libindy::utils::wallet::get_wallet_handle;
    use crate::libindy::utils::wallet::tests::create_main_wallet_and_its_backup;
    use crate::utils::devsetup::*;
    #[cfg(any(feature = "agency", feature = "pool_tests"))]
    use crate::utils::get_temp_dir_path;
    use crate::utils::timeout::TimeoutUtils;

    use super::*;

    fn _vcx_open_pool_c_closure() -> Result<(), u32> {
        let cb = return_types_u32::Return_U32::new().unwrap();

        let rc = vcx_open_pool(cb.command_handle, cb.get_callback());
        if rc != error::SUCCESS.code_num {
            return Err(rc);
        }
        cb.receive(TimeoutUtils::some_medium())
    }

    fn _vcx_open_wallet_c_closure() -> Result<(), u32> {
        let cb = return_types_u32::Return_U32::new().unwrap();

        let rc = vcx_open_wallet(cb.command_handle, cb.get_callback());
        if rc != error::SUCCESS.code_num {
            return Err(rc);
        }
        cb.receive(TimeoutUtils::some_medium())
    }

    fn _vcx_init_core_c_closure(config: &str) -> Result<(), u32> {
        let rc = vcx_init_core(CString::new(config).unwrap().into_raw());
        if rc != error::SUCCESS.code_num {
            return Err(rc);
        }
        Ok(())
    }

    fn _vcx_init_full(config: &str) -> Result<(), u32> {
        info!("_vcx_init_full >>>");
        let rc = vcx_init_core(CString::new(config).unwrap().into_raw());
        if rc != error::SUCCESS.code_num {
            error!("vcx_init_core failed");
            return Err(rc);
        }
        settings::get_agency_client()?.enable_test_mode();

        info!("_vcx_init_full >>> going to open pool");
        let cb = return_types_u32::Return_U32::new().unwrap();
        let rc = vcx_open_pool(cb.command_handle, cb.get_callback());
        if rc != error::SUCCESS.code_num {
            error!("vcx_open_pool failed");
            return Err(rc);
        }
        cb.receive(TimeoutUtils::some_short()).unwrap();

        info!("_vcx_init_full >>> going to open wallet");
        let cb = return_types_u32::Return_U32::new().unwrap();
        let rc = vcx_open_wallet(cb.command_handle, cb.get_callback());
        if rc != error::SUCCESS.code_num {
            error!("vcx_open_wallet failed");
            return Err(rc);
        }
        cb.receive(TimeoutUtils::some_custom(3)).unwrap();
        Ok(())
    }

    #[cfg(any(feature = "agency", feature = "pool_tests"))]
    fn config() -> String {
        json!({"agency_did" : "72x8p4HubxzUK1dwxcc5FU",
               "remote_to_sdk_did" : "UJGjM6Cea2YVixjWwHN9wq",
               "sdk_to_remote_did" : "AB3JM851T4EQmhh8CdagSP",
               "sdk_to_remote_verkey" : "888MFrZjXDoi2Vc8Mm14Ys112tEZdDegBZZoembFEATE",
               "institution_name" : "evernym enterprise",
               "agency_verkey" : "91qMFrZjXDoi2Vc8Mm14Ys112tEZdDegBZZoembFEATE",
               "remote_to_sdk_verkey" : "91qMFrZjXDoi2Vc8Mm14Ys112tEZdDegBZZoembFEATE",
               "genesis_path": get_temp_dir_path("pool1.txn").to_str().unwrap(),
               "payment_method": "null",
               "pool_config": json!({"timeout":60}).to_string()
           }).to_string()
    }

    #[cfg(feature = "pool_tests")]
    #[test]
    fn test_open_pool_fails_if_genesis_file_is_invalid() {
        let _setup = SetupWallet::init();

        // Write invalid genesis.txn
        let _genesis_transactions = TempFile::create_with_data(utils::constants::GENESIS_PATH, "{}");
        settings::set_config_value(settings::CONFIG_GENESIS_PATH, &_genesis_transactions.path);

        let err = _vcx_open_pool_c_closure().unwrap_err();
        assert_eq!(err, error::POOL_LEDGER_CONNECT.code_num);

        assert_eq!(get_pool_handle().unwrap_err().kind(), VcxErrorKind::NoPoolOpen);
        assert_eq!(get_wallet_handle(), INVALID_WALLET_HANDLE);

        delete_test_pool();
    }

    #[test]
    #[cfg(feature = "general_test")]
    fn test_init_core_fails_with_no_wallet_key() {
        let _setup = SetupEmpty::init();

        let content = json!({
            "wallet_name": settings::DEFAULT_WALLET_NAME,
        }).to_string();

        let err = init_core(&content).unwrap_err();
        assert_eq!(err.kind(), VcxErrorKind::MissingWalletKey);
        let rc = _vcx_open_wallet_c_closure().unwrap_err();
        assert_eq!(rc, error::MISSING_WALLET_KEY.code_num);
    }

    #[cfg(feature = "pool_tests")]
    #[test]
    fn test_vcx_init_with_default_values() {
        let _setup_defaults = SetupDefaults::init();
        let _setup_wallet = SetupWallet::init();
        let _setup_pool = SetupPoolConfig::init();

        _vcx_init_full("{}").unwrap();
    }

    #[cfg(feature = "pool_tests")]
    #[test]
    fn test_vcx_init_called_twice_fails() {
        let _setup_defaults = SetupDefaults::init();
        let _setup_wallet = SetupWallet::init();
        let _setup_pool = SetupPoolConfig::init();

        _vcx_init_full("{}").unwrap();

        // Repeat call
        let rc = _vcx_init_full("{}").unwrap_err();
        assert_eq!(rc, error::ALREADY_INITIALIZED.code_num);
    }

    #[cfg(feature = "pool_tests")]
    #[test]
    fn test_vcx_init_called_twice_passes_after_shutdown() {
        for _ in 0..2 {
            let _setup_defaults = SetupDefaults::init();
            let _setup_wallet = SetupWallet::init().skip_cleanup();
            let _setup_pool = SetupPoolConfig::init().skip_cleanup();

            _vcx_init_full("{}").unwrap();

            //Assert config values were set correctly
            assert_eq!(settings::get_config_value("wallet_name").unwrap(), _setup_wallet.wallet_name);

            //Verify shutdown was successful
            vcx_shutdown(true);
            assert_eq!(settings::get_config_value("wallet_name").unwrap_err().kind(), VcxErrorKind::InvalidConfiguration);
        }
    }


    #[test]
    #[cfg(feature = "general_test")]
    fn test_open_wallet_of_imported_wallet_succeeds() {
        let _setup = SetupDefaults::init();

        let (export_wallet_path, wallet_name) = create_main_wallet_and_its_backup();

        wallet::delete_wallet(&wallet_name, settings::DEFAULT_WALLET_KEY, settings::WALLET_KDF_RAW, None, None, None).unwrap();

        let import_config = json!({
            settings::CONFIG_WALLET_NAME: &wallet_name,
            settings::CONFIG_WALLET_KEY: settings::DEFAULT_WALLET_KEY,
            settings::CONFIG_WALLET_KEY_DERIVATION: settings::WALLET_KDF_RAW,
            settings::CONFIG_WALLET_BACKUP_KEY: settings::DEFAULT_WALLET_BACKUP_KEY,
            settings::CONFIG_EXPORTED_WALLET_PATH: export_wallet_path.path,
        }).to_string();
        import(&import_config).unwrap();

        let content = json!({
            "wallet_name": &wallet_name,
            "wallet_key": settings::DEFAULT_WALLET_KEY,
            "wallet_key_derivation": settings::WALLET_KDF_RAW,
        }).to_string();

        _vcx_init_core_c_closure(&content).unwrap();
        _vcx_open_wallet_c_closure().unwrap();

        vcx_shutdown(true);
    }

    #[test]
    #[cfg(feature = "general_test")]
    fn test_open_wallet_with_wrong_name_fails() {
        let _setup = SetupDefaults::init();

        let (export_wallet_path, wallet_name) = create_main_wallet_and_its_backup();

        wallet::delete_wallet(&wallet_name, settings::DEFAULT_WALLET_KEY, settings::WALLET_KDF_RAW, None, None, None).unwrap();

        let import_config = json!({
            settings::CONFIG_WALLET_NAME: wallet_name.as_str(),
            settings::CONFIG_WALLET_KEY: settings::DEFAULT_WALLET_KEY,
            settings::CONFIG_WALLET_KEY_DERIVATION: settings::WALLET_KDF_RAW,
            settings::CONFIG_EXPORTED_WALLET_PATH: export_wallet_path.path,
            settings::CONFIG_WALLET_BACKUP_KEY: settings::DEFAULT_WALLET_BACKUP_KEY,
        }).to_string();
        import(&import_config).unwrap();

        let content = json!({
            "wallet_name": "different_wallet_name",
            "wallet_key": settings::DEFAULT_WALLET_KEY,
            "wallet_key_derivation": settings::WALLET_KDF_RAW,
        }).to_string();


        _vcx_init_core_c_closure(&content).unwrap();
        let err = _vcx_open_wallet_c_closure().unwrap_err();
        assert_eq!(err, error::WALLET_NOT_FOUND.code_num);

        wallet::delete_wallet(wallet_name.as_str(), settings::DEFAULT_WALLET_KEY, settings::WALLET_KDF_RAW, None, None, None).unwrap();
    }

    #[test]
    #[cfg(feature = "general_test")]
    fn test_import_of_opened_wallet_fails() {
        let _setup = SetupDefaults::init();

        let (export_wallet_path, wallet_name) = create_main_wallet_and_its_backup();

        let content = json!({
            settings::CONFIG_WALLET_NAME: wallet_name.as_str(),
            "wallet_key": settings::DEFAULT_WALLET_KEY,
            "wallet_key_derivation": settings::WALLET_KDF_RAW
        }).to_string();


        _vcx_init_core_c_closure(&content).unwrap();
        _vcx_open_wallet_c_closure().unwrap();

        let import_config = json!({
            settings::CONFIG_WALLET_NAME: wallet_name.as_str(),
            settings::CONFIG_WALLET_KEY: settings::DEFAULT_WALLET_KEY,
            settings::CONFIG_EXPORTED_WALLET_PATH: export_wallet_path.path,
            settings::CONFIG_WALLET_BACKUP_KEY: settings::DEFAULT_WALLET_BACKUP_KEY,
        }).to_string();
        assert_eq!(import(&import_config).unwrap_err().kind(), VcxErrorKind::DuplicationWallet);

        vcx_shutdown(true);
    }


    #[test]
    #[cfg(feature = "general_test")]
    fn test_init_no_config_path() {
        let _setup = SetupEmpty::init();
        assert_eq!(vcx_init_core(ptr::null()), error::INVALID_OPTION.code_num)
    }

    #[test]
    #[cfg(feature = "general_test")]
    fn test_shutdown_with_no_previous_config() {
        let _setup = SetupDefaults::init();

        vcx_shutdown(true);
        vcx_shutdown(false);
    }

    #[test]
    #[cfg(feature = "general_test")]
    fn test_shutdown() {
        let _setup = SetupMocks::init();

        let data = r#"["name","male"]"#;
        let connection = connection::tests::build_test_connection_inviter_invited();
        let credentialdef = credential_def::create_and_publish_credentialdef("SID".to_string(), "NAME".to_string(), "4fUDR9R7fjwELRvH9JT6HH".to_string(), "id".to_string(), "tag".to_string(), "{}".to_string()).unwrap();
        let issuer_credential = issuer_credential::issuer_credential_create(credentialdef, "1".to_string(), "8XFh8yBzrpJQmNyZzgoTqB".to_owned(), "credential_name".to_string(), "{\"attr\":\"value\"}".to_owned(), 1).unwrap();
        let proof = proof::create_proof("1".to_string(), "[]".to_string(), "[]".to_string(), r#"{"support_revocation":false}"#.to_string(), "Optional".to_owned()).unwrap();
        let schema = schema::create_and_publish_schema("5", "VsKV7grR1BUE29mG2Fm2kX".to_string(), "name".to_string(), "0.1".to_string(), data.to_string()).unwrap();
        let disclosed_proof = disclosed_proof::create_proof("id", utils::mockdata::mockdata_proof::ARIES_PROOF_REQUEST_PRESENTATION).unwrap();
        let credential = credential::credential_create_with_offer("name", utils::mockdata::mockdata_credex::ARIES_CREDENTIAL_OFFER).unwrap();

        vcx_shutdown(true);
        assert_eq!(connection::release(connection).unwrap_err().kind(), VcxErrorKind::InvalidConnectionHandle);
        assert_eq!(issuer_credential::release(issuer_credential).unwrap_err().kind(), VcxErrorKind::InvalidIssuerCredentialHandle);
        assert_eq!(schema::release(schema).unwrap_err().kind(), VcxErrorKind::InvalidSchemaHandle);
        assert_eq!(proof::release(proof).unwrap_err().kind(), VcxErrorKind::InvalidProofHandle);
        assert_eq!(credential_def::release(credentialdef).unwrap_err().kind(), VcxErrorKind::InvalidCredDefHandle);
        assert_eq!(credential::release(credential).unwrap_err().kind(), VcxErrorKind::InvalidCredentialHandle);
        assert_eq!(disclosed_proof::release(disclosed_proof).unwrap_err().kind(), VcxErrorKind::InvalidDisclosedProofHandle);
        assert_eq!(wallet::get_wallet_handle(), INVALID_WALLET_HANDLE);
    }

    #[test]
    #[cfg(feature = "general_test")]
    fn test_error_c_message() {
        let _setup = SetupMocks::init();

        let c_message = CStringUtils::c_str_to_string(vcx_error_c_message(0)).unwrap().unwrap();
        assert_eq!(c_message, error::SUCCESS.message);

        let c_message = CStringUtils::c_str_to_string(vcx_error_c_message(1001)).unwrap().unwrap();
        assert_eq!(c_message, error::UNKNOWN_ERROR.message);

        let c_message = CStringUtils::c_str_to_string(vcx_error_c_message(100100)).unwrap().unwrap();
        assert_eq!(c_message, error::UNKNOWN_ERROR.message);

        let c_message = CStringUtils::c_str_to_string(vcx_error_c_message(1021)).unwrap().unwrap();
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

        let webhook_url = "http://www.evernym.com";
        assert_ne!(webhook_url, &settings::get_config_value(settings::CONFIG_WEBHOOK_URL).unwrap());

        let cb = return_types_u32::Return_U32::new().unwrap();
        assert_eq!(error::SUCCESS.code_num, vcx_update_webhook_url(cb.command_handle,
                                                                   CString::new(webhook_url.to_string()).unwrap().into_raw(),
                                                                   Some(cb.get_callback())));

        assert_eq!(webhook_url, &settings::get_config_value(settings::CONFIG_WEBHOOK_URL).unwrap());
    }

    #[test]
    #[cfg(feature = "general_test")]
    fn get_current_error_works_for_no_error() {
        let _setup = SetupDefaults::init();

        crate::error::reset_current_error();

        let mut error_json_p: *const c_char = ptr::null();

        vcx_get_current_error(&mut error_json_p);
        assert_eq!(None, CStringUtils::c_str_to_string(error_json_p).unwrap());
    }

    #[test]
    #[cfg(feature = "general_test")]
    fn get_current_error_works_for_sync_error() {
        let _setup = SetupDefaults::init();

        api::utils::vcx_provision_agent(ptr::null());

        let mut error_json_p: *const c_char = ptr::null();
        vcx_get_current_error(&mut error_json_p);
        assert!(CStringUtils::c_str_to_string(error_json_p).unwrap().is_some());
    }

    #[test]
    #[cfg(feature = "general_test")]
    fn get_current_error_works_for_async_error() {
        let _setup = SetupDefaults::init();

        extern fn cb(_storage_handle: i32,
                     _err: u32,
                     _config: *const c_char) {
            let mut error_json_p: *const c_char = ptr::null();
            vcx_get_current_error(&mut error_json_p);
            assert!(CStringUtils::c_str_to_string(error_json_p).unwrap().is_some());
        }

        let config = CString::new("{}").unwrap();
        api::utils::vcx_agent_provision_async(0, config.as_ptr(), Some(cb));
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

        assert_eq!(error::SUCCESS.code_num, vcx_set_active_txn_author_agreement_meta(CString::new(text.to_string()).unwrap().into_raw(),
                                                                                     CString::new(version.to_string()).unwrap().into_raw(),
                                                                                     std::ptr::null(),
                                                                                     CString::new(acc_mech_type.to_string()).unwrap().into_raw(),
                                                                                     time_of_acceptance));

        let expected = json!({
            "text": text,
            "version": version,
            "acceptanceMechanismType": acc_mech_type,
            "timeOfAcceptance": time_of_acceptance,
        });

        let auth_agreement = settings::get_config_value(settings::CONFIG_TXN_AUTHOR_AGREEMENT).unwrap();
        let auth_agreement = serde_json::from_str::<::serde_json::Value>(&auth_agreement).unwrap();

        assert_eq!(expected, auth_agreement);

        settings::set_testing_defaults();
    }

    #[test]
    #[cfg(feature = "general_test")]
    fn test_vcx_get_ledger_author_agreement() {
        let _setup = SetupMocks::init();

        let cb = return_types_u32::Return_U32_STR::new().unwrap();
        assert_eq!(vcx_get_ledger_author_agreement(cb.command_handle,
                                                   Some(cb.get_callback())), error::SUCCESS.code_num);
        let agreement = cb.receive(TimeoutUtils::some_short()).unwrap();
        assert_eq!(::utils::constants::DEFAULT_AUTHOR_AGREEMENT, agreement.unwrap());
    }

    #[cfg(feature = "general_test")]
    fn get_settings() -> String {
        json!({
            settings::CONFIG_INSTITUTION_NAME:            settings::get_config_value(settings::CONFIG_INSTITUTION_NAME).unwrap(),
            settings::CONFIG_INSTITUTION_DID:             settings::get_config_value(settings::CONFIG_INSTITUTION_DID).unwrap(),
            settings::CONFIG_PAYMENT_METHOD:              settings::get_config_value(settings::CONFIG_PAYMENT_METHOD).unwrap()
        }).to_string()
    }

    #[test]
    #[cfg(feature = "general_test")]
    fn test_init_core() {
        let _setup = SetupEmpty::init();

        let config = json!({
          "agency_did": "VsKV7grR1BUE29mG2Fm2kX",
          "agency_endpoint": "http://localhost:8080",
          "agency_verkey": "Hezce2UWMZ3wUhVkh2LfKSs8nDzWwzs2Win7EzNN3YaR",
          "genesis_path": "/tmp/foo/bar",
          "institution_did": "V4SGRU86Z58d6TV7PBUe6f",
          "institution_name": "alice-9b2e793a-2e89-42c0-8941-dd3360bb2043",
          "institution_verkey": "GJ1SzoWzavQYfNL9XkaJdrQejfztN4XqdsiV4ct3LXKL",
          "remote_to_sdk_did": "L8U9Ae48mLGxx3drppU8Ph",
          "remote_to_sdk_verkey": "BRhUCTk6KFgUk9cnnL9ozfjtvEwXnSPRfUduzjpMaZca",
          "sdk_to_remote_did": "6Ke2y7C9WVSwDa4PieDtc9",
          "sdk_to_remote_verkey": "3uDfyP3As6aMQSjYdd95y3UNVkpn2wqTZ6MHrJcCCSFc",
          "wallet_key": "1234567",
          "wallet_name": "alice"
        });
        let cstring_config = CString::new(config.to_string()).unwrap().into_raw();
        assert_eq!(vcx_init_core(cstring_config), error::SUCCESS.code_num);
    }

    #[test]
    #[cfg(feature = "pool_tests")]
    fn test_open_pool() {
        let _setup = SetupEmpty::init();

        let genesis_path = create_tmp_genesis_txn_file();
        settings::set_config_value(settings::CONFIG_GENESIS_PATH, &genesis_path);

        let cb = return_types_u32::Return_U32::new().unwrap();
        let rc = vcx_open_pool(cb.command_handle, cb.get_callback());
        assert_eq!(rc, error::SUCCESS.code_num);
        cb.receive(TimeoutUtils::some_short()).unwrap();

        delete_test_pool();
        settings::set_testing_defaults();
    }

    #[test]
    #[cfg(feature = "general_test")]
    fn test_open_wallet() {
        let _setup = SetupWallet::init();

        let cb = return_types_u32::Return_U32::new().unwrap();
        let rc = vcx_open_wallet(cb.command_handle, cb.get_callback());
        assert_eq!(rc, error::SUCCESS.code_num);
        cb.receive(TimeoutUtils::some_custom(3)).unwrap();

        _test_add_and_get_wallet_record();

        settings::set_testing_defaults();
    }

    #[cfg(feature = "pool_tests")]
    #[test]
    fn test_full_init() {
        let _setup_defaults = SetupDefaults::init();
        let _setup_wallet = SetupWallet::init();
        let _setup_pool = SetupPoolConfig::init();

        _vcx_init_full(&config()).unwrap();

        // Assert pool was initialized
        assert_ne!(get_pool_handle().unwrap(), 0);
    }

    #[test]
    #[cfg(feature = "pool_tests")]
    fn test_init_composed() {
        let _setup = SetupEmpty::init();
        let _setup_wallet = SetupWallet::init();

        let wallet_name = settings::get_config_value(settings::CONFIG_WALLET_NAME).unwrap();
        let wallet_key = settings::get_config_value(settings::CONFIG_WALLET_KEY).unwrap();
        let wallet_kdf = settings::get_config_value(settings::CONFIG_WALLET_KEY_DERIVATION).unwrap();
        let genesis_path = create_tmp_genesis_txn_file();

        let config = json!({
          "agency_did": "VsKV7grR1BUE29mG2Fm2kX",
          "agency_endpoint": "http://localhost:8080",
          "agency_verkey": "Hezce2UWMZ3wUhVkh2LfKSs8nDzWwzs2Win7EzNN3YaR",
          "genesis_path": genesis_path,
          "institution_did": "V4SGRU86Z58d6TV7PBUe6f",
          "institution_name": "alice-9b2e793a-2e89-42c0-8941-dd3360bb2043",
          "institution_verkey": "GJ1SzoWzavQYfNL9XkaJdrQejfztN4XqdsiV4ct3LXKL",
          "remote_to_sdk_did": "L8U9Ae48mLGxx3drppU8Ph",
          "remote_to_sdk_verkey": "BRhUCTk6KFgUk9cnnL9ozfjtvEwXnSPRfUduzjpMaZca",
          "sdk_to_remote_did": "6Ke2y7C9WVSwDa4PieDtc9",
          "sdk_to_remote_verkey": "3uDfyP3As6aMQSjYdd95y3UNVkpn2wqTZ6MHrJcCCSFc",
          "wallet_key": wallet_key,
          "wallet_key_derivation": wallet_kdf,
          "wallet_name": wallet_name,
          "protocol_version": "2"
        });

        _vcx_init_full(&config.to_string()).unwrap();
        configure_trustee_did();
        setup_libnullpay_nofees();

        info!("test_init_composed :: creating schema + creddef to verify wallet and pool connectivity");
        let attrs_list = json!(["address1", "address2", "city", "state", "zip"]).to_string();
        let (schema_id, _schema_json, _cred_def_id, _cred_def_json, _cred_def_handle, _rev_reg_id) =
            libindy::utils::anoncreds::tests::create_and_store_credential_def(&attrs_list, true);
        assert!(schema_id.len() > 0);

        delete_test_pool();
        settings::set_testing_defaults();
    }

    #[cfg(feature = "pool_tests")]
    #[test]
    fn test_agency_client_does_not_have_to_be_initialized() {
        let _setup = SetupLibraryWalletPool::init();

        let config = json!({
            "institution_name": "faber",
            "institution_did": "44x8p4HubxzUK1dwxcc5FU",
            "institution_verkey": "444MFrZjXDoi2Vc8Mm14Ys112tEZdDegBZZoembFEATE"
        }).to_string();

        api::wallet::vcx_wallet_set_handle(get_wallet_handle());
        api::utils::vcx_pool_set_handle(get_pool_handle().unwrap());

        settings::clear_config();

        assert_eq!(vcx_init_core(CString::new(config).unwrap().into_raw()), error::SUCCESS.code_num);

        let connection_handle = connection::create_connection("test_create_fails").unwrap();
        connection::connect(connection_handle).unwrap_err();

        settings::set_testing_defaults();
    }

    #[cfg(feature = "pool_tests")]
    #[test]
    fn test_open_pool_fails_if_genesis_path_is_invalid() {
        let _setup = SetupWallet::init();

        let content = json!({
            "genesis_path": "invalid/txn/path"
        }).to_string();

        init_core(&content).unwrap();
        let rc = _vcx_open_pool_c_closure().unwrap_err();
        assert_eq!(rc, error::INVALID_GENESIS_TXN_PATH.code_num);
    }
}
