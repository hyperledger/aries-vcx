use indy_api_types::{CommandHandle, ErrorCode, errors::prelude::*, WalletHandle};
use indy_utils::ctypes;
use libc::c_char;

use crate::Locator;
use crate::services::CommandMetric;

/// Creates keys (signing and encryption keys) for a new account.
/// #Params
/// alias: alias for a new keys
/// Example:
/// {
///     "alias": string
/// }
/// cb: Callback that takes command result as parameter.
///
/// #Returns
/// Error Code
/// cb:
/// - err: Error code.
///   alias: alias for a new keys
///   account_id: address of a new keys
///   pub_key: public key
///
/// #Errors
/// Common*
#[no_mangle]
pub extern "C" fn cheqd_keys_add_random(
    command_handle: CommandHandle,
    wallet_handle: WalletHandle,
    alias: *const c_char,
    cb: Option<
        extern "C" fn(command_handle_: CommandHandle, err: ErrorCode, key_info: *const c_char),
    >,
) -> ErrorCode {
    debug!("cheqd_keys_add_random > wallet_handle {:?} alias {:?} ", wallet_handle, alias);

    check_useful_c_str!(alias, ErrorCode::CommonInvalidParam1);
    check_useful_c_callback!(cb, ErrorCode::CommonInvalidParam2);

    debug!("cheqd_keys_add_random > alias {:?} ", alias);

    let locator = Locator::instance();

    let action = async move {
        let res = locator.cheqd_keys_controller.add_random(wallet_handle, &alias).await;
        res
    };

    let cb = move |res: IndyResult<_>| {
        let (err, res) = prepare_result!(res, String::new());
        debug!("cheqd_keys_add_random ? err {:?} res {:?}", err, res);

        let res = ctypes::string_to_cstring(res);
        cb(command_handle, err, res.as_ptr())
    };

    locator
        .executor
        .spawn_ok_instrumented(CommandMetric::CheqdKeysAddRandom, action, cb);

    let res = ErrorCode::Success;
    debug!("indy_replace_keys_start < {:?}", res);
    res
}

/// Creates keys (signing and encryption keys) for a new account.
/// #Params
/// alias: alias for a new keys
/// mnemonic: for generating keys
/// passphrase: password for a key, default is ""
/// Example:
/// {
///     "alias": string
///     "mnemonic": string
///     "passphrase": string
/// }
/// cb: Callback that takes command result as parameter.
///
/// #Returns
/// Error Code
/// cb:
/// - err: Error code.
///   alias: alias for a new keys
///   account_id: address of a new keys
///   pub_key: public key
///
/// #Errors
/// Common*
#[no_mangle]
pub extern "C" fn cheqd_keys_add_from_mnemonic(
    command_handle: CommandHandle,
    wallet_handle: WalletHandle,
    alias: *const c_char,
    mnemonic: *const c_char,
    passphrase: *const c_char,
    cb: Option<
        extern "C" fn(command_handle_: CommandHandle, err: ErrorCode, key_info: *const c_char),
    >,
) -> ErrorCode {
    debug!(
        "cheqd_keys_add_from_mnemonic > wallet_handle {:?} alias {:?}, mnemonic {:?} ",
        wallet_handle, alias, mnemonic
    );

    check_useful_c_str!(alias, ErrorCode::CommonInvalidParam1);
    check_useful_c_str!(mnemonic, ErrorCode::CommonInvalidParam2);
    check_useful_c_str_allow_empty!(passphrase, ErrorCode::CommonInvalidParam3);
    check_useful_c_callback!(cb, ErrorCode::CommonInvalidParam4);

    debug!(
        "cheqd_keys_add_from_mnemonic > alias {:?}, mnemonic {:?} ",
        alias, mnemonic
    );

    let locator = Locator::instance();

    let action = async move {
        let res = locator
            .cheqd_keys_controller
            .add_from_mnemonic(wallet_handle, &alias, &mnemonic, &passphrase)
            .await;
        res
    };

    let cb = move |res: IndyResult<_>| {
        let (err, res) = prepare_result!(res, String::new());
        debug!(
            "cheqd_keys_add_from_mnemonic ? err {:?} res {:?}",
            err, res
        );

        let res = ctypes::string_to_cstring(res);
        cb(command_handle, err, res.as_ptr())
    };

    locator
        .executor
        .spawn_ok_instrumented(CommandMetric::CheqdKeysAddFromMnemonic, action, cb);

    let res = ErrorCode::Success;
    debug!("cheqd_keys_add_from_mnemonic < {:?}", res);
    res
}

/// Get Key info by alias
/// #Params
/// alias: account alias for getting its keys
/// Example:
/// {
///     "alias": string
/// }
/// cb: Callback that takes command result as parameter.
///
/// #Returns
/// Error Code
/// cb:
/// - err: Error code.
///   alias: alias of asked keys
///   account_id: address of asked keys
///   pub_key: public key of asked keys
///
/// #Errors
/// Common*
#[no_mangle]
pub extern "C" fn cheqd_keys_get_info(
    command_handle: CommandHandle,
    wallet_handle: WalletHandle,
    alias: *const c_char,
    cb: Option<
        extern "C" fn(command_handle_: CommandHandle, err: ErrorCode, key_info: *const c_char),
    >,
) -> ErrorCode {
    debug!("cheqd_keys_key_info > wallet_handle {:?} alias {:?} ", wallet_handle, alias);

    check_useful_c_str!(alias, ErrorCode::CommonInvalidParam1);
    check_useful_c_callback!(cb, ErrorCode::CommonInvalidParam2);

    debug!("cheqd_keys_key_info > alias {:?} ", alias);

    let locator = Locator::instance();

    let action = async move {
        let res = locator.cheqd_keys_controller.get_info(wallet_handle, &alias).await;
        res
    };

    let cb = move |res: IndyResult<_>| {
        let (err, res) = prepare_result!(res, String::new());
        debug!("cheqd_keys_key_info ? err {:?} res {:?}", err, res);

        let res = ctypes::string_to_cstring(res);
        cb(command_handle, err, res.as_ptr())
    };

    locator
        .executor
        .spawn_ok_instrumented(CommandMetric::CheqdKeysKeyInfo, action, cb);

    let res = ErrorCode::Success;
    debug!("cheqd_keys_key_info < {:?}", res);
    res
}

/// List keys in specific wallet
/// #Params
/// command_handle: command handle to map callback to caller context.
/// wallet_handle: specific wallet
/// cb: Callback that takes command result as parameter.
///
/// #Returns
/// Error Code
/// cb:
/// - err: Error code.
///   List of keys as string json.
///
/// #Errors
/// Common*
#[no_mangle]
pub extern "C" fn cheqd_keys_list(
    command_handle: CommandHandle,
    wallet_handle: WalletHandle,
    cb: Option<
        extern "C" fn(command_handle_: CommandHandle, err: ErrorCode, key_info: *const c_char),
    >,
) -> ErrorCode {
    debug!(
        "cheqd_keys_list > wallet_handle {:?}",
        wallet_handle
    );

    check_useful_c_callback!(cb, ErrorCode::CommonInvalidParam3);

    debug!(
        "cheqd_keys_list ? wallet_handle {:?}",
        wallet_handle
    );

    let locator = Locator::instance();

    let action = async move {
        let res = locator
            .cheqd_keys_controller
            .list(wallet_handle)
            .await;
        res
    };

    let cb = move |res: IndyResult<_>| {
        let (err, res) = prepare_result!(res, String::new());
        debug!("cheqd_keys_get_list_keys ? err {:?} res {:?}", err, res);

        let res = ctypes::string_to_cstring(res);
        cb(command_handle, err, res.as_ptr())
    };

    locator.executor.spawn_ok_instrumented(CommandMetric::CheqdKeysGetListKeys, action, cb);

    let res = ErrorCode::Success;
    debug!("cheqd_keys_list < {:?}", res);
    res
}

/// Signs a message with a key.
///
/// #Params
/// alias: alias of an account associated with a key to use for signing
/// message_raw: a pointer to first byte of message to be signed
/// message_len: a message length
/// cb: Callback that takes command result as parameter.
///
/// #Returns
/// Signature bytes
///
/// #Errors
/// Common*
#[no_mangle]
pub extern "C" fn cheqd_keys_sign(
    command_handle: CommandHandle,
    wallet_handle: WalletHandle,
    alias: *const c_char,
    msg_raw: *const u8,
    msg_len: u32,
    cb: Option<
        extern "C" fn(
            command_handle_: CommandHandle,
            err: ErrorCode,
            signed_raw: *const u8,
            signed_len: u32,
        ),
    >,
) -> ErrorCode {
    debug!(
        "cheqd_keys_sign > wallet_handle {:?} alias {:?} msg_raw {:?} msg_len {:?}",
        wallet_handle, alias, msg_raw, msg_len
    );

    check_useful_c_str!(alias, ErrorCode::CommonInvalidParam1);
    check_useful_c_byte_array!(
        msg_raw,
        msg_len,
        ErrorCode::CommonInvalidParam2,
        ErrorCode::CommonInvalidParam3
    );
    check_useful_c_callback!(cb, ErrorCode::CommonInvalidParam4);

    debug!("cheqd_keys_sign > alias {:?} ", alias);

    let locator = Locator::instance();

    let action = async move {
        let res = locator.cheqd_keys_controller.sign(wallet_handle, &alias, &msg_raw).await;
        res
    };

    let cb = move |res: IndyResult<_>| {
        let (err, res) = prepare_result!(res, Vec::new());
        debug!("cheqd_keys_sign ? err {:?} res {:?}", err, res);

        let (signed_raw, signed_len) = ctypes::vec_to_pointer(&res);
        cb(command_handle, err, signed_raw, signed_len)
    };

    locator
        .executor
        .spawn_ok_instrumented(CommandMetric::CheqdKeysSign, action, cb);

    let res = ErrorCode::Success;
    debug!("cheqd_keys_sign < {:?}", res);
    res
}
