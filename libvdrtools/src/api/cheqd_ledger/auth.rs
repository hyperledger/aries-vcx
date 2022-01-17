use indy_api_types::{errors::prelude::*, CommandHandle, ErrorCode, WalletHandle};

use crate::services::CommandMetric;
use crate::Locator;
use indy_utils::ctypes;
use libc::c_char;


/// Build txn before sending
///
/// #Params
/// command_handle: command handle to map callback to caller context.
/// pool_alias: string alias of a pool
/// sender_public_key: public key of sender
/// msg_raw: message in raw format,
/// msg_len: length of message,
/// account_number: number of accounts,
/// sequence_number: how many txns are already written,
/// max_gas: how much gas user is ready to pay.,
/// max_coin_amount: how many coins user can pay,
/// max_coin_denom: which kink of coins user is ready to pay,
/// timeout_height: block height until which the transaction is valid,
/// memo: a note or comment to send with the transaction,
/// cb: Callback that takes command result as parameter.
///
/// #Returns
/// Success or error message.
#[no_mangle]
pub extern "C" fn cheqd_ledger_auth_build_tx(
    command_handle: CommandHandle,
    pool_alias: *const c_char,
    sender_public_key: *const c_char,
    msg_raw: *const u8,
    msg_len: u32,
    account_number: u64,
    sequence_number: u64,
    max_gas: u64,
    max_coin_amount: u64,
    max_coin_denom: *const c_char,
    timeout_height: u64,
    memo: *const c_char,
    cb: Option<
        extern "C" fn(
            command_handle_: CommandHandle,
            err: ErrorCode,
            tx_raw: *const u8,
            tx_len: u32,
        ),
    >,
) -> ErrorCode {
    debug!(
        "cheqd_ledger_auth_build_tx > pool_alias {:?} sender_public_key {:?} msg_raw {:?} \
        msg_len {:?} account_number {:?} sequence_number {:?} max_gas {:?} max_coin_amount \
        {:?} max_coin_denom {:?} timeout_height {:?} memo {:?}",
        pool_alias,
        sender_public_key,
        msg_raw,
        msg_len,
        account_number,
        sequence_number,
        max_gas,
        max_coin_amount,
        max_coin_denom,
        timeout_height,
        memo
    );

    check_useful_c_str!(pool_alias, ErrorCode::CommonInvalidParam2);
    check_useful_c_str!(sender_public_key, ErrorCode::CommonInvalidParam3);
    check_useful_c_byte_array!(
        msg_raw,
        msg_len,
        ErrorCode::CommonInvalidParam4,
        ErrorCode::CommonInvalidParam5
    );
    check_useful_c_str!(max_coin_denom, ErrorCode::CommonInvalidParam10);
    check_useful_c_str!(memo, ErrorCode::CommonInvalidParam12);
    check_useful_c_callback!(cb, ErrorCode::CommonInvalidParam13);

    debug!(
        "cheqd_ledger_auth_build_tx > pool_alias {:?} sender_public_key {:?} msg_raw {:?} \
        account_number {:?} sequence_number {:?} max_gas {:?} max_coin_amount \
        {:?} max_coin_denom {:?} timeout_height {:?} memo {:?}",
        pool_alias,
        sender_public_key,
        msg_raw,
        account_number,
        sequence_number,
        max_gas,
        max_coin_amount,
        max_coin_denom,
        timeout_height,
        memo
    );

    let locator = Locator::instance();

    let action = async move {
        let res = locator
            .cheqd_ledger_controller
            .auth_build_tx(
                &pool_alias,
                &sender_public_key,
                &msg_raw,
                account_number,
                sequence_number,
                max_gas,
                max_coin_amount,
                &max_coin_denom,
                timeout_height,
                &memo,
            )
            .await;
        res
    };

    let cb = move |res: IndyResult<_>| {
        let (err, tx) = prepare_result!(res, Vec::new());
        debug!("cheqd_ledger_auth_build_tx ? err {:?} tx {:?}", err, tx);

        let (tx_raw, tx_len) = ctypes::vec_to_pointer(&tx);
        cb(command_handle, err, tx_raw, tx_len)
    };

    locator
        .executor
        .spawn_ok_instrumented(CommandMetric::CheqdLedgerCommandBuildTx, action, cb);

    let res = ErrorCode::Success;
    debug!("cheqd_ledger_auth_build_tx < {:?}", res);
    res
}


/// Build query for getting info about account.
///
/// #Params
/// command_handle: command handle to map callback to caller context.
/// address: address of queried account
/// cb: Callback that takes command result as parameter.
///
/// #Returns
/// Success or error message.

#[no_mangle]
pub extern "C" fn cheqd_ledger_auth_build_query_account(
    command_handle: CommandHandle,
    address: *const c_char,
    cb: Option<extern "C" fn(command_handle_: CommandHandle, err: ErrorCode, query: *const c_char)>,
) -> ErrorCode {
    debug!(
        "cheqd_ledger_auth_build_query_account > address {:?}",
        address
    );

    check_useful_c_str!(address, ErrorCode::CommonInvalidParam2);
    check_useful_c_callback!(cb, ErrorCode::CommonInvalidParam3);

    debug!(
        "cheqd_ledger_auth_build_query_account > address {:?}",
        address
    );

    let locator = Locator::instance();

    let action = async move {
        let res = locator
            .cheqd_ledger_controller
            .auth_build_query_account(&address);
        res
    };

    let cb = move |res: IndyResult<_>| {
        let (err, query) = prepare_result!(res, String::new());
        debug!(
            "cheqd_ledger_auth_build_query_account: query: {:?}",
            query
        );

        let query = ctypes::string_to_cstring(query);
        cb(command_handle, err, query.as_ptr())
    };

    locator.executor.spawn_ok_instrumented(
        CommandMetric::CheqdLedgerCommandBuildQueryCosmosAuthAccount,
        action,
        cb,
    );

    let res = ErrorCode::Success;
    debug!(
        "cheqd_ledger_auth_build_query_account < {:?}",
        res
    );
    res
}

/// Parse response from query account.
///
/// #Params
/// command_handle: command handle to map callback to caller context.
/// query_resp: string representation of response from ledger
/// cb: Callback that takes command result as parameter.
///
/// #Returns
/// Success or error message.
#[no_mangle]
pub extern "C" fn cheqd_ledger_auth_parse_query_account_resp(
    command_handle: CommandHandle,
    query_resp: *const c_char,
    cb: Option<extern "C" fn(command_handle_: CommandHandle, err: ErrorCode, resp: *const c_char)>,
) -> ErrorCode {
    debug!(
        "cheqd_ledger_auth_parse_query_account_resp > query_resp {:?}",
        query_resp
    );

    check_useful_c_str!(query_resp, ErrorCode::CommonInvalidParam2);
    check_useful_c_callback!(cb, ErrorCode::CommonInvalidParam3);

    debug!(
        "cheqd_ledger_auth_parse_query_account_resp > query_resp {:?}",
        query_resp
    );

    let locator = Locator::instance();

    let action = async move {
        let res = locator
            .cheqd_ledger_controller
            .auth_parse_query_account_resp(&query_resp);
        res
    };

    let cb = move |res: IndyResult<_>| {
        let (err, resp) = prepare_result!(res, String::new());
        debug!(
            "cheqd_ledger_auth_parse_query_account_resp: resp: {:?}",
            resp
        );
        let resp = ctypes::string_to_cstring(resp);
        cb(command_handle, err, resp.as_ptr())
    };

    locator.executor.spawn_ok_instrumented(
        CommandMetric::CheqdLedgerCommandParseQueryCosmosAuthAccountResp,
        action,
        cb,
    );

    let res = ErrorCode::Success;
    debug!(
        "cheqd_ledger_auth_parse_query_account_resp < {:?}",
        res
    );
    res
}

/// Signs request message.
///
/// Adds submitter information to passed request json, signs it with submitter
/// sign key (see wallet_sign).
///
/// #Params
/// command_handle: command handle to map callback to caller context.
/// wallet_handle: wallet handle (created by open_wallet).
/// key_alias: alias of an account stored in the wallet and associated with a key to use for signing.
/// message_raw: a pointer to first byte of transaction to be signed
/// message_len: a message length
/// cb: Callback that takes command result as parameter.
///
/// #Returns
/// Signed transaction as bytes
/// Common*
#[no_mangle]
pub extern "C" fn cheqd_ledger_sign_tx(
    command_handle: CommandHandle,
    wallet_handle: WalletHandle,
    key_alias: *const c_char,
    tx_raw: *const u8,
    tx_len: u32,
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
        "cheqd_ledger_sign_tx > wallet_handle {:?} key_alias {:?} tx_raw {:?} tx_len {:?}",
        wallet_handle, key_alias, tx_raw, tx_len
    );

    check_useful_c_str!(key_alias, ErrorCode::CommonInvalidParam3);
    check_useful_c_byte_array!(
        tx_raw,
        tx_len,
        ErrorCode::CommonInvalidParam4,
        ErrorCode::CommonInvalidParam5
    );
    check_useful_c_callback!(cb, ErrorCode::CommonInvalidParam6);

    debug!("cheqd_ledger_sign_tx > key_alias {:?} ", key_alias);

    let locator = Locator::instance();

    let action = async move {
        let res = locator
            .cheqd_ledger_controller
            .sign_tx(wallet_handle, &key_alias, &tx_raw).await;
        res
    };

    let cb = move |res: IndyResult<_>| {
        let (err, res) = prepare_result!(res, Vec::new());
        debug!("cheqd_ledger_sign_tx ? err {:?} res {:?}", err, res);

        let (signed_raw, signed_len) = ctypes::vec_to_pointer(&res);
        cb(command_handle, err, signed_raw, signed_len)
    };

    locator
        .executor
        .spawn_ok_instrumented(CommandMetric::CheqdKeysSign, action, cb);

    let res = ErrorCode::Success;
    debug!("cheqd_ledger_sign_tx < {:?}", res);
    res
}
