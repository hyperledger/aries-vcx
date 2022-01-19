use indy_api_types::{CommandHandle, ErrorCode, errors::prelude::*};
use libc::c_char;
use indy_api_types::errors::IndyResult;
use crate::Locator;
use indy_utils::ctypes;
use crate::services::CommandMetric;

/// Build txn for querying txn by hash
/// #Params
/// command_handle: command handle to map callback to caller context.
/// hash: hash-string of txn which should be queried from ledger.
/// cb: Callback that takes command result as parameter.
///
/// #Returns
/// Error Code
/// cb:
/// - err: Error code.
///   String of Request like:
/// "{
///     "path":"/cosmos.tx.v1beta1.Service/GetTx",
///     "data":"0A4032363239374435374131464631453443393436324534383944464635353944394632354645443536423231343241323337394444313336414545333146443945",
///     "prove":true
/// }"
/// "data" string - it's a protobuf string.
/// #Errors
/// Common*
#[no_mangle]
pub extern "C" fn cheqd_ledger_tx_build_query_get_tx_by_hash(
    command_handle: CommandHandle,
    hash: *const c_char,
    cb: Option<extern "C" fn(command_handle_: CommandHandle, err: ErrorCode, query: *const c_char)>,
) -> ErrorCode {
    debug!("cheqd_ledger_tx_build_query_get_tx_by_hash > hash {:?}", hash);

    check_useful_c_str!(hash, ErrorCode::CommonInvalidParam2);
    check_useful_c_callback!(cb, ErrorCode::CommonInvalidParam3);

    debug!("cheqd_ledger_tx_build_query_get_tx_by_hash > hash {:?}", hash);

    let locator = Locator::instance();

    let action = async move {
        let res = locator.cheqd_ledger_controller.cheqd_build_query_get_tx_by_hash(&hash);
        res
    };

    let cb = move |res: IndyResult<_>| {
        let (err, query) = prepare_result!(res, String::new());
        debug!("cheqd_ledger_tx_build_query_get_tx_by_hash: query: {:?}", query);

        let query = ctypes::string_to_cstring(query);
        cb(command_handle, err, query.as_ptr())
    };

    locator.executor.spawn_ok_instrumented(
        CommandMetric::CheqdLedgerCommandBuildQueryGetTxByHash,
        action,
        cb,
    );

    let res = ErrorCode::Success;
    debug!("cheqd_ledger_tx_build_query_get_tx_by_hash < {:?}", res);
    res
}


/// Parse response from get tx by hash function
/// #Params
/// command_handle: command handle to map callback to caller context.
/// query_resp: response from ledger with protobuf inside.
/// cb: Callback that takes command result as parameter.
///
/// #Returns
/// Error Code
/// cb:
/// - err: Error code.
/// - JSON string which can be looked like:
/// {
/// 	"tx": {
/// 		"body": {
/// 			"messages": [{
/// 				"type_url": "MsgCreateNym",
/// 				"value": {
/// 					"creator": "cheqd1l9sq0se0jd3vklyrrtjchx4ua47awug5vsyeeh",
/// 					"alias": "test_alias",
/// 					"verkey": "test_verkey",
/// 					"did": "test_did",
/// 					"role": "test_role"
/// 				}
/// 			}],
/// 			"memo": "memo",
/// 			"timeout_height": 52,
/// 			"extension_options": [],
/// 			"non_critical_extension_options": []
/// 		},
/// 		"auth_info": {
/// 			"signer_infos": [{
/// 				"public_key": {
/// 					"secp256k1": {
/// 						"key": [2, 59, 126, 95, 52, 102, 213, 99, 251, 102, 62, 148, 101, 72, 226, 188, 243, 222, 31, 35, 148, 19, 127, 79, 75, 79, 37, 160, 132, 193, 33, 148, 7]
/// 					}
/// 				},
/// 				"mode_info": {
/// 					"sum": {
/// 						"Single": {
/// 							"mode": 1
/// 						}
/// 					}
/// 				},
/// 				"sequence": 0
/// 			}],
/// 			"fee": {
/// 				"amount": [{
/// 					"denom": "ncheq",
/// 					"amount": "0"
/// 				}],
/// 				"gas_limit": 300000,
/// 				"payer": "",
/// 				"granter": ""
/// 			}
/// 		},
/// 		"signatures": [
/// 			[1, 225, 116, 194, 154, 244, 148, 8, 209, 8, 174, 61, 108, 6, 39, 116, 111, 218, 47, 116, 88, 255, 47, 247, 235, 37, 91, 162, 57, 189, 40, 227, 81, 132, 215, 23, 63, 222, 4, 15, 25, 23, 227, 183, 91, 125, 75, 61, 151, 211, 195, 174, 194, 110, 10, 206, 153, 85, 166, 178, 8, 252, 146, 123]
/// 		]
/// 	},
/// 	"tx_response": {
/// 		"height": 33,
/// 		"txhash": "2FDD5C0975E18CF34EB20CBF9855C90FE29355247EEE403587068E455A4053EC",
/// 		"codespace": "",
/// 		"code": 0,
/// 		"data": "0A0B0A094372656174654E796D",
/// 		"raw_log": [{
/// 			"events ": [{
/// 				"type ": "message ",
/// 				"attributes ": [{
/// 					"key ": "action ",
/// 					"value ": "CreateNym "
/// 				}]
/// 			}]
/// 		}],
/// 		"logs": [{
/// 			"msg_index": 0,
/// 			"log": "",
/// 			"events": [{
/// 				"type": "message",
/// 				"attributes": [{
/// 					"key": "action",
/// 					"value": "CreateNym"
/// 				}]
/// 			}]
/// 		}],
/// 		"info": "",
/// 		"gas_wanted": 300000,
/// 		"gas_used": 52848,
/// 		"tx": {
/// 			"type_url": "/cosmos.tx.v1beta1.Tx",
/// 			"value": [10, 145, 1, 10, 134, 1, 10, 37, 47, 99, 104, 101, 113, 100, 105, 100, 46, 99, 104, 101, 113, 100, 110, 111, 100, 101, 46, 99, 104, 101, 113, 100, 46, 77, 115, 103, 67, 114, 101, 97, 116, 101, 78, 121, 109, 18, 93, 10, 45, 99, 111, 115, 109, 111, 115, 49, 120, 51, 51, 120, 107, 106, 100, 51, 103, 113, 108, 102, 104, 122, 53, 108, 57, 104, 54, 48, 109, 53, 51, 112, 114, 50, 109, 100, 100, 52, 121, 51, 110, 99, 56, 54, 104, 48, 18, 10, 116, 101, 115, 116, 95, 97, 108, 105, 97, 115, 26, 11, 116, 101, 115, 116, 95, 118, 101, 114, 107, 101, 121, 34, 8, 116, 101, 115, 116, 95, 100, 105, 100, 42, 9, 116, 101, 115, 116, 95, 114, 111, 108, 101, 18, 4, 109, 101, 109, 111, 24, 52, 18, 97, 10, 78, 10, 70, 10, 31, 47, 99, 111, 115, 109, 111, 115, 46, 99, 114, 121, 112, 116, 111, 46, 115, 101, 99, 112, 50, 53, 54, 107, 49, 46, 80, 117, 98, 75, 101, 121, 18, 35, 10, 33, 2, 59, 126, 95, 52, 102, 213, 99, 251, 102, 62, 148, 101, 72, 226, 188, 243, 222, 31, 35, 148, 19, 127, 79, 75, 79, 37, 160, 132, 193, 33, 148, 7, 18, 4, 10, 2, 8, 1, 18, 15, 10, 9, 10, 4, 99, 104, 101, 113, 18, 1, 48, 16, 224, 167, 18, 26, 64, 1, 225, 116, 194, 154, 244, 148, 8, 209, 8, 174, 61, 108, 6, 39, 116, 111, 218, 47, 116, 88, 255, 47, 247, 235, 37, 91, 162, 57, 189, 40, 227, 81, 132, 215, 23, 63, 222, 4, 15, 25, 23, 227, 183, 91, 125, 75, 61, 151, 211, 195, 174, 194, 110, 10, 206, 153, 85, 166, 178, 8, 252, 146, 123]
/// 		},
/// 		"timestamp": "2021-09-21T08:16:24Z"
/// 	}
/// }
///
/// #Errors
/// Common*
#[no_mangle]
pub extern "C" fn cheqd_ledger_cheqd_parse_query_get_tx_by_hash_resp(
    command_handle: CommandHandle,
    query_resp: *const c_char,
    cb: Option<extern "C" fn(command_handle_: CommandHandle, err: ErrorCode, resp: *const c_char)>,
) -> ErrorCode {
    debug!(
        "cheqd_ledger_cheqd_parse_query_get_tx_by_hash_resp > query_resp {:?}",
        query_resp
    );

    check_useful_c_str!(query_resp, ErrorCode::CommonInvalidParam2);
    check_useful_c_callback!(cb, ErrorCode::CommonInvalidParam3);

    debug!(
        "cheqd_ledger_cheqd_parse_query_get_tx_by_hash_resp > query_resp {:?}",
        query_resp
    );

    let locator = Locator::instance();

    let action = async move {
        let res = locator
            .cheqd_ledger_controller
            .cheqd_parse_query_get_tx_by_hash_resp(&query_resp);
        res
    };

    let cb = move |res: IndyResult<_>| {
        let (err, resp) = prepare_result!(res, String::new());
        debug!(
            "cheqd_ledger_cheqd_parse_query_get_tx_by_hash_resp: resp: {:?}",
            resp
        );
        let resp = ctypes::string_to_cstring(resp);
        cb(command_handle, err, resp.as_ptr())
    };

    locator.executor.spawn_ok_instrumented(
        CommandMetric::CheqdLedgerCommandParseQueryGetTxByHash,
        action,
        cb,
    );

    let res = ErrorCode::Success;
    debug!("cheqd_ledger_cheqd_parse_query_get_tx_by_hash_resp < {:?}", res);
    res
}


/// Build tx for querying tx simulate request
/// #Params
/// command_handle: command handle to map callback to caller context.
/// tx_raw: transaction in raw format. array of bytes
/// tx_len: length of transaction array
/// cb: Callback that takes command result as parameter.
///
/// #Returns
/// Error Code
/// cb:
/// - err: Error code.
///   String of Request like:
/// "{
///     "path":"/cosmos.tx.v1beta1.Service/Simulate",
///     "data":"0A4032363239374435374131464631453443393436324534383944464635353944394632354645443536423231343241323337394444313336414545333146443945",
///     "prove":true
/// }"
/// "data" string - it's a cosmos transaction protobuf.
/// #Errors
/// Common*
#[no_mangle]
pub extern "C" fn cheqd_ledger_tx_build_query_simulate(
    command_handle: CommandHandle,
    tx_raw: *const u8,
    tx_len: u32,
    cb: Option<
        extern "C" fn(
            command_handle_: CommandHandle,
            err: ErrorCode,
            tx_commit_response: *const c_char,
        ),
    >,
) -> ErrorCode {
    debug!(
        "cheqd_ledger_tx_build_query_simulate > tx_raw {:?} tx_len {:?}",
        tx_raw, tx_len
    );

    check_useful_c_byte_array!(
        tx_raw,
        tx_len,
        ErrorCode::CommonInvalidParam2,
        ErrorCode::CommonInvalidParam3
    );
    check_useful_c_callback!(cb, ErrorCode::CommonInvalidParam5);

    debug!(
        "cheqd_ledger_tx_build_query_simulate > tx_raw {:?} tx_len {:?}",
        tx_raw, tx_len
    );

    let locator = Locator::instance();

    let action = async move {
        let res = locator
            .cheqd_ledger_controller
            .tx_build_query_simulate(&tx_raw);
        res
    };

    let cb = move |res: IndyResult<_>| {
        let (err, query) = prepare_result!(res, String::new());
        debug!(
            "cheqd_ledger_tx_build_query_simulate: query: {:?}",
            query
        );

        let query = ctypes::string_to_cstring(query);
        cb(command_handle, err, query.as_ptr())
    };

    locator.executor.spawn_ok_instrumented(
        CommandMetric::CheqdLedgerCommandBuildQueryGas,
        action,
        cb,
    );

    let res = ErrorCode::Success;
    debug!(
        "cheqd_ledger_tx_build_query_simulate < {:?}",
        res
    );
    res
}


/// Parse response for get SimulateResponse
/// #Params
/// command_handle: command handle to map callback to caller context.
/// query_resp: response from ledger with protobuf inside.
/// cb: Callback that takes command result as parameter.
///
/// #Returns
/// Error Code
/// cb:
/// - err: Error code.
/// - JSON string which can be looked like:
/// {
///    "gas_info":{
///       "gas_wanted":300000,
///       "gas_used":52848
///    },
///    "result":{
///       "data":[10, 145, 1, 10, 134, 1, 10, 37, 47, 99, 104, 101, 113, 100, 105, 100, 46, 99, 104, 101, 113, 100, 110, 111, 100, 101, 46, 99, 104, 101, 113, 100, 46, 77, 115, 103, 67, 114, 101, 97, 116, 101, 78, 121, 109, 18, 93, 10, 45, 99, 111, 115, 109, 111, 115, 49, 120, 51, 51, 120, 107, 106, 100, 51, 103, 113, 108, 102, 104, 122, 53, 108, 57, 104, 54, 48, 109, 53, 51, 112, 114, 50, 109, 100, 100, 52, 121, 51, 110, 99, 56, 54, 104, 48, 18, 10, 116, 101, 115, 116, 95, 97, 108, 105, 97, 115, 26, 11, 116, 101, 115, 116, 95, 118, 101, 114, 107, 101, 121, 34, 8, 116, 101, 115, 116, 95, 100, 105, 100, 42, 9, 116, 101, 115, 116, 95, 114, 111, 108, 101, 18, 4, 109, 101, 109, 111, 24, 52, 18, 97, 10, 78, 10, 70, 10, 31, 47, 99, 111, 115, 109, 111, 115, 46, 99, 114, 121, 112, 116, 111, 46, 115, 101, 99, 112, 50, 53, 54, 107, 49, 46, 80, 117, 98, 75, 101, 121, 18, 35, 10, 33, 2, 59, 126, 95, 52, 102, 213, 99, 251, 102, 62, 148, 101, 72, 226, 188, 243, 222, 31, 35, 148, 19, 127, 79, 75, 79, 37, 160, 132, 193, 33, 148, 7, 18, 4, 10, 2, 8, 1, 18, 15, 10, 9, 10, 4, 99, 104, 101, 113, 18, 1, 48, 16, 224, 167, 18, 26, 64, 1, 225, 116, 194, 154, 244, 148, 8, 209, 8, 174, 61, 108, 6, 39, 116, 111, 218, 47, 116, 88, 255, 47, 247, 235, 37, 91, 162, 57, 189, 40, 227, 81, 132, 215, 23, 63, 222, 4, 15, 25, 23, 227, 183, 91, 125, 75, 61, 151, 211, 195, 174, 194, 110, 10, 206, 153, 85, 166, 178, 8, 252, 146, 123],
///       "log":"",
///       "events":[
///          {
///             "r#type":"message",
///             "attributes":[
///                {
///                   "key": [10, 145, 1, 10, 134, 1, 10, 37, 47, 99, 104, 101, 113, 100, 105, 100, 46, 99, 104, 101, 113, 100, 110, 111, 100, 101, 46, 99, 104, 101, 113, 100, 46, 77, 115, 103, 67, 114, 101, 97, 116, 101, 78, 121, 109, 18, 93, 10, 45, 99, 111, 115, 109, 111, 115, 49, 120, 51, 51, 120, 107, 106, 100, 51, 103, 113, 108, 102, 104, 122, 53, 108, 57, 104, 54, 48, 109, 53, 51, 112, 114, 50, 109, 100, 100, 52, 121, 51, 110, 99, 56, 54, 104, 48, 18, 10, 116, 101, 115, 116, 95, 97, 108, 105, 97, 115, 26, 11, 116, 101, 115, 116, 95, 118, 101, 114, 107, 101, 121, 34, 8, 116, 101, 115, 116, 95, 100, 105, 100, 42, 9, 116, 101, 115, 116, 95, 114, 111, 108, 101, 18, 4, 109, 101, 109, 111, 24, 52, 18, 97, 10, 78, 10, 70, 10, 31, 47, 99, 111, 115, 109, 111, 115, 46, 99, 114, 121, 112, 116, 111, 46, 115, 101, 99, 112, 50, 53, 54, 107, 49, 46, 80, 117, 98, 75, 101, 121, 18, 35, 10, 33, 2, 59, 126, 95, 52, 102, 213, 99, 251, 102, 62, 148, 101, 72, 226, 188, 243, 222, 31, 35, 148, 19, 127, 79, 75, 79, 37, 160, 132, 193, 33, 148, 7, 18, 4, 10, 2, 8, 1, 18, 15, 10, 9, 10, 4, 99, 104, 101, 113, 18, 1, 48, 16, 224, 167, 18, 26, 64, 1, 225, 116, 194, 154, 244, 148, 8, 209, 8, 174, 61, 108, 6, 39, 116, 111, 218, 47, 116, 88, 255, 47, 247, 235, 37, 91, 162, 57, 189, 40, 227, 81, 132, 215, 23, 63, 222, 4, 15, 25, 23, 227, 183, 91, 125, 75, 61, 151, 211, 195, 174, 194, 110, 10, 206, 153, 85, 166, 178, 8, 252, 146, 123],
///                   "value":[10, 145, 1, 10, 134, 1, 10, 37, 47, 99, 104, 101, 113, 100, 105, 100, 46, 99, 104, 101, 113, 100, 110, 111, 100, 101, 46, 99, 104, 101, 113, 100, 46, 77, 115, 103, 67, 114, 101, 97, 116, 101, 78, 121, 109, 18, 93, 10, 45, 99, 111, 115, 109, 111, 115, 49, 120, 51, 51, 120, 107, 106, 100, 51, 103, 113, 108, 102, 104, 122, 53, 108, 57, 104, 54, 48, 109, 53, 51, 112, 114, 50, 109, 100, 100, 52, 121, 51, 110, 99, 56, 54, 104, 48, 18, 10, 116, 101, 115, 116, 95, 97, 108, 105, 97, 115, 26, 11, 116, 101, 115, 116, 95, 118, 101, 114, 107, 101, 121, 34, 8, 116, 101, 115, 116, 95, 100, 105, 100, 42, 9, 116, 101, 115, 116, 95, 114, 111, 108, 101, 18, 4, 109, 101, 109, 111, 24, 52, 18, 97, 10, 78, 10, 70, 10, 31, 47, 99, 111, 115, 109, 111, 115, 46, 99, 114, 121, 112, 116, 111, 46, 115, 101, 99, 112, 50, 53, 54, 107, 49, 46, 80, 117, 98, 75, 101, 121, 18, 35, 10, 33, 2, 59, 126, 95, 52, 102, 213, 99, 251, 102, 62, 148, 101, 72, 226, 188, 243, 222, 31, 35, 148, 19, 127, 79, 75, 79, 37, 160, 132, 193, 33, 148, 7, 18, 4, 10, 2, 8, 1, 18, 15, 10, 9, 10, 4, 99, 104, 101, 113, 18, 1, 48, 16, 224, 167, 18, 26, 64, 1, 225, 116, 194, 154, 244, 148, 8, 209, 8, 174, 61, 108, 6, 39, 116, 111, 218, 47, 116, 88, 255, 47, 247, 235, 37, 91, 162, 57, 189, 40, 227, 81, 132, 215, 23, 63, 222, 4, 15, 25, 23, 227, 183, 91, 125, 75, 61, 151, 211, 195, 174, 194, 110, 10, 206, 153, 85, 166, 178, 8, 252, 146, 123]
///                }
///             ]
///          }
///       ],
///       "index":false
///    }
/// }
///
/// #Errors
/// Common*
#[no_mangle]
pub extern "C" fn cheqd_ledger_tx_parse_query_simulate_resp(
    command_handle: CommandHandle,
    query_resp: *const c_char,
    cb: Option<extern "C" fn(command_handle_: CommandHandle, err: ErrorCode, resp: *const c_char)>,
) -> ErrorCode {
    debug!(
        "cheqd_ledger_tx_parse_query_simulate_resp > query_resp {:?}",
        query_resp
    );

    check_useful_c_str!(query_resp, ErrorCode::CommonInvalidParam2);
    check_useful_c_callback!(cb, ErrorCode::CommonInvalidParam3);

    debug!(
        "cheqd_ledger_tx_parse_query_simulate_resp > query_resp {:?}",
        query_resp
    );

    let locator = Locator::instance();

    let action = async move {
        let res = locator
            .cheqd_ledger_controller
            .tx_parse_query_simulate_resp(&query_resp);
        res
    };

    let cb = move |res: IndyResult<_>| {
        let (err, resp) = prepare_result!(res, String::new());
        debug!(
            "cheqd_ledger_tx_parse_query_simulate_resp: resp: {:?}",
            resp
        );
        let resp = ctypes::string_to_cstring(resp);
        cb(command_handle, err, resp.as_ptr())
    };

    locator.executor.spawn_ok_instrumented(
        CommandMetric::CheqdLedgerCommandParseQueryGasResp,
        action,
        cb,
    );

    let res = ErrorCode::Success;
    debug!(
        "cheqd_ledger_tx_parse_query_simulate_resp < {:?}",
        res
    );
    res
}
