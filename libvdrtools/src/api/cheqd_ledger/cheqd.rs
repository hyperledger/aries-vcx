use indy_api_types::{CommandHandle, ErrorCode, errors::prelude::*, WalletHandle};
use indy_utils::ctypes;
use libc::c_char;

use crate::Locator;
use crate::services::CommandMetric;


/// Signs write request message with a key associated with passed DID.
///
/// #Params
/// command_handle: command handle to map callback to caller context.
/// wallet_handle: wallet handle (created by open_wallet).
/// did:  DID which key needs to be used for signing the request.
/// request_bytes: a pointer to first byte of write request message
/// request_len: a message length
/// cb: Callback that takes command result as parameter.
///
/// #Returns
/// Request with signature inside.
///
/// #Errors
/// Common*
/// Wallet*
#[no_mangle]
pub extern "C" fn cheqd_ledger_cheqd_sign_msg_write_request(
    command_handle: CommandHandle,
    wallet_handle: WalletHandle,
    did: *const c_char,
    request_bytes: *const u8,
    request_len: u32,
    cb: Option<
        extern "C" fn(
            command_handle_: CommandHandle,
            err: ErrorCode,
            msg: *const u8,
            msg_len: u32,
        ),
    >,
) -> ErrorCode {
    debug!(
        "cheqd_ledger_cheqd_sign_msg_write_request >  \
            wallet_handle {:?} did {:?} request_bytes {:?} request_len {:?}",
         wallet_handle, did, request_bytes, request_len
    );

    check_useful_c_str!(did, ErrorCode::CommonInvalidParam3);
    check_useful_c_byte_array!(
        request_bytes,
        request_len,
        ErrorCode::CommonInvalidParam4,
        ErrorCode::CommonInvalidParam5
    );
    check_useful_c_callback!(cb, ErrorCode::CommonInvalidParam6);

    debug!(
        "cheqd_ledger_cheqd_sign_msg_write_request?  \
            wallet_handle {:?} verkey {:?} request_bytes {:?} request_len {:?}",
        wallet_handle, did, request_bytes, request_len
    );

    let locator = Locator::instance();

    let action = async move {
        let res = locator
            .cheqd_ledger_controller
            .sign_cheqd_request(wallet_handle, &request_bytes, &did)
            .await;
        res
    };


    let cb = move |res: IndyResult<_>| {
        let (err, msg) = prepare_result!(res, Vec::new());
        debug!(
            "cheqd_ledger_cheqd_sign_msg_write_request: signature: {:?}",
            msg
        );
        let (msg_raw, msg_len) = ctypes::vec_to_pointer(&msg);
        cb(command_handle, err, msg_raw, msg_len)
    };

    locator.executor.spawn_ok_instrumented(
        CommandMetric::CheqdLedgerCommandSignRequest,
        action,
        cb);

    let res = ErrorCode::Success;
    debug!("cheqd_ledger_cheqd_sign_msg_write_request < {:?}", res);
    res
}



/// Build request message to create `DID` on the Ledger.
///
/// #Params
/// command_handle: command handle to map callback to caller context.
/// wallet_handle: wallet handle (created by open_wallet).
/// did: DID as base58-encoded string.
/// verkey: Verification key associated with DID.
/// cb: Callback that takes command result as parameter.
///
/// #Returns
/// Built request message as bytes.
#[no_mangle]
pub extern "C" fn cheqd_ledger_cheqd_build_msg_create_did(
    command_handle: CommandHandle,
    did: *const c_char,
    verkey: *const c_char,
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
        "cheqd_ledger_cheqd_build_msg_create_did > did {:?} verkey {:?}",
        did, verkey
    );

    check_useful_c_str!(did, ErrorCode::CommonInvalidParam2);
    check_useful_c_str!(verkey, ErrorCode::CommonInvalidParam3);
    check_useful_c_callback!(cb, ErrorCode::CommonInvalidParam4);

    debug!(
        "cheqd_ledger_cheqd_build_msg_create_did > did {:?} verkey {:?}",
        did, verkey
    );

    let locator = Locator::instance();

    let action = async move {
        let res = locator
            .cheqd_ledger_controller
            .cheqd_build_msg_create_did(&did, &verkey)
            .await;
        res
    };

    let cb = move |res: IndyResult<_>| {
        let (err, msg) = prepare_result!(res, Vec::new());
        debug!("cheqd_ledger_cheqd_build_msg_create_did ? err {:?} res {:?}", err, msg);

        let (msg_raw, msg_len) = ctypes::vec_to_pointer(&msg);
        cb(command_handle, err, msg_raw, msg_len)
    };

    locator.executor.spawn_ok_instrumented(
        CommandMetric::CheqdLedgerCommandBuildMsgCreateDid,
        action,
        cb,
    );

    let res = ErrorCode::Success;
    debug!("cheqd_ledger_cheqd_build_msg_create_did < {:?}", res);
    res
}

/// Parse response received on creating a DID on the Ledger
///
/// #Params
/// command_handle: command handle to map callback to caller context.
/// commit_resp: response for creating a DID.
/// cb: Callback that takes command result as parameter.
///
/// #Returns
/// DID data:
///     {
///         "id" - string // same as DID
///     }
#[no_mangle]
pub extern "C" fn cheqd_ledger_cheqd_parse_msg_create_did_resp(
    command_handle: CommandHandle,
    commit_resp: *const c_char,
    cb: Option<
        extern "C" fn(command_handle_: CommandHandle, err: ErrorCode, msg_resp: *const c_char),
    >,
) -> ErrorCode {
    debug!(
        "cheqd_ledger_cheqd_parse_msg_create_did_resp > commit_resp {:?}",
        commit_resp
    );

    check_useful_c_str!(commit_resp, ErrorCode::CommonInvalidParam2);
    check_useful_c_callback!(cb, ErrorCode::CommonInvalidParam3);

    debug!(
        "cheqd_ledger_cheqd_parse_msg_create_did_resp > commit_resp {:?}",
        commit_resp
    );

    let locator = Locator::instance();

    let action = async move {
        let res = locator
            .cheqd_ledger_controller
            .cheqd_parse_msg_create_did_resp(&commit_resp);
        res
    };

    let cb = move |res: IndyResult<_>| {
        let (err, msg_resp) = prepare_result!(res, String::new());
        debug!(
            "cheqd_ledger_cheqd_parse_msg_create_did_resp: msg_resp: {:?}",
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
    debug!("cheqd_ledger_cheqd_parse_msg_create_did_resp < {:?}", res);
    res
}

/// Build request message to update `DID` on the Ledger.
///
/// #Params
/// command_handle: command handle to map callback to caller context.
/// wallet_handle: wallet handle (created by open_wallet).
/// did: Target DID to update,
/// verkey: Verification key associated with DID.
/// version_id: index of previously created DID
///             In order to get this value, you need to get DID info from the ledger and take `version_id` from the parsed response.
/// cb: Callback that takes command result as parameter.
///
/// #Returns
/// Success or error message.
#[no_mangle]
pub extern "C" fn cheqd_ledger_cheqd_build_msg_update_did(
    command_handle: CommandHandle,
    did: *const c_char,
    verkey: *const c_char,
    version_id: *const c_char,
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
        "cheqd_ledger_cheqd_build_msg_update_did > did {:?} verkey {:?} version_id {:?}",
        did, verkey, version_id,
    );
    check_useful_c_str!(did, ErrorCode::CommonInvalidParam2);
    check_useful_c_str!(verkey, ErrorCode::CommonInvalidParam3);
    check_useful_c_str!(version_id, ErrorCode::CommonInvalidParam4);
    check_useful_c_callback!(cb, ErrorCode::CommonInvalidParam5);

    debug!(
        "cheqd_ledger_cheqd_build_msg_update_did > did {:?} verkey {:?} version_id {:?}",
        did, verkey, version_id,
    );

    let locator = Locator::instance();

    let action = async move {
        let res = locator
            .cheqd_ledger_controller
            .cheqd_build_msg_update_did(&did, &verkey, &version_id)
            .await;
        res
    };

    let cb = move |res: IndyResult<_>| {
        let (err, msg) = prepare_result!(res, Vec::new());
        debug!("cheqd_ledger_cheqd_build_msg_update_did ? err {:?} res {:?}", err, msg);

        let (msg_raw, msg_len) = ctypes::vec_to_pointer(&msg);
        cb(command_handle, err, msg_raw, msg_len)
    };

    locator.executor.spawn_ok_instrumented(
        CommandMetric::CheqdLedgerCommandBuildMsgUpdateDid,
        action,
        cb,
    );

    let res = ErrorCode::Success;
    debug!("cheqd_ledger_cheqd_build_msg_update_did < {:?}", res);
    res
}

/// Parse response received on updating a DID on the Ledger
///
/// #Params
/// command_handle: command handle to map callback to caller context.
/// commit_resp: response for creating a DID.
/// cb: Callback that takes command result as parameter.
///
/// #Returns
/// DID data:
///     {
///         "id" - string // same as DID
///     }
#[no_mangle]
pub extern "C" fn cheqd_ledger_cheqd_parse_msg_update_did_resp(
    command_handle: CommandHandle,
    commit_resp: *const c_char,
    cb: Option<
        extern "C" fn(command_handle_: CommandHandle, err: ErrorCode, msg_resp: *const c_char),
    >,
) -> ErrorCode {
    debug!(
        "cheqd_ledger_cheqd_parse_msg_update_did_resp > commit_resp {:?}",
        commit_resp
    );

    check_useful_c_str!(commit_resp, ErrorCode::CommonInvalidParam2);
    check_useful_c_callback!(cb, ErrorCode::CommonInvalidParam3);

    debug!(
        "cheqd_ledger_cheqd_parse_msg_update_did_resp > commit_resp {:?}",
        commit_resp
    );

    let locator = Locator::instance();

    let action = async move {
        let res = locator
            .cheqd_ledger_controller
            .cheqd_parse_msg_update_did_resp(&commit_resp);
        res
    };

    let cb = move |res: IndyResult<_>| {
        let (err, msg_resp) = prepare_result!(res, String::new());
        debug!(
            "cheqd_ledger_cheqd_parse_msg_update_did_resp: msg_resp: {:?}",
            msg_resp
        );
        let msg_resp = ctypes::string_to_cstring(msg_resp);
        cb(command_handle, err, msg_resp.as_ptr())
    };

    locator.executor.spawn_ok_instrumented(
        CommandMetric::CheqdLedgerCommandParseMsgUpdateDidResp,
        action,
        cb,
    );

    let res = ErrorCode::Success;
    debug!("cheqd_ledger_cheqd_parse_msg_update_did_resp < {:?}", res);
    res
}


/// Build request for getting DID from the Ledger
///
/// #Params
/// command_handle: command handle to map callback to caller context.
/// did: requesting DID
/// cb: Callback that takes command result as parameter.
///
/// #Returns
/// Success or error message.
#[no_mangle]
pub extern "C" fn cheqd_ledger_cheqd_build_query_get_did(
    command_handle: CommandHandle,
    did: *const c_char,
    cb: Option<extern "C" fn(command_handle_: CommandHandle, err: ErrorCode, query: *const c_char)>,
) -> ErrorCode {
    debug!("cheqd_ledger_cheqd_build_query_get_did > did {:?}", did);

    check_useful_c_str!(did, ErrorCode::CommonInvalidParam2);
    check_useful_c_callback!(cb, ErrorCode::CommonInvalidParam3);

    debug!("cheqd_ledger_cheqd_build_query_get_did > did {:?}", did);

    let locator = Locator::instance();

    let action = async move {
        let res = locator.cheqd_ledger_controller.cheqd_build_query_get_did(&did);
        res
    };

    let cb = move |res: IndyResult<_>| {
        let (err, query) = prepare_result!(res, String::new());
        debug!("cheqd_ledger_cheqd_build_query_get_did: query: {:?}", query);

        let query = ctypes::string_to_cstring(query);
        cb(command_handle, err, query.as_ptr())
    };

    locator.executor.spawn_ok_instrumented(
        CommandMetric::CheqdLedgerCommandBuildQueryGetDid,
        action,
        cb,
    );

    let res = ErrorCode::Success;
    debug!("cheqd_ledger_cheqd_build_query_get_did < {:?}", res);
    res
}

/// Parse response after sending request for getting DID
///
/// #Params
/// command_handle: command handle to map callback to caller context.
/// query_resp: response for getting a DID.
/// cb: Callback that takes command result as parameter.
///
/// #Returns
/// DID data:
///     {
///         "did" - string
///         "metadata" {
///             created: string,
///             updated: string,
///             deactivated: bool,
///             version_id: string,
///         }
///     }
#[no_mangle]
pub extern "C" fn cheqd_ledger_cheqd_parse_query_get_did_resp(
    command_handle: CommandHandle,
    query_resp: *const c_char,
    cb: Option<extern "C" fn(command_handle_: CommandHandle, err: ErrorCode, resp: *const c_char)>,
) -> ErrorCode {
    debug!(
        "cheqd_ledger_cheqd_parse_query_get_did_resp > query_resp {:?}",
        query_resp
    );

    check_useful_c_str!(query_resp, ErrorCode::CommonInvalidParam2);
    check_useful_c_callback!(cb, ErrorCode::CommonInvalidParam3);

    debug!(
        "cheqd_ledger_cheqd_parse_query_get_did_resp > query_resp {:?}",
        query_resp
    );

    let locator = Locator::instance();

    let action = async move {
        let res = locator
            .cheqd_ledger_controller
            .cheqd_parse_query_get_did_resp(&query_resp);
        res
    };

    let cb = move |res: IndyResult<_>| {
        let (err, resp) = prepare_result!(res, String::new());
        debug!(
            "cheqd_ledger_cheqd_parse_query_get_did_resp: resp: {:?}",
            resp
        );
        let resp = ctypes::string_to_cstring(resp);
        cb(command_handle, err, resp.as_ptr())
    };

    locator.executor.spawn_ok_instrumented(
        CommandMetric::CheqdLedgerCommandParseQueryGetDidResp,
        action,
        cb,
    );

    let res = ErrorCode::Success;
    debug!("cheqd_ledger_cheqd_parse_query_get_did_resp < {:?}", res);
    res
}
