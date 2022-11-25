use std::ptr::null;

use futures::future::BoxFuture;
use libc::c_char;

use aries_vcx::error::{VcxError, VcxErrorKind};
use aries_vcx::indy;
use aries_vcx::vdrtools::{SearchHandle, WalletHandle, CommandHandle};
use aries_vcx::indy::wallet::{import, RestoreWalletConfigs, WalletConfig};
use aries_vcx::utils::error;

use crate::api_lib;
use crate::api_lib::global::profile::get_main_wallet;
use crate::api_lib::global::wallet::open_as_main_wallet;
use crate::api_lib::global::wallet::{export_main_wallet, get_main_wallet_handle};
use crate::api_lib::utils::cstring::CStringUtils;
use crate::api_lib::utils::error::{set_current_error, set_current_error_vcx};
use crate::api_lib::utils::runtime::execute_async;

/// Creates new wallet and master secret using provided config. Keeps wallet closed.
///
/// #Params
/// command_handle: command handle to map callback to user context.
///
/// wallet_config: wallet configuration
///
/// cb: Callback that provides configuration or error status
///
/// # Example wallet config ->
/// {
///   "wallet_name": "my_wallet_name",
///   "wallet_key": "123456",
///   "wallet_key_derivation": "ARGON2I_MOD",
///   "wallet_type": "postgres_storage",
///   "storage_config": "{\"url\":\"localhost:5432\"}",
///   "storage_credentials": "{\"account\":\"postgres\",\"password\":\"password_123\",\"admin_account\":\"postgres\",\"admin_password\":\"password_foo\"}"
/// }
///
/// #Returns
/// Error code as a u32
#[no_mangle]
pub extern "C" fn vcx_create_wallet(
    command_handle: CommandHandle,
    wallet_config: *const c_char,
    cb: Option<extern "C" fn(xcommand_handle: CommandHandle, err: u32)>,
) -> u32 {
    info!("vcx_create_wallet >>>");

    check_useful_c_callback!(cb, VcxErrorKind::InvalidOption);
    check_useful_c_str!(wallet_config, VcxErrorKind::InvalidOption);

    trace!(
        "vcx_create_wallet(command_handle: {}, wallet_config: {})",
        command_handle,
        wallet_config
    );

    let wallet_config = match serde_json::from_str::<WalletConfig>(&wallet_config) {
        Ok(wallet_config) => wallet_config,
        Err(err) => {
            set_current_error(&err);
            error!("vcx_create_wallet >>> invalid wallet configuration; err: {:?}", err);
            return error::INVALID_CONFIGURATION.code_num;
        }
    };

    execute_async::<BoxFuture<'static, Result<(), ()>>>(Box::pin(async move {
        match api_lib::global::wallet::create_main_wallet(&wallet_config).await {
            Err(err) => {
                error!("vcx_create_wallet_cb(command_handle: {}, rc: {}", command_handle, err);
                cb(command_handle, err.into());
            }
            Ok(_) => {
                trace!(
                    "vcx_create_wallet_cb(command_handle: {}, rc: {})",
                    command_handle,
                    error::SUCCESS.message
                );
                cb(command_handle, 0);
            }
        }
        Ok(())
    }));

    error::SUCCESS.code_num
}

/// Creates issuer's did and keypair and stores them in the wallet.
///
/// #Params
/// command_handle: command handle to map callback to user context.
///
/// enterprise_seed: Seed used to generate institution did, keypair and other secrets
///
/// cb: Callback that provides institution config or error status
///
/// # Example institution config ->{
///   "institution_did": "V4SGRU86Z58d6TV7PBUe6f",
///   "institution_verkey": "GJ1SzoWzavQYfNL9XkaJdrQejfztN4XqdsiV4ct3LXKL",
/// }
///
/// #Returns
/// Error code as a u32
#[no_mangle]
pub extern "C" fn vcx_configure_issuer_wallet(
    command_handle: CommandHandle,
    enterprise_seed: *const c_char,
    cb: Option<extern "C" fn(xcommand_handle: CommandHandle, err: u32, *const c_char)>,
) -> u32 {
    info!("vcx_configure_issuer_wallet >>>");

    check_useful_c_callback!(cb, VcxErrorKind::InvalidOption);
    check_useful_c_str!(enterprise_seed, VcxErrorKind::InvalidOption);

    trace!(
        "vcx_configure_issuer_wallet(command_handle: {}, enterprise_seed: {})",
        command_handle,
        enterprise_seed
    );

    execute_async::<BoxFuture<'static, Result<(), ()>>>(Box::pin(async move {
        // todo future - use profile wallet to stop indy dependency
        match indy::wallet::wallet_configure_issuer(get_main_wallet_handle(), &enterprise_seed).await {
            Err(err) => {
                error!(
                    "vcx_configure_issuer_wallet_cb(command_handle: {}, rc: {}",
                    command_handle, err
                );
                cb(command_handle, err.into(), null());
            }
            Ok(conf) => {
                let conf = serde_json::to_string(&conf).unwrap();
                trace!(
                    "vcx_configure_issuer_wallet_cb(command_handle: {}, rc: {}, conf: {})",
                    command_handle,
                    error::SUCCESS.message,
                    conf
                );
                let conf = CStringUtils::string_to_cstring(conf.to_string());
                cb(command_handle, 0, conf.as_ptr());
            }
        }
        Ok(())
    }));

    error::SUCCESS.code_num
}

/// Opens wallet chosen using provided config.
///
/// #Params
/// command_handle: command handle to map callback to user context.
///
/// wallet_config: wallet configuration
///
/// cb: Callback that provides wallet handle as u32 (wrappers require unsigned integer) or error status
///
/// #Returns
/// Error code as a u32
#[no_mangle]
pub extern "C" fn vcx_open_main_wallet(
    command_handle: CommandHandle,
    wallet_config: *const c_char,
    cb: Option<extern "C" fn(xcommand_handle: CommandHandle, err: u32, wh: i32)>,
) -> u32 {
    info!("vcx_open_main_wallet >>>");

    check_useful_c_callback!(cb, VcxErrorKind::InvalidOption);
    check_useful_c_str!(wallet_config, VcxErrorKind::InvalidOption);

    let wallet_config = match serde_json::from_str::<WalletConfig>(&wallet_config) {
        Ok(wallet_config) => wallet_config,
        Err(err) => {
            set_current_error(&err);
            error!("vcx_open_main_wallet >>> invalid wallet configuration; err: {:?}", err);
            return error::INVALID_CONFIGURATION.code_num;
        }
    };

    trace!("vcx_open_main_wallet(command_handle: {})", command_handle);

    execute_async::<BoxFuture<'static, Result<(), ()>>>(Box::pin(async move {
        match open_as_main_wallet(&wallet_config).await {
            Err(err) => {
                set_current_error_vcx(&err);
                error!(
                    "vcx_open_main_wallet_cb(command_handle: {}, rc: {}",
                    command_handle, err
                );
                cb(command_handle, err.into(), aries_vcx::vdrtools::INVALID_WALLET_HANDLE.0);
            }
            Ok(wh) => {
                trace!(
                    "vcx_open_main_wallet_cb(command_handle: {}, rc: {}, wh: {})",
                    command_handle,
                    error::SUCCESS.message,
                    wh.0
                );
                cb(command_handle, 0, wh.0);
            }
        }
        Ok(())
    }));

    error::SUCCESS.code_num
}

/// Closes the main wallet.
///
/// #Params
/// command_handle: command handle to map callback to user context.
///
/// #Returns
/// Error code as a u32
#[no_mangle]
pub extern "C" fn vcx_close_main_wallet(
    command_handle: CommandHandle,
    cb: Option<extern "C" fn(xcommand_handle: CommandHandle, err: u32)>,
) -> u32 {
    info!("vcx_close_main_wallet >>>");

    check_useful_c_callback!(cb, VcxErrorKind::InvalidOption);

    trace!("vcx_close_main_wallet(command_handle: {})", command_handle);

    execute_async::<BoxFuture<'static, Result<(), ()>>>(Box::pin(async move {
        match api_lib::global::wallet::close_main_wallet().await {
            Err(err) => {
                error!(
                    "vcx_close_main_wallet_cb(command_handle: {}, rc: {}",
                    command_handle, err
                );
                cb(command_handle, err.into());
            }
            Ok(_) => {
                trace!(
                    "vcx_close_main_wallet_cb(command_handle: {}, rc: {})",
                    command_handle,
                    error::SUCCESS.message
                );
                cb(command_handle, 0);
            }
        }
        Ok(())
    }));

    error::SUCCESS.code_num
}

/// Adds a record to the wallet
/// Assumes there is an open wallet.
/// #Params
///
/// command_handle: command handle to map callback to user context.
///
/// type_: type of record. (e.g. 'data', 'string', 'foobar', 'image')
///
/// id: the id ("key") of the record.
///
/// value: value of the record with the associated id.
///
/// tags_json: the record tags used for search and storing meta information as json:
///   {
///     "tagName1": <str>, // string tag (will be stored encrypted)
///     "tagName2": <int>, // int tag (will be stored encrypted)
///     "~tagName3": <str>, // string tag (will be stored un-encrypted)
///     "~tagName4": <int>, // int tag (will be stored un-encrypted)
///   }
///  The tags_json must be valid json, and if no tags are to be associated with the
/// record, then the empty '{}' json must be passed.
///
/// cb: Callback that any errors or a receipt of transfer
///
/// #Returns
/// Error code as a u32
///
#[no_mangle]
pub extern "C" fn vcx_wallet_add_record(
    command_handle: CommandHandle,
    type_: *const c_char,
    id: *const c_char,
    value: *const c_char,
    tags_json: *const c_char,
    cb: Option<extern "C" fn(xcommand_handle: CommandHandle, err: u32)>,
) -> u32 {
    info!("vcx_wallet_add_record >>>");

    check_useful_c_str!(type_, VcxErrorKind::InvalidOption);
    check_useful_c_str!(id, VcxErrorKind::InvalidOption);
    check_useful_c_str!(value, VcxErrorKind::InvalidOption);
    check_useful_c_str!(tags_json, VcxErrorKind::InvalidOption);
    check_useful_c_callback!(cb, VcxErrorKind::InvalidOption);

    trace!(
        "vcx_wallet_add_record(command_handle: {}, type_: {}, id: {}, value: {}, tags_json: {})",
        command_handle,
        secret!(&type_),
        secret!(&id),
        secret!(&value),
        secret!(&tags_json)
    );

    execute_async::<BoxFuture<'static, Result<(), ()>>>(Box::pin(async move {
        let wallet = get_main_wallet();
        match wallet.add_wallet_record(&type_, &id, &value, Some(&tags_json)).await {
            Ok(()) => {
                trace!(
                    "vcx_wallet_add_record(command_handle: {}, rc: {})",
                    command_handle,
                    error::SUCCESS.message
                );

                cb(command_handle, error::SUCCESS.code_num);
            }
            Err(err) => {
                set_current_error_vcx(&err);
                trace!("vcx_wallet_add_record(command_handle: {}, rc: {})", command_handle, err);

                cb(command_handle, err.into());
            }
        };

        Ok(())
    }));

    error::SUCCESS.code_num
}

/// Updates the value of a record already in the wallet.
/// Assumes there is an open wallet and that a type and id pair already exists.
/// #Params
///
/// command_handle: command handle to map callback to user context.
///
/// type_: type of record. (e.g. 'data', 'string', 'foobar', 'image')
///
/// id: the id ("key") of the record.
///
/// value: New value of the record with the associated id.
///
/// cb: Callback that any errors or a receipt of transfer
///
/// #Returns
/// Error code as a u32
///
#[no_mangle]
pub extern "C" fn vcx_wallet_update_record_value(
    command_handle: CommandHandle,
    type_: *const c_char,
    id: *const c_char,
    value: *const c_char,
    cb: Option<extern "C" fn(xcommand_handle: CommandHandle, err: u32)>,
) -> u32 {
    info!("vcx_wallet_update_record_value >>>");

    check_useful_c_str!(type_, VcxErrorKind::InvalidOption);
    check_useful_c_str!(id, VcxErrorKind::InvalidOption);
    check_useful_c_str!(value, VcxErrorKind::InvalidOption);
    check_useful_c_callback!(cb, VcxErrorKind::InvalidOption);

    trace!(
        "vcx_wallet_update_record_value(command_handle: {}, type_: {}, id: {}, value: {})",
        command_handle,
        secret!(&type_),
        secret!(&id),
        secret!(&value)
    );

    execute_async::<BoxFuture<'static, Result<(), ()>>>(Box::pin(async move {
        let wallet = get_main_wallet();
        match wallet.update_wallet_record_value(&type_, &id, &value).await {
            Ok(()) => {
                trace!(
                    "vcx_wallet_update_record_value(command_handle: {}, rc: {})",
                    command_handle,
                    error::SUCCESS.message
                );

                cb(command_handle, error::SUCCESS.code_num);
            }
            Err(err) => {
                set_current_error_vcx(&err);
                trace!(
                    "vcx_wallet_update_record_value(command_handle: {}, rc: {})",
                    command_handle,
                    err
                );

                cb(command_handle, err.into());
            }
        };

        Ok(())
    }));

    error::SUCCESS.code_num
}

/// Updates the value of a record tags already in the wallet.
/// Assumes there is an open wallet and that a type and id pair already exists.
/// #Params
///
/// command_handle: command handle to map callback to user context.
///
/// type_: type of record. (e.g. 'data', 'string', 'foobar', 'image')
///
/// id: the id ("key") of the record.
///
/// tags_json: New tags for the record with the associated id and type.
///
/// cb: Callback that any errors or a receipt of transfer
///
/// #Returns
/// Error code as a u32
///
#[no_mangle]
pub extern "C" fn vcx_wallet_update_record_tags(
    command_handle: CommandHandle,
    type_: *const c_char,
    id: *const c_char,
    tags_json: *const c_char,
    cb: Option<extern "C" fn(xcommand_handle: CommandHandle, err: u32)>,
) -> u32 {
    info!("vcx_wallet_update_record_tags >>>");

    check_useful_c_str!(type_, VcxErrorKind::InvalidOption);
    check_useful_c_str!(id, VcxErrorKind::InvalidOption);
    check_useful_c_str!(tags_json, VcxErrorKind::InvalidOption);
    check_useful_c_callback!(cb, VcxErrorKind::InvalidOption);

    trace!(
        "vcx_wallet_update_record_tags(command_handle: {}, type_: {}, id: {}, tags_json: {})",
        command_handle,
        secret!(&type_),
        secret!(&id),
        secret!(&tags_json)
    );

    execute_async::<BoxFuture<'static, Result<(), ()>>>(Box::pin(async move {
        let wallet = get_main_wallet();
        match wallet.update_wallet_record_tags(&type_, &id, &tags_json).await {
            Ok(()) => {
                trace!(
                    "vcx_wallet_update_record_tags(command_handle: {}, rc: {})",
                    command_handle,
                    error::SUCCESS.message
                );

                cb(command_handle, error::SUCCESS.code_num);
            }
            Err(err) => {
                set_current_error_vcx(&err);
                trace!(
                    "vcx_wallet_update_record_tags(command_handle: {}, rc: {})",
                    command_handle,
                    err
                );

                cb(command_handle, err.into());
            }
        };

        Ok(())
    }));

    error::SUCCESS.code_num
}

/// Adds tags to a record.
/// Assumes there is an open wallet and that a type and id pair already exists.
/// #Params
///
/// command_handle: command handle to map callback to user context.
///
/// type_: type of record. (e.g. 'data', 'string', 'foobar', 'image')
///
/// id: the id ("key") of the record.
///
/// tags_json: Tags for the record with the associated id and type.
///
/// cb: Callback that any errors or a receipt of transfer
///
/// #Returns
/// Error code as a u32
///
#[no_mangle]
pub extern "C" fn vcx_wallet_add_record_tags(
    command_handle: CommandHandle,
    type_: *const c_char,
    id: *const c_char,
    tags_json: *const c_char,
    cb: Option<extern "C" fn(xcommand_handle: CommandHandle, err: u32)>,
) -> u32 {
    info!("vcx_wallet_add_record_tags >>>");

    check_useful_c_str!(type_, VcxErrorKind::InvalidOption);
    check_useful_c_str!(id, VcxErrorKind::InvalidOption);
    check_useful_c_str!(tags_json, VcxErrorKind::InvalidOption);
    check_useful_c_callback!(cb, VcxErrorKind::InvalidOption);

    trace!(
        "vcx_wallet_add_record_tags(command_handle: {}, type_: {}, id: {}, tags_json: {})",
        command_handle,
        secret!(&type_),
        secret!(&id),
        secret!(&tags_json)
    );

    execute_async::<BoxFuture<'static, Result<(), ()>>>(Box::pin(async move {
        let wallet = get_main_wallet();
        match wallet.add_wallet_record_tags(&type_, &id, &tags_json).await {
            Ok(()) => {
                trace!(
                    "vcx_wallet_add_record_tags(command_handle: {}, rc: {})",
                    command_handle,
                    error::SUCCESS.message
                );

                cb(command_handle, error::SUCCESS.code_num);
            }
            Err(err) => {
                set_current_error_vcx(&err);
                trace!(
                    "vcx_wallet_add_record_tags(command_handle: {}, rc: {})",
                    command_handle,
                    err
                );

                cb(command_handle, err.into());
            }
        };

        Ok(())
    }));

    error::SUCCESS.code_num
}

/// Deletes tags from a record.
/// Assumes there is an open wallet and that a type and id pair already exists.
/// #Params
///
/// command_handle: command handle to map callback to user context.
///
/// type_: type of record. (e.g. 'data', 'string', 'foobar', 'image')
///
/// id: the id ("key") of the record.
///
/// tag_names_json: Tags to remove from the record with the associated id and type.
///
/// cb: Callback that any errors or a receipt of transfer
///
/// #Returns
/// Error code as a u32
///
#[no_mangle]
pub extern "C" fn vcx_wallet_delete_record_tags(
    command_handle: CommandHandle,
    type_: *const c_char,
    id: *const c_char,
    tag_names_json: *const c_char,
    cb: Option<extern "C" fn(xcommand_handle: CommandHandle, err: u32)>,
) -> u32 {
    info!("vcx_wallet_delete_record_tags >>>");

    check_useful_c_str!(type_, VcxErrorKind::InvalidOption);
    check_useful_c_str!(id, VcxErrorKind::InvalidOption);
    check_useful_c_str!(tag_names_json, VcxErrorKind::InvalidOption);
    check_useful_c_callback!(cb, VcxErrorKind::InvalidOption);

    trace!(
        "vcx_wallet_delete_record_tags(command_handle: {}, type_: {}, id: {}, tag_names_json: {})",
        command_handle,
        secret!(&type_),
        secret!(&id),
        secret!(&tag_names_json)
    );

    execute_async::<BoxFuture<'static, Result<(), ()>>>(Box::pin(async move {
        let wallet = get_main_wallet();
        match wallet.delete_wallet_record_tags(&type_, &id, &tag_names_json).await {
            Ok(()) => {
                trace!(
                    "vcx_wallet_delete_record_tags(command_handle: {}, rc: {})",
                    command_handle,
                    error::SUCCESS.message
                );

                cb(command_handle, error::SUCCESS.code_num);
            }
            Err(err) => {
                set_current_error_vcx(&err);
                trace!(
                    "vcx_wallet_delete_record_tags(command_handle: {}, rc: {})",
                    command_handle,
                    err
                );

                cb(command_handle, err.into());
            }
        };

        Ok(())
    }));

    error::SUCCESS.code_num
}

/// Deletes an existing record.
/// Assumes there is an open wallet and that a type and id pair already exists.
/// #Params
///
/// command_handle: command handle to map callback to user context.
///
/// type_: type of record. (e.g. 'data', 'string', 'foobar', 'image')
///
/// id: the id ("key") of the record.
///
/// cb: Callback that any errors or a receipt of transfer
///
/// #Returns
/// Error code as a u32
/// Error will be a libindy error code
///
#[no_mangle]
pub extern "C" fn vcx_wallet_get_record(
    command_handle: CommandHandle,
    type_: *const c_char,
    id: *const c_char,
    options_json: *const c_char,
    cb: Option<extern "C" fn(xcommand_handle: CommandHandle, err: u32, record_json: *const c_char)>,
) -> u32 {
    info!("vcx_wallet_get_record >>>");

    check_useful_c_str!(type_, VcxErrorKind::InvalidOption);
    check_useful_c_str!(id, VcxErrorKind::InvalidOption);
    check_useful_c_str!(options_json, VcxErrorKind::InvalidOption);
    check_useful_c_callback!(cb, VcxErrorKind::InvalidOption);

    trace!(
        "vcx_wallet_get_record(command_handle: {}, type_: {}, id: {}, options: {})",
        command_handle,
        secret!(&type_),
        secret!(&id),
        options_json
    );

    execute_async::<BoxFuture<'static, Result<(), ()>>>(Box::pin(async move {
        let wallet = get_main_wallet();
        match wallet.get_wallet_record(&type_, &id, &options_json).await {
            Ok(err) => {
                trace!(
                    "vcx_wallet_get_record(command_handle: {}, rc: {}, record_json: {})",
                    command_handle,
                    error::SUCCESS.message,
                    err
                );

                let msg = CStringUtils::string_to_cstring(err);

                cb(command_handle, error::SUCCESS.code_num, msg.as_ptr());
            }
            Err(err) => {
                set_current_error_vcx(&err);
                trace!(
                    "vcx_wallet_get_record(command_handle: {}, rc: {}, record_json: {})",
                    command_handle,
                    err,
                    "null"
                );

                let msg = CStringUtils::string_to_cstring("".to_string());
                cb(command_handle, err.into(), msg.as_ptr());
            }
        };

        Ok(())
    }));

    error::SUCCESS.code_num
}

/// Deletes an existing record.
/// Assumes there is an open wallet and that a type and id pair already exists.
/// #Params
///
/// command_handle: command handle to map callback to user context.
///
/// type_: type of record. (e.g. 'data', 'string', 'foobar', 'image')
///
/// id: the id ("key") of the record.
///
/// cb: Callback that any errors or a receipt of transfer
///
/// #Returns
/// Error code as a u32
/// Error will be a libindy error code
///
#[no_mangle]
pub extern "C" fn vcx_wallet_delete_record(
    command_handle: CommandHandle,
    type_: *const c_char,
    id: *const c_char,
    cb: Option<extern "C" fn(xcommand_handle: CommandHandle, err: u32)>,
) -> u32 {
    info!("vcx_wallet_delete_record >>>");

    check_useful_c_str!(type_, VcxErrorKind::InvalidOption);
    check_useful_c_str!(id, VcxErrorKind::InvalidOption);
    check_useful_c_callback!(cb, VcxErrorKind::InvalidOption);

    trace!(
        "vcx_wallet_delete_record(command_handle: {}, type_: {}, id: {})",
        command_handle,
        secret!(&type_),
        secret!(&id)
    );

    execute_async::<BoxFuture<'static, Result<(), ()>>>(Box::pin(async move {
        let wallet = get_main_wallet();
        match wallet.delete_wallet_record(&type_, &id).await {
            Ok(()) => {
                trace!(
                    "vcx_wallet_delete_record(command_handle: {}, rc: {})",
                    command_handle,
                    error::SUCCESS.message
                );

                cb(command_handle, error::SUCCESS.code_num);
            }
            Err(err) => {
                set_current_error_vcx(&err);
                trace!(
                    "vcx_wallet_delete_record(command_handle: {}, rc: {})",
                    command_handle,
                    err
                );

                cb(command_handle, err.into());
            }
        };

        Ok(())
    }));

    error::SUCCESS.code_num
}

/// Opens a storage search handle
///
/// #Params
///
/// command_handle: command handle to map callback to user context.
///
/// type_: type of record. (e.g. 'data', 'string', 'foobar', 'image')
///
/// query_json: MongoDB style query to wallet record tags:
///  {
///    "tagName": "tagValue",
///    $or: {
///      "tagName2": { $regex: 'pattern' },
///      "tagName3": { $gte: 123 },
///    },
///  }
/// options_json:
///  {
///    retrieveRecords: (optional, true by default) If false only "counts" will be calculated,
///    retrieveTotalCount: (optional, false by default) Calculate total count,
///    retrieveType: (optional, false by default) Retrieve record type,
///    retrieveValue: (optional, true by default) Retrieve record value,
///    retrieveTags: (optional, false by default) Retrieve record tags,
///  }
/// cb: Callback that any errors or a receipt of transfer
///
/// #Returns
/// Error code as a u32
#[no_mangle]
pub extern "C" fn vcx_wallet_open_search(
    command_handle: CommandHandle,
    type_: *const c_char,
    query_json: *const c_char,
    options_json: *const c_char,
    cb: Option<extern "C" fn(command_handle_: CommandHandle, err: u32, search_handle: SearchHandle)>,
) -> u32 {
    info!("vcx_wallet_open_search >>>");

    check_useful_c_str!(type_, VcxErrorKind::InvalidOption);
    check_useful_c_str!(query_json, VcxErrorKind::InvalidOption);
    check_useful_c_str!(options_json, VcxErrorKind::InvalidOption);
    check_useful_c_callback!(cb, VcxErrorKind::InvalidOption);

    trace!(
        "vcx_wallet_open_search(command_handle: {}, type_: {}, query_json: {}, options_json: {})",
        command_handle,
        secret!(&type_),
        secret!(&query_json),
        secret!(&options_json)
    );

    execute_async::<BoxFuture<'static, Result<(), ()>>>(Box::pin(async move {
        // todo future - use profile wallet to stop binding to indy
        match indy::wallet::open_search_wallet(get_main_wallet_handle(), &type_, &query_json, &options_json).await {
            Ok(handle) => {
                trace!(
                    "vcx_wallet_open_search(command_handle: {}, rc_: {}, search_handle: {:?})",
                    command_handle,
                    error::SUCCESS.message,
                    handle
                );

                cb(command_handle, error::SUCCESS.code_num, handle);
            }
            Err(err) => {
                set_current_error_vcx(&err);
                trace!(
                    "vcx_wallet_get_record(command_handle: {}, rc: {}, record_json: {})",
                    command_handle,
                    err,
                    "null"
                );

                cb(command_handle, err.into(), SearchHandle(0));
            }
        };

        Ok(())
    }));

    error::SUCCESS.code_num
}

/// Fetch next records for wallet search.
///
/// Not if there are no records this call returns WalletNoRecords error.
///
/// #Params
/// command_handle: command handle to map callback to user context.
/// wallet_search_handle: wallet search handle (created by vcx_wallet_open_search)
/// count: Count of records to fetch
///
/// #Returns
/// wallet records json:
/// {
///   totalCount: <int>, // present only if retrieveTotalCount set to true
///   records: [{ // present only if retrieveRecords set to true
///       id: "Some id",
///       type: "Some type", // present only if retrieveType set to true
///       value: "Some value", // present only if retrieveValue set to true
///       tags: <tags json>, // present only if retrieveTags set to true
///   }],
/// }
#[no_mangle]
pub extern "C" fn vcx_wallet_search_next_records(
    command_handle: CommandHandle,
    wallet_search_handle: SearchHandle,
    count: usize,
    cb: Option<extern "C" fn(command_handle_: CommandHandle, err: u32, records_json: *const c_char)>,
) -> u32 {
    info!("vcx_wallet_search_next_records >>>");

    check_useful_c_callback!(cb, VcxErrorKind::InvalidOption);

    trace!(
        "vcx_wallet_search_next_records(command_handle: {}, wallet_search_handle: {:?})",
        command_handle,
        wallet_search_handle
    );

    execute_async::<BoxFuture<'static, Result<(), ()>>>(Box::pin(async move {
        // todo future - use profile wallet to stop binding to indy
        match indy::wallet::fetch_next_records_wallet(get_main_wallet_handle(), wallet_search_handle, count).await {
            Ok(handle) => {
                trace!(
                    "vcx_wallet_search_next_records(command_handle: {}, rc: {}, record_json: {})",
                    command_handle,
                    error::SUCCESS.message,
                    handle
                );

                let msg = CStringUtils::string_to_cstring(handle);

                cb(command_handle, error::SUCCESS.code_num, msg.as_ptr());
            }
            Err(err) => {
                set_current_error_vcx(&err);
                trace!(
                    "vcx_wallet_get_record(command_handle: {}, rc: {}, record_json: {})",
                    command_handle,
                    err,
                    "null"
                );

                let msg = CStringUtils::string_to_cstring("".to_string());
                cb(command_handle, err.into(), msg.as_ptr());
            }
        };

        Ok(())
    }));

    error::SUCCESS.code_num
}

/// Close a search
///
/// #Params
///
/// command_handle: command handle to map callback to user context.
///
/// search_handle: wallet search handle
///
/// cb: Callback that any errors or a receipt of transfer
///
/// #Returns
/// Error code as a u32
#[no_mangle]
pub extern "C" fn vcx_wallet_close_search(
    command_handle: CommandHandle,
    search_handle: SearchHandle,
    cb: Option<extern "C" fn(xcommand_handle: CommandHandle, err: u32)>,
) -> u32 {
    info!("vcx_wallet_close_search >>>");

    check_useful_c_callback!(cb, VcxErrorKind::InvalidOption);

    trace!(
        "vcx_wallet_close_search(command_handle: {}, search_handle: {:?})",
        command_handle,
        search_handle
    );

    execute_async::<BoxFuture<'static, Result<(), ()>>>(Box::pin(async move {
        trace!(
            "vcx_wallet_close_search(command_handle: {}, rc: {})",
            command_handle,
            error::SUCCESS.message
        );
        match indy::wallet::close_search_wallet(search_handle).await {
            Ok(()) => {
                trace!(
                    "vcx_wallet_close_search(command_handle: {}, rc: {})",
                    command_handle,
                    error::SUCCESS.message
                );
                cb(command_handle, error::SUCCESS.code_num);
            }
            Err(err) => {
                trace!(
                    "vcx_wallet_close_search(command_handle: {}, rc: {})",
                    command_handle,
                    err
                );
                cb(command_handle, err.into());
            }
        };

        Ok(())
    }));

    error::SUCCESS.code_num
}

/// Exports opened wallet
///
/// Note this endpoint is EXPERIMENTAL. Function signature and behaviour may change
/// in the future releases.
///
/// #Params:
/// command_handle: Handle for User's Reference only.
/// path: Path to export wallet to User's File System.
/// backup_key: String representing the User's Key for securing (encrypting) the exported Wallet.
/// cb: Callback that provides the success/failure of the api call.
/// #Returns
/// Error code - success indicates that the api call was successfully created and execution
/// is scheduled to begin in a separate thread.
#[no_mangle]
pub extern "C" fn vcx_wallet_export(
    command_handle: CommandHandle,
    path: *const c_char,
    backup_key: *const c_char,
    cb: Option<extern "C" fn(xcommand_handle: CommandHandle, err: u32)>,
) -> u32 {
    info!("vcx_wallet_export >>>");

    check_useful_c_callback!(cb, VcxErrorKind::InvalidOption);
    check_useful_c_str!(path, VcxErrorKind::InvalidOption);
    check_useful_c_str!(backup_key, VcxErrorKind::InvalidOption);

    trace!(
        "vcx_wallet_export(command_handle: {}, path: {}, backup_key: ****)",
        command_handle,
        path
    );

    execute_async::<BoxFuture<'static, Result<(), ()>>>(Box::pin(async move {
        trace!(
            "vcx_wallet_export(command_handle: {}, path: {}, backup_key: ****)",
            command_handle,
            path
        );
        match export_main_wallet(&path, &backup_key).await {
            Ok(()) => {
                let return_code = error::SUCCESS.code_num;
                trace!(
                    "vcx_wallet_export(command_handle: {}, rc: {})",
                    command_handle,
                    return_code
                );
                cb(command_handle, return_code);
            }
            Err(err) => {
                error!("vcx_wallet_export(command_handle: {}, rc: {})", command_handle, err);
                cb(command_handle, err.into());
            }
        };

        Ok(())
    }));

    error::SUCCESS.code_num
}

/// Creates a new secure wallet and then imports its content
/// according to fields provided in import_config
/// Cannot be used if wallet is already opened (Especially if vcx_init has already been used).
///
/// Note this only works for default storage type (file), as currently this function does not let
/// you pass down information about wallet storage_type, storage_config, storage_credentials.
///
/// Note this endpoint is EXPERIMENTAL. Function signature and behaviour may change
/// in the future releases.
///
/// config: "{"wallet_name":"","wallet_key":"","exported_wallet_path":"","backup_key":"","key_derivation":""}"
/// exported_wallet_path: Path of the file that contains exported wallet content
/// backup_key: Key used when creating the backup of the wallet (For encryption/decrption)
/// Optional<key_derivation>: method of key derivation used by libindy. By default, libvcx uses ARGON2I_INT
/// cb: Callback that provides the success/failure of the api call.
/// #Returns
/// Error code - success indicates that the api call was successfully created and execution
/// is scheduled to begin in a separate thread.
#[no_mangle]
pub extern "C" fn vcx_wallet_import(
    command_handle: CommandHandle,
    config: *const c_char,
    cb: Option<extern "C" fn(xcommand_handle: CommandHandle, err: u32)>,
) -> u32 {
    info!("vcx_wallet_import >>>");

    check_useful_c_callback!(cb, VcxErrorKind::InvalidOption);
    check_useful_c_str!(config, VcxErrorKind::InvalidOption);

    trace!("vcx_wallet_import(command_handle: {}, config: ****)", command_handle);

    let config = match serde_json::from_str::<RestoreWalletConfigs>(&config) {
        Ok(config) => config,
        Err(err) => {
            set_current_error(&err);
            error!("vcx_wallet_import >>> invalid import configuration; err: {:?}", err);
            return error::INVALID_CONFIGURATION.code_num;
        }
    };

    execute_async::<BoxFuture<'static, Result<(), ()>>>(Box::pin(async move {
        trace!("vcx_wallet_import(command_handle: {}, config: ****)", command_handle);
        match import(&config).await {
            Ok(()) => {
                trace!(
                    "vcx_wallet_import(command_handle: {}, rc: {})",
                    command_handle,
                    error::SUCCESS.message
                );
                cb(command_handle, error::SUCCESS.code_num);
            }
            Err(err) => {
                error!("vcx_wallet_import(command_handle: {}, rc: {})", command_handle, err);
                cb(command_handle, err.into());
            }
        };
        Ok(())
    }));

    error::SUCCESS.code_num
}

/// Set the wallet handle before calling vcx_init_minimal
///
/// #params
///
/// handle: wallet handle that libvcx should use
///
/// #Returns
/// Error code as u32
#[no_mangle]
pub extern "C" fn vcx_wallet_set_handle(handle: WalletHandle) -> WalletHandle {
    api_lib::global::wallet::set_main_wallet_handle(handle)
}

#[cfg(test)]
pub mod tests {
    use std::ffi::CString;

    use aries_vcx::global::settings;
    use aries_vcx::indy::wallet::{delete_wallet, WalletConfig};
    use aries_vcx::utils::devsetup::{SetupDefaults, SetupEmpty, TempFile};

    use crate::api_lib::api_c::vcx::test_utils::{_test_add_and_get_wallet_record, _vcx_create_and_open_wallet};
    use crate::api_lib::global::wallet::{close_main_wallet, create_and_open_as_main_wallet};
    use crate::api_lib::utils::return_types_u32;
    use crate::api_lib::utils::timeout::TimeoutUtils;

    use super::*;

    #[test]
    #[cfg(feature = "general_test")]
    fn test_create_wallet() {
        let _setup = SetupEmpty::init();

        let wallet_name = format!("test_create_wallet_{}", uuid::Uuid::new_v4().to_string());
        let config = json!({
            "wallet_name": wallet_name,
            "wallet_key": settings::DEFAULT_WALLET_KEY,
            "wallet_key_derivation": settings::WALLET_KDF_RAW
        })
        .to_string();
        let cb = return_types_u32::Return_U32::new().unwrap();
        let err = vcx_create_wallet(
            cb.command_handle,
            CString::new(format!("{}", config)).unwrap().into_raw(),
            Some(cb.get_callback()),
        );
        assert_eq!(err, error::SUCCESS.code_num);
        cb.receive(TimeoutUtils::some_custom(1)).unwrap();
    }

    #[tokio::test]
    #[cfg(feature = "general_test")]
    async fn test_add_record() {
        _vcx_create_and_open_wallet().unwrap();

        let xtype = CStringUtils::string_to_cstring("record_type".to_string());
        let id = CStringUtils::string_to_cstring("123".to_string());
        let value = CStringUtils::string_to_cstring("Record Value".to_string());
        let tags = CStringUtils::string_to_cstring("{}".to_string());
        // Valid add
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
        cb.receive(TimeoutUtils::some_medium()).unwrap();
        // Failure because of duplicate
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
        assert_eq!(
            cb.receive(TimeoutUtils::some_medium()).err(),
            Some(error::DUPLICATE_WALLET_RECORD.code_num)
        );

        close_main_wallet().await.unwrap();
    }

    #[tokio::test]
    #[cfg(feature = "general_test")]
    async fn test_add_record_with_tag() {
        _vcx_create_and_open_wallet().unwrap();

        let xtype = CStringUtils::string_to_cstring("record_type".to_string());
        let id = CStringUtils::string_to_cstring("123".to_string());
        let value = CStringUtils::string_to_cstring("Record Value".to_string());
        let tags = CStringUtils::string_to_cstring(r#"{"tagName1":"tag1","tagName2":"tag2"}"#.to_string());

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
        cb.receive(TimeoutUtils::some_medium()).unwrap();

        close_main_wallet().await.unwrap();
    }

    #[tokio::test]
    #[cfg(feature = "general_test")]
    async fn test_get_record_fails_with_no_value() {
        _vcx_create_and_open_wallet().unwrap();

        let xtype = CStringUtils::string_to_cstring("record_type".to_string());
        let id = CStringUtils::string_to_cstring("123".to_string());
        let options = json!({
            "retrieveType": true,
            "retrieveValue": true,
            "retrieveTags": false
        })
        .to_string();
        let options = CStringUtils::string_to_cstring(options);

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
        assert_eq!(
            cb.receive(TimeoutUtils::some_medium()).err(),
            Some(error::WALLET_RECORD_NOT_FOUND.code_num)
        );

        close_main_wallet().await.unwrap();
    }

    #[tokio::test]
    #[cfg(feature = "general_test")]
    async fn test_get_record_value_success() {
        _vcx_create_and_open_wallet().unwrap();
        _test_add_and_get_wallet_record();
        close_main_wallet().await.unwrap();
    }

    #[tokio::test]
    #[cfg(feature = "general_test")]
    async fn test_delete_record() {
        _vcx_create_and_open_wallet().unwrap();

        let xtype = CStringUtils::string_to_cstring("record_type".to_string());
        let id = CStringUtils::string_to_cstring("123".to_string());
        let value = CStringUtils::string_to_cstring("Record Value".to_string());
        let tags = CStringUtils::string_to_cstring("{}".to_string());

        // Add record
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
        cb.receive(TimeoutUtils::some_medium()).unwrap();

        // Successful deletion
        let cb = return_types_u32::Return_U32::new().unwrap();
        assert_eq!(
            vcx_wallet_delete_record(cb.command_handle, xtype.as_ptr(), id.as_ptr(), Some(cb.get_callback())),
            error::SUCCESS.code_num
        );
        cb.receive(TimeoutUtils::some_medium()).unwrap();

        // Fails with no record
        let cb = return_types_u32::Return_U32::new().unwrap();
        assert_eq!(
            vcx_wallet_delete_record(cb.command_handle, xtype.as_ptr(), id.as_ptr(), Some(cb.get_callback())),
            error::SUCCESS.code_num
        );
        assert_eq!(
            cb.receive(TimeoutUtils::some_medium()).err(),
            Some(error::WALLET_RECORD_NOT_FOUND.code_num)
        );

        close_main_wallet().await.unwrap();
    }

    #[tokio::test]
    #[cfg(feature = "general_test")]
    async fn test_update_record_value() {
        _vcx_create_and_open_wallet().unwrap();

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

        // Assert no record to update
        let cb = return_types_u32::Return_U32::new().unwrap();
        assert_eq!(
            vcx_wallet_update_record_value(
                cb.command_handle,
                xtype.as_ptr(),
                id.as_ptr(),
                options.as_ptr(),
                Some(cb.get_callback())
            ),
            error::SUCCESS.code_num
        );
        assert_eq!(
            cb.receive(TimeoutUtils::some_medium()).err(),
            Some(error::WALLET_RECORD_NOT_FOUND.code_num)
        );

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
        cb.receive(TimeoutUtils::some_medium()).unwrap();

        // Assert update works
        let cb = return_types_u32::Return_U32::new().unwrap();
        assert_eq!(
            vcx_wallet_update_record_value(
                cb.command_handle,
                xtype.as_ptr(),
                id.as_ptr(),
                options.as_ptr(),
                Some(cb.get_callback())
            ),
            error::SUCCESS.code_num
        );
        cb.receive(TimeoutUtils::some_medium()).unwrap();
        close_main_wallet().await.unwrap();
    }

    #[tokio::test(flavor = "multi_thread", worker_threads = 2)]
    #[cfg(feature = "general_test")]
    async fn test_wallet_export_import() {
        let _setup = SetupDefaults::init();

        let wallet_name = "test_wallet_import_export";

        let export_file = TempFile::prepare_path(wallet_name);

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
        create_and_open_as_main_wallet(&wallet_config).await.unwrap();

        let backup_key = settings::get_config_value(settings::CONFIG_WALLET_BACKUP_KEY).unwrap();

        let cb = return_types_u32::Return_U32::new().unwrap();
        let cstr_file = CString::new(export_file.path.clone()).unwrap();
        let cstr_backup_key = CString::new(backup_key.clone()).unwrap();
        assert_eq!(
            vcx_wallet_export(
                cb.command_handle,
                cstr_file.as_ptr(),
                cstr_backup_key.as_ptr(),
                Some(cb.get_callback())
            ),
            error::SUCCESS.code_num
        );
        cb.receive(TimeoutUtils::some_long()).unwrap();

        close_main_wallet().await.unwrap();
        delete_wallet(&wallet_config).await.unwrap();

        let import_config = json!({
            settings::CONFIG_WALLET_NAME: wallet_config.wallet_name.clone(),
            settings::CONFIG_WALLET_KEY: wallet_config.wallet_key.clone(),
            settings::CONFIG_EXPORTED_WALLET_PATH: export_file.path,
            settings::CONFIG_WALLET_BACKUP_KEY: backup_key,
            settings::CONFIG_WALLET_KEY_DERIVATION: settings::WALLET_KDF_RAW,
        })
        .to_string();

        let cb = return_types_u32::Return_U32::new().unwrap();
        let cstr_config = CString::new(import_config).unwrap();
        assert_eq!(
            vcx_wallet_import(cb.command_handle, cstr_config.as_ptr(), Some(cb.get_callback())),
            error::SUCCESS.code_num
        );
        cb.receive(TimeoutUtils::some_long()).unwrap();

        delete_wallet(&wallet_config).await.unwrap();
    }
}
