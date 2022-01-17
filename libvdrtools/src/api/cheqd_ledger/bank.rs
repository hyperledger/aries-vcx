use indy_api_types::{CommandHandle, ErrorCode, errors::prelude::*};
use indy_utils::ctypes;
use libc::c_char;

use crate::Locator;
use crate::services::CommandMetric;

/// Send coins to other account.
///
/// #Params
/// command_handle: command handle to map callback to caller context.
/// from: address of sender coins
/// to: address of getter coins
/// amount: Amount of coins for sending
/// denom: Denomination of coins
/// cb: Callback that takes command result as parameter.
///
/// #Returns
/// Success or error message.
#[no_mangle]
pub extern "C" fn cheqd_ledger_bank_build_msg_send(
    command_handle: CommandHandle,
    from: *const c_char,
    to: *const c_char,
    amount: *const c_char,
    denom: *const c_char,
    cb: Option<
        extern "C" fn(
            command_handle_: CommandHandle,
            err: ErrorCode,
            msg_raw: *const u8,
            msg_len: u32,
        ),
    >,
) -> ErrorCode {
    debug!(
        "cheqd_ledger_bank_build_msg_send > from {:?} to {:?} amount {:?} denom {:?}",
        from, to, amount, denom
    );

    check_useful_c_str!(from, ErrorCode::CommonInvalidParam2);
    check_useful_c_str!(to, ErrorCode::CommonInvalidParam3);
    check_useful_c_str!(amount, ErrorCode::CommonInvalidParam4);
    check_useful_c_str!(denom, ErrorCode::CommonInvalidParam5);
    check_useful_c_callback!(cb, ErrorCode::CommonInvalidParam6);

    debug!(
        "cheqd_ledger_bank_build_msg_send > did {:?} creator {:?} verkey {:?} alias {:?}",
        from, to, amount, denom
    );

    let locator = Locator::instance();

    let action = async move {
        let res = locator
            .cheqd_ledger_controller
            .bank_build_msg_send(&from, &to, &amount, &denom);
        res
    };

    let cb = move |res: IndyResult<_>| {
        let (err, msg) = prepare_result!(res, Vec::new());
        debug!(
            "cheqd_ledger_bank_build_msg_send: signature: {:?}",
            msg
        );
        let (msg_raw, msg_len) = ctypes::vec_to_pointer(&msg);
        cb(command_handle, err, msg_raw, msg_len)
    };

    locator.executor.spawn_ok_instrumented(
        CommandMetric::CheqdLedgerCommandBuildMsgSend,
        action,
        cb,
    );

    let res = ErrorCode::Success;
    debug!("cheqd_ledger_bank_build_msg_send < {:?}", res);
    res
}

/// Parse response for send coins tx.
///
/// #Params
/// command_handle: command handle to map callback to caller context.
/// commit_resp: response for send coins tx.
/// cb: Callback that takes command result as parameter.
///
/// #Returns
/// Success or error message.
#[no_mangle]
pub extern "C" fn cheqd_ledger_bank_parse_msg_send_resp(
    command_handle: CommandHandle,
    commit_resp: *const c_char,
    cb: Option<
        extern "C" fn(command_handle_: CommandHandle, err: ErrorCode, msg_resp: *const c_char),
    >,
) -> ErrorCode {
    debug!(
        "cheqd_ledger_bank_parse_msg_send_resp > commit_resp {:?}",
        commit_resp
    );

    check_useful_c_str!(commit_resp, ErrorCode::CommonInvalidParam2);
    check_useful_c_callback!(cb, ErrorCode::CommonInvalidParam3);

    debug!(
        "cheqd_ledger_bank_parse_msg_send_resp > commit_resp {:?}",
        commit_resp
    );

    let locator = Locator::instance();

    let action = async move {
        let res = locator
            .cheqd_ledger_controller
            .bank_parse_msg_send_resp(&commit_resp);
        res
    };

    let cb = move |res: IndyResult<_>| {
        let (err, msg_resp) = prepare_result!(res, String::new());
        debug!(
            "cheqd_ledger_bank_parse_msg_send_resp: msg_resp: {:?}",
            msg_resp
        );
        let msg_resp = ctypes::string_to_cstring(msg_resp);
        cb(command_handle, err, msg_resp.as_ptr())
    };

    locator.executor.spawn_ok_instrumented(
        CommandMetric::CheqdLedgerCommandParseMsgCreateDidResp,
        action,
        cb,
    );

    let res = ErrorCode::Success;
    debug!("cheqd_ledger_bank_parse_msg_send_resp < {:?}", res);
    res
}

/// Get balance of account.
///
/// #Params
/// command_handle: command handle to map callback to caller context.
/// address: address of account which need to get.
/// denom: currency of balance for getting.
/// cb: Callback that takes command result as parameter.
///
/// #Returns
/// Success or error message.
#[no_mangle]
pub extern "C" fn cheqd_ledger_bank_build_query_balance(
    command_handle: CommandHandle,
    address: *const c_char,
    denom: *const c_char,
    cb: Option<
        extern "C" fn(command_handle_: CommandHandle, err: ErrorCode, msg_resp: *const c_char),
    >,
) -> ErrorCode {
    debug!(
        "cheqd_ledger_bank_build_query_balance > address {:?} denom {:?}",
        address, denom
    );
    check_useful_c_str!(address, ErrorCode::CommonInvalidParam2);
    check_useful_c_str!(denom, ErrorCode::CommonInvalidParam3);
    check_useful_c_callback!(cb, ErrorCode::CommonInvalidParam4);

    debug!(
        "cheqd_ledger_bank_build_query_balance > address {:?} denom {:?}",
        address, denom
    );

    let locator = Locator::instance();

    let action = async move {
        let res = locator
            .cheqd_ledger_controller
            .bank_build_query_balance(address, denom);
        res
    };

    let cb = move |res: IndyResult<_>| {
        let (err, msg_resp) = prepare_result!(res, String::new());
        debug!(
            "cheqd_ledger_bank_build_query_balance: signature: {:?}",
            msg_resp
        );
        let msg_resp = ctypes::string_to_cstring(msg_resp);
        cb(command_handle, err, msg_resp.as_ptr())
    };

    locator.executor.spawn_ok_instrumented(
        CommandMetric::CheqdLedgerCommandBuildQueryBalance,
        action,
        cb,
    );

    let res = ErrorCode::Success;
    debug!("cheqd_ledger_bank_build_query_balance < {:?}", res);
    res
}

/// Parse response for get balance tx.
///
/// #Params
/// command_handle: command handle to map callback to caller context.
/// commit_resp: response for get balance tx.
/// cb: Callback that takes command result as parameter.
///
/// #Returns
/// Success or error message.
#[no_mangle]
pub extern "C" fn cheqd_ledger_bank_parse_query_balance_resp(
    command_handle: CommandHandle,
    commit_resp: *const c_char,
    cb: Option<
        extern "C" fn(command_handle_: CommandHandle, err: ErrorCode, msg_resp: *const c_char),
    >,
) -> ErrorCode {
    debug!(
        "cheqd_ledger_bank_parse_query_balance_resp > commit_resp {:?}",
        commit_resp
    );

    check_useful_c_str!(commit_resp, ErrorCode::CommonInvalidParam2);
    check_useful_c_callback!(cb, ErrorCode::CommonInvalidParam3);

    debug!(
        "cheqd_ledger_bank_parse_query_balance_resp > commit_resp {:?}",
        commit_resp
    );

    let locator = Locator::instance();

    let action = async move {
        let res = locator
            .cheqd_ledger_controller
            .bank_parse_query_balance_resp(&commit_resp);
        res
    };

    let cb = move |res: IndyResult<_>| {
        let (err, msg_resp) = prepare_result!(res, String::new());
        debug!(
            "cheqd_ledger_bank_parse_query_balance_resp: msg_resp: {:?}",
            msg_resp
        );
        let msg_resp = ctypes::string_to_cstring(msg_resp);
        cb(command_handle, err, msg_resp.as_ptr())
    };

    locator.executor.spawn_ok_instrumented(
        CommandMetric::CheqdLedgerCommandParseQueryBalanceResp,
        action,
        cb,
    );

    let res = ErrorCode::Success;
    debug!("cheqd_ledger_bank_parse_query_balance_resp < {:?}", res);
    res
}
