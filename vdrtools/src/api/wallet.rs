use indy_api_types::{
    domain::wallet::{Config, Credentials, ExportConfig, KeyConfig},
    errors::prelude::*,
    validation::Validatable,
    wallet::*,
    CommandHandle, ErrorCode, WalletHandle, INVALID_WALLET_HANDLE,
};

use indy_utils::ctypes;
use libc::c_char;
use serde_json;

use crate::Locator;
use crate::services::CommandMetric;

/// Register custom wallet storage implementation.
///
/// #Params
/// command_handle: Command handle to map callback to caller context.
/// type_: Storage type name.
/// create: WalletType create operation handler
/// open: WalletType open operation handler
/// close: Wallet close operation handler
/// delete: WalletType delete operation handler
/// add_record: WalletType add record operation handler
/// update_record_value: WalletType update record value operation handler
/// update_record_tags: WalletType update record tags operation handler
/// add_record_tags: WalletType add record tags operation handler
/// delete_record_tags: WalletType delete record tags operation handler
/// delete_record: WalletType delete record operation handler
/// get_record: WalletType get record operation handler
/// get_record_id: WalletType get record id operation handler
/// get_record_type: WalletType get record type operation handler
/// get_record_value: WalletType get record value operation handler
/// get_record_tags: WalletType get record tags operation handler
/// free_record: WalletType free record operation handler
/// search_records: WalletType search records operation handler
/// search_all_records: WalletType search all records operation handler
/// get_search_total_count: WalletType get search total count operation handler
/// fetch_search_next_record: WalletType fetch search next record operation handler
/// free_search: WalletType free search operation handler
/// free: Handler that allows to de-allocate strings allocated in caller code
///
/// #Returns
/// Error code
#[no_mangle]
pub extern "C" fn indy_register_wallet_storage(
    command_handle: CommandHandle,
    type_: *const c_char,
    create: Option<WalletCreate>,
    open: Option<WalletOpen>,
    close: Option<WalletClose>,
    delete: Option<WalletDelete>,
    add_record: Option<WalletAddRecord>,
    update_record_value: Option<WalletUpdateRecordValue>,
    update_record_tags: Option<WalletUpdateRecordTags>,
    add_record_tags: Option<WalletAddRecordTags>,
    delete_record_tags: Option<WalletDeleteRecordTags>,
    delete_record: Option<WalletDeleteRecord>,
    get_record: Option<WalletGetRecord>,
    get_record_id: Option<WalletGetRecordId>,
    get_record_type: Option<WalletGetRecordType>,
    get_record_value: Option<WalletGetRecordValue>,
    get_record_tags: Option<WalletGetRecordTags>,
    free_record: Option<WalletFreeRecord>,
    get_storage_metadata: Option<WalletGetStorageMetadata>,
    set_storage_metadata: Option<WalletSetStorageMetadata>,
    free_storage_metadata: Option<WalletFreeStorageMetadata>,
    search_records: Option<WalletSearchRecords>,
    search_all_records: Option<WalletSearchAllRecords>,
    get_search_total_count: Option<WalletGetSearchTotalCount>,
    fetch_search_next_record: Option<WalletFetchSearchNextRecord>,
    free_search: Option<WalletFreeSearch>,
    cb: Option<extern "C" fn(command_handle_: CommandHandle, err: ErrorCode)>,
) -> ErrorCode {
    debug!("indy_register_wallet_type: > type_ {:?}", type_); // TODO: Log all params

    check_useful_c_str!(type_, ErrorCode::CommonInvalidParam2);
    check_useful_c_callback!(create, ErrorCode::CommonInvalidParam3);
    check_useful_c_callback!(open, ErrorCode::CommonInvalidParam4);
    check_useful_c_callback!(close, ErrorCode::CommonInvalidParam5);
    check_useful_c_callback!(delete, ErrorCode::CommonInvalidParam6);
    check_useful_c_callback!(add_record, ErrorCode::CommonInvalidParam7);
    check_useful_c_callback!(update_record_value, ErrorCode::CommonInvalidParam8);
    check_useful_c_callback!(update_record_tags, ErrorCode::CommonInvalidParam9);
    check_useful_c_callback!(add_record_tags, ErrorCode::CommonInvalidParam10);
    check_useful_c_callback!(delete_record_tags, ErrorCode::CommonInvalidParam11);
    check_useful_c_callback!(delete_record, ErrorCode::CommonInvalidParam12);
    check_useful_c_callback!(get_record, ErrorCode::CommonInvalidParam13);
    check_useful_c_callback!(get_record_id, ErrorCode::CommonInvalidParam14);
    check_useful_c_callback!(get_record_type, ErrorCode::CommonInvalidParam15);
    check_useful_c_callback!(get_record_value, ErrorCode::CommonInvalidParam16);
    check_useful_c_callback!(get_record_tags, ErrorCode::CommonInvalidParam17);
    check_useful_c_callback!(free_record, ErrorCode::CommonInvalidParam18);
    check_useful_c_callback!(get_storage_metadata, ErrorCode::CommonInvalidParam19);
    check_useful_c_callback!(set_storage_metadata, ErrorCode::CommonInvalidParam20);
    check_useful_c_callback!(free_storage_metadata, ErrorCode::CommonInvalidParam21);
    check_useful_c_callback!(search_records, ErrorCode::CommonInvalidParam22);
    check_useful_c_callback!(search_all_records, ErrorCode::CommonInvalidParam23);
    check_useful_c_callback!(get_search_total_count, ErrorCode::CommonInvalidParam24);
    check_useful_c_callback!(fetch_search_next_record, ErrorCode::CommonInvalidParam25);
    check_useful_c_callback!(free_search, ErrorCode::CommonInvalidParam26);
    check_useful_c_callback!(cb, ErrorCode::CommonInvalidParam27);

    debug!("indy_register_wallet_type ? type_ {:?}", type_);

    let locator = Locator::instance();

    let action = async move {
        let res = locator.wallet_controller.register_type(
            type_,
            create,
            open,
            close,
            delete,
            add_record,
            update_record_value,
            update_record_tags,
            add_record_tags,
            delete_record_tags,
            delete_record,
            get_record,
            get_record_id,
            get_record_type,
            get_record_value,
            get_record_tags,
            free_record,
            get_storage_metadata,
            set_storage_metadata,
            free_storage_metadata,
            search_records,
            search_all_records,
            get_search_total_count,
            fetch_search_next_record,
            free_search,
        );
        res
    };

    let cb = move |res: IndyResult<_>| {
        let err = prepare_result!(res);
        debug!("indy_register_wallet_type ? err {:?}", err);

        cb(command_handle, err);
    };

    locator.executor.spawn_ok_instrumented(CommandMetric::WalletCommandRegisterWalletType, action, cb);

    let res = ErrorCode::Success;
    debug!("indy_register_wallet_type < {:?}", res);
    res
}

/// Create a new secure wallet.
///
/// #Params
/// config: Wallet configuration json.
/// {
///   "id": string, Identifier of the wallet.
///         Configured storage uses this identifier to lookup exact wallet data placement.
///   "storage_type": optional<string>, Type of the wallet storage. Defaults to 'default'.
///                  'Default' storage type allows to store wallet data in the local file.
///                  Custom storage types can be registered with indy_register_wallet_storage call.
///   "storage_config": optional<object>, Storage configuration json. Storage type defines set of supported keys.
///                     Can be optional if storage supports default configuration.
///                     For 'default' storage type configuration is:
///   {
///     "path": optional<string>, Path to the directory with wallet files.
///             Defaults to $HOME/.indy_client/wallet.
///             Wallet will be stored in the file {path}/{id}/sqlite.db
///   }
/// }
/// credentials: Wallet credentials json
/// {
///   "key": string, Key or passphrase used for wallet key derivation.
///                  Look to key_derivation_method param for information about supported key derivation methods.
///   "storage_credentials": optional<object> Credentials for wallet storage. Storage type defines set of supported keys.
///                          Can be optional if storage supports default configuration.
///                          For 'default' storage type should be empty.
///   "key_derivation_method": optional<string> Algorithm to use for wallet key derivation:
///                          ARGON2I_MOD - derive secured wallet master key (used by default)
///                          ARGON2I_INT - derive secured wallet master key (less secured but faster)
///                          RAW - raw wallet key master provided (skip derivation).
///                                RAW keys can be generated with indy_generate_wallet_key call
/// }
///
/// #Returns
/// err: Error code
///
/// #Errors
/// Common*
/// Wallet*
#[no_mangle]
pub extern "C" fn indy_create_wallet(
    command_handle: CommandHandle,
    config: *const c_char,
    credentials: *const c_char,
    cb: Option<extern "C" fn(command_handle_: CommandHandle, err: ErrorCode)>,
) -> ErrorCode {
    debug!(
        "indy_create_wallet > config {:?} credentials {:?}",
        config, credentials
    );

    check_useful_validatable_json!(config, ErrorCode::CommonInvalidParam2, Config);
    check_useful_json!(credentials, ErrorCode::CommonInvalidParam3, Credentials);
    check_useful_c_callback!(cb, ErrorCode::CommonInvalidParam4);

    debug!(
        "indy_create_wallet ? config {:?} credentials {:?}",
        config,
        secret!(&credentials)
    );

    let locator = Locator::instance();

    let action = async move {
        let res = locator.wallet_controller.create(config, credentials).await;
        res
    };

    let cb = move |res: IndyResult<_>| {
        let err = prepare_result!(res);
        debug!("indy_create_wallet ? err {:?}", err);

        cb(command_handle, err);
    };

    locator.executor.spawn_ok_instrumented(CommandMetric::WalletCommandCreate, action, cb);

    let res = ErrorCode::Success;
    debug!("indy_create_wallet < {:?}", res);
    res
}

/// Open the wallet.
///
/// Wallet must be previously created with indy_create_wallet method.
///
/// #Params
/// config: Wallet configuration json.
///   {
///       "id": string, Identifier of the wallet.
///             Configured storage uses this identifier to lookup exact wallet data placement.
///       "storage_type": optional<string>, Type of the wallet storage. Defaults to 'default'.
///                       'Default' storage type allows to store wallet data in the local file.
///                       Custom storage types can be registered with indy_register_wallet_storage call.
///       "storage_config": optional<object>, Storage configuration json. Storage type defines set of supported keys.
///                         Can be optional if storage supports default configuration.
///                         For 'default' storage type configuration is:
///           {
///              "path": optional<string>, Path to the directory with wallet files.
///                      Defaults to $HOME/.indy_client/wallet.
///                      Wallet will be stored in the file {path}/{id}/sqlite.db
///           }
///       "cache": optional<object>, Cache configuration json. If omitted the cache is disabled (default).
///       {
///           "size": optional<int>, Number of items in cache,
///           "entities": List<string>, Types of items being cached. eg. ["vdrtools::Did", "vdrtools::Key"]
///           "algorithm" optional<string>, cache algorithm, defaults to lru, which is the only one supported for now.
///       }
///   }
/// credentials: Wallet credentials json
///   {
///       "key": string, Key or passphrase used for wallet key derivation.
///                      Look to key_derivation_method param for information about supported key derivation methods.
///       "rekey": optional<string>, If present than wallet master key will be rotated to a new one.
///       "storage_credentials": optional<object> Credentials for wallet storage. Storage type defines set of supported keys.
///                              Can be optional if storage supports default configuration.
///                              For 'default' storage type should be empty.
///       "key_derivation_method": optional<string> Algorithm to use for wallet key derivation:
///                          ARGON2I_MOD - derive secured wallet master key (used by default)
///                          ARGON2I_INT - derive secured wallet master key (less secured but faster)
///                          RAW - raw wallet key master provided (skip derivation).
///                                RAW keys can be generated with indy_generate_wallet_key call
///       "rekey_derivation_method": optional<string> Algorithm to use for wallet rekey derivation:
///                          ARGON2I_MOD - derive secured wallet master rekey (used by default)
///                          ARGON2I_INT - derive secured wallet master rekey (less secured but faster)
///                          RAW - raw wallet rekey master provided (skip derivation).
///                                RAW keys can be generated with indy_generate_wallet_key call
///   }
///
/// #Returns
/// err: Error code
/// handle: Handle to opened wallet to use in methods that require wallet access.
///
/// #Errors
/// Common*
/// Wallet*
#[no_mangle]
pub extern "C" fn indy_open_wallet(
    command_handle: CommandHandle,
    config: *const c_char,
    credentials: *const c_char,
    cb: Option<
        extern "C" fn(command_handle_: CommandHandle, err: ErrorCode, wallet_handle: WalletHandle),
    >,
) -> ErrorCode {
    debug!(
        "indy_open_wallet > config {:?} credentials {:?}",
        config, credentials
    );

    check_useful_validatable_json!(config, ErrorCode::CommonInvalidParam2, Config);
    check_useful_json!(credentials, ErrorCode::CommonInvalidParam3, Credentials);
    check_useful_c_callback!(cb, ErrorCode::CommonInvalidParam4);

    debug!(
        "indy_open_wallet ? config {:?} credentials {:?}",
        config,
        secret!(&credentials)
    );

    let locator = Locator::instance();

    let action = async move {
        let res = locator.wallet_controller.open(config, credentials).await;
        res
    };

    let cb = move |res: IndyResult<_>| {
        let (err, handle) = prepare_result!(res, INVALID_WALLET_HANDLE);
        debug!("indy_open_wallet ? err {:?} handle {:?}", err, handle);

        cb(command_handle, err, handle)
    };

    locator.executor.spawn_ok_instrumented(CommandMetric::WalletCommandOpen, action, cb);

    let res = ErrorCode::Success;
    debug!("indy_open_wallet < {:?}", res);
    res
}

/// Exports opened wallet
///
/// #Params:
/// wallet_handle: wallet handle returned by indy_open_wallet
/// export_config: JSON containing settings for input operation.
///   {
///     "path": <string>, Path of the file that contains exported wallet content
///     "key": <string>, Key or passphrase used for wallet export key derivation.
///                     Look to key_derivation_method param for information about supported key derivation methods.
///     "key_derivation_method": optional<string> Algorithm to use for wallet export key derivation:
///                              ARGON2I_MOD - derive secured export key (used by default)
///                              ARGON2I_INT - derive secured export key (less secured but faster)
///                              RAW - raw export key provided (skip derivation).
///                                RAW keys can be generated with indy_generate_wallet_key call
///   }
///
/// #Returns
/// Error code
///
/// #Errors
/// Common*
/// Wallet*
#[no_mangle]
pub extern "C" fn indy_export_wallet(
    command_handle: CommandHandle,
    wallet_handle: WalletHandle,
    export_config: *const c_char,
    cb: Option<extern "C" fn(command_handle_: CommandHandle, err: ErrorCode)>,
) -> ErrorCode {
    debug!(
        "indy_export_wallet > wallet_handle {:?} export_config {:?}",
        wallet_handle, export_config
    );

    check_useful_json!(export_config, ErrorCode::CommonInvalidParam3, ExportConfig);
    check_useful_c_callback!(cb, ErrorCode::CommonInvalidParam4);

    debug!(
        "indy_export_wallet ? wallet_handle {:?} export_config {:?}",
        wallet_handle,
        secret!(&export_config)
    );

    let locator = Locator::instance();

    let action = async move {
        let res = locator
            .wallet_controller
            .export(wallet_handle, export_config)
            .await;
        res
    };

    let cb = move |res: IndyResult<_>| {
        let err = prepare_result!(res);
        debug!("indy_export_wallet ? err {:?}", err);

        cb(command_handle, err);
    };

    locator.executor.spawn_ok_instrumented(CommandMetric::WalletCommandExport, action, cb);

    let res = ErrorCode::Success;
    debug!("indy_export_wallet < {:?}", res);
    res
}

/// Creates a new secure wallet and then imports its content
/// according to fields provided in import_config
/// This can be seen as an indy_create_wallet call with additional content import
///
/// #Params
/// config: Wallet configuration json.
/// {
///   "id": string, Identifier of the wallet.
///         Configured storage uses this identifier to lookup exact wallet data placement.
///   "storage_type": optional<string>, Type of the wallet storage. Defaults to 'default'.
///                  'Default' storage type allows to store wallet data in the local file.
///                  Custom storage types can be registered with indy_register_wallet_storage call.
///   "storage_config": optional<object>, Storage configuration json. Storage type defines set of supported keys.
///                     Can be optional if storage supports default configuration.
///                     For 'default' storage type configuration is:
///   {
///     "path": optional<string>, Path to the directory with wallet files.
///             Defaults to $HOME/.indy_client/wallet.
///             Wallet will be stored in the file {path}/{id}/sqlite.db
///   }
/// }
/// credentials: Wallet credentials json
/// {
///   "key": string, Key or passphrase used for wallet key derivation.
///                  Look to key_derivation_method param for information about supported key derivation methods.
///   "storage_credentials": optional<object> Credentials for wallet storage. Storage type defines set of supported keys.
///                          Can be optional if storage supports default configuration.
///                          For 'default' storage type should be empty.
///   "key_derivation_method": optional<string> Algorithm to use for wallet key derivation:
///                             ARGON2I_MOD - derive secured wallet master key (used by default)
///                             ARGON2I_INT - derive secured wallet master key (less secured but faster)
///                             RAW - raw wallet key master provided (skip derivation).
///                                RAW keys can be generated with indy_generate_wallet_key call
/// }
/// import_config: Import settings json.
/// {
///   "path": <string>, path of the file that contains exported wallet content
///   "key": <string>, key used for export of the wallet
/// }
///
/// #Returns
/// Error code
///
/// #Errors
/// Common*
/// Wallet*
#[no_mangle]
pub extern "C" fn indy_import_wallet(
    command_handle: CommandHandle,
    config: *const c_char,
    credentials: *const c_char,
    import_config: *const c_char,
    cb: Option<extern "C" fn(command_handle_: CommandHandle, err: ErrorCode)>,
) -> ErrorCode {
    debug!(
        "indy_import_wallet > config {:?} credentials {:?} import_config {:?}",
        config, credentials, import_config
    );

    check_useful_validatable_json!(config, ErrorCode::CommonInvalidParam2, Config);
    check_useful_json!(credentials, ErrorCode::CommonInvalidParam3, Credentials);
    check_useful_json!(import_config, ErrorCode::CommonInvalidParam4, ExportConfig);
    check_useful_c_callback!(cb, ErrorCode::CommonInvalidParam5);

    debug!(
        "indy_import_wallet ? config {:?} credentials {:?} import_config {:?}",
        config,
        secret!(&credentials),
        secret!(&import_config)
    );

    let locator = Locator::instance();

    let action = async move {
        let res = locator
            .wallet_controller
            .import(config, credentials, import_config)
            .await;
        res
    };

    let cb = move |res: IndyResult<_>| {
        let err = prepare_result!(res);
        debug!("indy_import_wallet ? err {:?}", err);

        cb(command_handle, err);
    };

    locator.executor.spawn_ok_instrumented(CommandMetric::WalletCommandImport, action, cb);

    let res = ErrorCode::Success;
    debug!("indy_import_wallet < {:?}", res);
    res
}

/// Closes opened wallet and frees allocated resources.
///
/// #Params
/// wallet_handle: wallet handle returned by indy_open_wallet.
///
/// #Returns
/// Error code
///
/// #Errors
/// Common*
/// Wallet*
#[no_mangle]
pub extern "C" fn indy_close_wallet(
    command_handle: CommandHandle,
    wallet_handle: WalletHandle,
    cb: Option<extern "C" fn(command_handle_: CommandHandle, err: ErrorCode)>,
) -> ErrorCode {
    debug!("indy_close_wallet > wallet_handle {:?}", wallet_handle);

    check_useful_c_callback!(cb, ErrorCode::CommonInvalidParam3);

    debug!("indy_close_wallet ? wallet_handle {:?}", wallet_handle);

    let locator = Locator::instance();

    let action = async move {
        let res = locator.wallet_controller.close(wallet_handle).await;
        res
    };

    let cb = move |res: IndyResult<_>| {
        let err = prepare_result!(res);
        debug!("indy_close_wallet ? err {:?}", err);

        cb(command_handle, err)
    };

    locator.executor.spawn_ok_instrumented(CommandMetric::WalletCommandClose, action, cb);

    let res = ErrorCode::Success;
    debug!("indy_close_wallet < {:?}", res);
    res
}

/// Deletes created wallet.
///
/// #Params
/// config: Wallet configuration json.
/// {
///   "id": string, Identifier of the wallet.
///         Configured storage uses this identifier to lookup exact wallet data placement.
///   "storage_type": optional<string>, Type of the wallet storage. Defaults to 'default'.
///                  'Default' storage type allows to store wallet data in the local file.
///                  Custom storage types can be registered with indy_register_wallet_storage call.
///   "storage_config": optional<object>, Storage configuration json. Storage type defines set of supported keys.
///                     Can be optional if storage supports default configuration.
///                     For 'default' storage type configuration is:
///   {
///     "path": optional<string>, Path to the directory with wallet files.
///             Defaults to $HOME/.indy_client/wallet.
///             Wallet will be stored in the file {path}/{id}/sqlite.db
///   }
/// }
/// credentials: Wallet credentials json
/// {
///   "key": string, Key or passphrase used for wallet key derivation.
///                  Look to key_derivation_method param for information about supported key derivation methods.
///   "storage_credentials": optional<object> Credentials for wallet storage. Storage type defines set of supported keys.
///                          Can be optional if storage supports default configuration.
///                          For 'default' storage type should be empty.
///   "key_derivation_method": optional<string> Algorithm to use for wallet key derivation:
///                             ARGON2I_MOD - derive secured wallet master key (used by default)
///                             ARGON2I_INT - derive secured wallet master key (less secured but faster)
///                             RAW - raw wallet key master provided (skip derivation).
///                                RAW keys can be generated with indy_generate_wallet_key call
/// }
///
/// #Returns
/// Error code
///
/// #Errors
/// Common*
/// Wallet*
#[no_mangle]
pub extern "C" fn indy_delete_wallet(
    command_handle: CommandHandle,
    config: *const c_char,
    credentials: *const c_char,
    cb: Option<extern "C" fn(command_handle_: CommandHandle, err: ErrorCode)>,
) -> ErrorCode {
    debug!(
        "indy_delete_wallet > config {:?} credentials {:?}",
        config, credentials
    );

    check_useful_validatable_json!(config, ErrorCode::CommonInvalidParam2, Config);
    check_useful_json!(credentials, ErrorCode::CommonInvalidParam3, Credentials);
    check_useful_c_callback!(cb, ErrorCode::CommonInvalidParam4);

    debug!(
        "indy_delete_wallet ? config {:?} credentials {:?}",
        config,
        secret!(&credentials)
    );

    let locator = Locator::instance();

    let action = async move {
        let res = locator.wallet_controller.delete(config, credentials).await;
        res
    };

    let cb = move |res: IndyResult<_>| {
        let err = prepare_result!(res);
        debug!("indy_delete_wallet ? err {:?}", err);

        cb(command_handle, err);
    };

    locator.executor.spawn_ok_instrumented(CommandMetric::WalletCommandDelete, action, cb);

    let res = ErrorCode::Success;
    debug!("indy_delete_wallet < {:?}", res);
    res
}

/// Generate wallet master key.
/// Returned key is compatible with "RAW" key derivation method.
/// It allows to avoid expensive key derivation for use cases when wallet keys can be stored in a secure enclave.
///
/// #Params
/// config: (optional) key configuration json.
/// {
///   "seed": string, (optional) Seed that allows deterministic key creation (if not set random one will be created).
///                              Can be UTF-8, base64 or hex string.
/// }
///
/// #Returns
/// err: Error code
///
/// #Errors
/// Common*
/// Wallet*
#[no_mangle]
pub extern "C" fn indy_generate_wallet_key(
    command_handle: CommandHandle,
    config: *const c_char,
    cb: Option<extern "C" fn(command_handle_: CommandHandle, err: ErrorCode, key: *const c_char)>,
) -> ErrorCode {
    debug!("indy_generate_wallet_key > config {:?}", config);

    check_useful_opt_json!(config, ErrorCode::CommonInvalidParam2, KeyConfig);
    check_useful_c_callback!(cb, ErrorCode::CommonInvalidParam3);

    debug!(
        "indy_generate_wallet_key ? config: {:?}",
        secret!(config.as_ref())
    );

    let locator = Locator::instance();

    let action = async move {
        let res = locator.wallet_controller.generate_key(config);
        res
    };

    let cb = move |res: IndyResult<_>| {
        let (err, key) = prepare_result!(res, String::new());
        debug!("indy_generate_wallet_key ? err {:?} result {:?}", err, key);

        let res = ctypes::string_to_cstring(key);
        cb(command_handle, err, res.as_ptr())
    };

    locator.executor.spawn_ok_instrumented(CommandMetric::WalletCommandGenerateKey, action, cb);

    let res = ErrorCode::Success;
    debug!("indy_generate_wallet_key {:?}", res);
    res
}
