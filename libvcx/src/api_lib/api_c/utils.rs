use std::ptr;

use futures::future::{BoxFuture, FutureExt};
use libc::c_char;
use serde_json;

use aries_vcx::agency_client::configuration::AgentProvisionConfig;
use aries_vcx::agency_client::messages::update_message::UIDsByConn;
use aries_vcx::agency_client::testing::mocking::AgencyMock;
use aries_vcx::agency_client::MessageStatusCode;
use aries_vcx::error::{VcxError, VcxErrorKind};
use aries_vcx::indy_sys::CommandHandle;
use aries_vcx::utils::constants::*;
use aries_vcx::utils::error;
use aries_vcx::global::settings;
use aries_vcx::global::pool::get_main_pool_handle;

use crate::api_lib::api_handle::connection;
use crate::api_lib::api_handle::connection::{parse_connection_handles, parse_status_codes};
use crate::api_lib::api_handle::utils::agency_update_messages;
use crate::api_lib::global::agency_client::get_main_agency_client;
use crate::api_lib::global::wallet::get_main_wallet_handle;
use crate::api_lib::utils::cstring::CStringUtils;
use crate::api_lib::utils::error::{set_current_error, set_current_error_vcx};
use crate::api_lib::utils::runtime::execute_async;

/// Provision an agent in the agency.
///
/// #Params
/// command_handle: command handle to map callback to user context.
///
/// agency_config: agency config as a string
///
/// cb: Callback that provides agency configuration or error status
///
/// #Example input agency config ->
/// {
///  "agency_did": "VsKV7grR1BUE29mG2Fm2kX",
///  "agency_endpoint": "http://127.0.0.1:8080",
///  "agency_verkey": "Hezce2UWMZ3wUhVkh2LfKSs8nDzWwzs2Win7EzNN3YaR",
///  "agent_seed": "000000000000000000Aliceagentseed" // OPTIONAL
/// }
///
/// #Example output agency configuration ->
/// {
///  "agency_did": "VsKV7grR1BUE29mG2Fm2kX",
///  "agency_endpoint": "http://127.0.0.1:8080",
///  "agency_verkey": "Hezce2UWMZ3wUhVkh2LfKSs8nDzWwzs2Win7EzNN3YaR",
///  "remote_to_sdk_did": "GkdUhwyWqNw3vGs6FQFFHb",
///  "remote_to_sdk_verkey": "9axcTwXeJ1haJBw9LqexT8dRpiFCJwA6ZUevM5nfiDKg",
///  "sdk_to_remote_did": "C5DiHD1n3MqNcv5h7PBK9J",
///  "sdk_to_remote_verkey": "732pD7kDiBjSyS57aNXi52Xpg2DLCTb43aLpddo2X8CG"
/// }
///
/// #Returns
/// Error code as a u32
#[no_mangle]
pub extern "C" fn vcx_provision_cloud_agent(
    command_handle: CommandHandle,
    agency_config: *const c_char,
    cb: Option<extern "C" fn(xcommand_handle: CommandHandle, err: u32, config: *const c_char)>,
) -> u32 {
    info!("vcx_provision_cloud_agent >>>");

    check_useful_c_callback!(cb, VcxErrorKind::InvalidOption);
    check_useful_c_str!(agency_config, VcxErrorKind::InvalidOption);

    trace!(
        "vcx_provision_cloud_agent(command_handle: {}, agency_config: {})",
        command_handle,
        agency_config
    );

    let agency_config = match serde_json::from_str::<AgentProvisionConfig>(&agency_config) {
        Ok(agency_config) => agency_config,
        Err(err) => {
            error!(
                "vcx_provision_cloud_agent >>> invalid agency configuration; err: {:?}",
                err
            );
            return error::INVALID_CONFIGURATION.code_num;
        }
    };

    execute_async::<BoxFuture<'static, Result<(), ()>>>(
        async move {
            match aries_vcx::utils::provision::provision_cloud_agent(
                &mut get_main_agency_client().unwrap(),
                get_main_wallet_handle(),
                &agency_config,
            )
            .await
            {
                Err(err) => {
                    set_current_error_vcx(&err);
                    error!(
                        "vcx_provision_cloud_agent_cb(command_handle: {}, rc: {}, config: NULL",
                        command_handle, err
                    );
                    cb(command_handle, err.into(), ptr::null_mut());
                }
                Ok(agency_config) => {
                    let agency_config = json!(&agency_config).to_string();
                    trace!(
                        "vcx_provision_cloud_agent_cb(command_handle: {}, rc: {}, config: {})",
                        command_handle,
                        error::SUCCESS.message,
                        agency_config
                    );
                    let msg = CStringUtils::string_to_cstring(agency_config);
                    cb(command_handle, 0, msg.as_ptr());
                }
            }
            Ok(())
        }
        .boxed(),
    );

    error::SUCCESS.code_num
}

#[no_mangle]
pub extern "C" fn vcx_set_next_agency_response(message_index: u32) {
    info!("vcx_set_next_agency_response >>>");

    let message = match message_index {
        4 => UPDATE_CREDENTIAL_RESPONSE.to_vec(),
        5 => UPDATE_PROOF_RESPONSE.to_vec(),
        6 => CREDENTIAL_REQ_RESPONSE.to_vec(),
        7 => PROOF_RESPONSE.to_vec(),
        8 => CREDENTIAL_RESPONSE.to_vec(),
        9 => GET_MESSAGES_INVITE_ACCEPTED_RESPONSE.to_vec(),
        _ => Vec::new(),
    };

    AgencyMock::set_next_response(message);
}

/// Retrieve messages from the agent
///
/// #params
///
/// command_handle: command handle to map callback to user context.
///
/// pw_dids: comma separated connection handles
///
/// message_status: optional, comma separated -  - query for messages with the specified status.
///                            Statuses:
///                                 MS-101 - Created
///                                 MS-102 - Sent
///                                 MS-103 - Received
///                                 MS-104 - Accepted
///                                 MS-105 - Rejected
///                                 MS-106 - Reviewed
///
/// uids: optional, comma separated - query for messages with the specified uids
///
/// cb: Callback that provides array of matching messages retrieved
///
/// # Example message_status -> MS-103, MS-106
///
/// # Example uids -> s82g63, a2h587
///
/// # Example pw_dids -> did1, did2
///
/// # Example messages -> "[{"pairwiseDID":"did","msgs":[{"statusCode":"MS-106","payload":null,"senderDID":"","uid":"6BDkgc3z0E","type":"aries","refMsgId":null,"deliveryDetails":[],"decryptedPayload":"{"@msg":".....","@type":{"fmt":"json","name":"aries","ver":"1.0"}}"}]}]"
///
/// #Returns
/// Error code as a u32
#[no_mangle]
#[deprecated(since = "0.20.0", note = "Deprecated in favor of vcx_connection_messages_download.")]
pub extern "C" fn vcx_v2_messages_download(
    command_handle: CommandHandle,
    conn_handles: *const c_char,
    message_statuses: *const c_char,
    uids: *const c_char,
    cb: Option<extern "C" fn(xcommand_handle: CommandHandle, err: u32, messages: *const c_char)>,
) -> u32 {
    info!("vcx_v2_messages_download >>>");

    check_useful_c_callback!(cb, VcxErrorKind::InvalidOption);

    let conn_handles = if !conn_handles.is_null() {
        check_useful_c_str!(conn_handles, VcxErrorKind::InvalidOption);
        let v: Vec<&str> = conn_handles.split(',').collect();
        let v = v.iter().map(|s| s.to_string()).collect::<Vec<String>>();
        v.to_owned()
    } else {
        return VcxError::from_msg(VcxErrorKind::InvalidJson, "List of connection handles can't be null").into();
    };

    let conn_handles = match parse_connection_handles(conn_handles) {
        Ok(handles) => handles,
        Err(err) => return err.into(),
    };

    let message_statuses = if !message_statuses.is_null() {
        check_useful_c_str!(message_statuses, VcxErrorKind::InvalidOption);
        let v: Vec<&str> = message_statuses.split(',').collect();
        let v = v.iter().map(|s| s.to_string()).collect::<Vec<String>>();
        Some(v.to_owned())
    } else {
        None
    };

    let message_statuses = match parse_status_codes(message_statuses) {
        Ok(statuses) => statuses,
        Err(_err) => return VcxError::from(VcxErrorKind::InvalidConnectionHandle).into(),
    };

    let uids = if !uids.is_null() {
        check_useful_c_str!(uids, VcxErrorKind::InvalidOption);
        let v: Vec<&str> = uids.split(',').collect();
        let v = v.iter().map(|s| s.to_string()).collect::<Vec<String>>();
        Some(v.to_owned())
    } else {
        None
    };

    trace!(
        "vcx_v2_messages_download(command_handle: {}, message_statuses: {:?}, uids: {:?})",
        command_handle,
        message_statuses,
        uids
    );

    execute_async::<BoxFuture<'static, Result<(), ()>>>(
        async move {
            match connection::download_messages(conn_handles, message_statuses, uids).await {
                Ok(err) => {
                    match serde_json::to_string(&err) {
                        Ok(err) => {
                            trace!(
                                "vcx_v2_messages_download_cb(command_handle: {}, rc: {}, messages: {})",
                                command_handle,
                                error::SUCCESS.message,
                                err
                            );

                            let msg = CStringUtils::string_to_cstring(err);
                            cb(command_handle, error::SUCCESS.code_num, msg.as_ptr());
                        }
                        Err(err) => {
                            let err = VcxError::from_msg(
                                VcxErrorKind::InvalidJson,
                                format!("Cannot serialize messages: {}", err),
                            );
                            error!(
                                "vcx_v2_messages_download_cb(command_handle: {}, rc: {}, messages: {})",
                                command_handle, err, "null"
                            );

                            cb(command_handle, err.into(), ptr::null_mut());
                        }
                    };
                }
                Err(err) => {
                    set_current_error_vcx(&err);
                    error!(
                        "vcx_v2_messages_download_cb(command_handle: {}, rc: {}, messages: {})",
                        command_handle, err, "null"
                    );

                    cb(command_handle, err.into(), ptr::null_mut());
                }
            };

            Ok(())
        }
        .boxed(),
    );

    error::SUCCESS.code_num
}

/// Update the status of messages from the specified connection
///
/// #params
///
/// command_handle: command handle to map callback to user context.
///
/// message_status: target message status
///                 Statuses:
///                     MS-101 - Created
///                     MS-102 - Sent
///                     MS-103 - Received
///                     MS-104 - Accepted
///                     MS-105 - Rejected
///                     MS-106 - Reviewed
///
/// msg_json: messages to update: [{"pairwiseDID":"QSrw8hebcvQxiwBETmAaRs","uids":["mgrmngq"]},...]
///
/// cb: Callback that provides success or failure of request
///
/// #Returns
/// Error code as a u32
#[no_mangle]
pub extern "C" fn vcx_messages_update_status(
    command_handle: CommandHandle,
    message_status: *const c_char,
    msg_json: *const c_char,
    cb: Option<extern "C" fn(xcommand_handle: CommandHandle, err: u32)>,
) -> u32 {
    info!("vcx_messages_update_status >>>");

    check_useful_c_callback!(cb, VcxErrorKind::InvalidOption);
    check_useful_c_str!(message_status, VcxErrorKind::InvalidOption);
    check_useful_c_str!(msg_json, VcxErrorKind::InvalidOption);

    trace!(
        "vcx_messages_set_status(command_handle: {}, message_status: {:?}, uids: {:?})",
        command_handle,
        message_status,
        msg_json
    );

    let status_code: MessageStatusCode = match ::serde_json::from_str(&format!("\"{}\"", message_status)) {
        Ok(status_code) => status_code,
        Err(err) => {
            set_current_error(&err);
            error!(
                "vcx_messages_update_status >>> Cannot deserialize MessageStatusCode: {:?}",
                err
            );
            return error::INVALID_CONFIGURATION.code_num;
        }
    };

    let uids_by_conns: Vec<UIDsByConn> = match serde_json::from_str(&msg_json) {
        Ok(status_code) => status_code,
        Err(err) => {
            set_current_error(&err);
            error!(
                "vcx_messages_update_status >>> Cannot deserialize UIDsByConn: {:?}",
                err
            );
            return error::INVALID_CONFIGURATION.code_num;
        }
    };

    execute_async::<BoxFuture<'static, Result<(), ()>>>(
        async move {
            match agency_update_messages(status_code, uids_by_conns).await {
                Ok(()) => {
                    trace!(
                        "vcx_messages_set_status_cb(command_handle: {}, rc: {})",
                        command_handle,
                        error::SUCCESS.message
                    );

                    cb(command_handle, error::SUCCESS.code_num);
                }
                Err(err) => {
                    error!(
                        "vcx_messages_set_status_cb(command_handle: {}, rc: {})",
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

/// Set the pool handle before calling vcx_init_minimal
///
/// #params
///
/// handle: pool handle that libvcx should use
///
/// #Returns
/// Error code as u32
#[no_mangle]
pub extern "C" fn vcx_pool_set_handle(handle: i32) -> i32 {
    if handle <= 0 {
        aries_vcx::global::pool::set_main_pool_handle(None);
    } else {
        aries_vcx::global::pool::set_main_pool_handle(Some(handle));
    }

    handle
}

/// Endorse transaction to the ledger preserving an original author
///
/// #Params
/// command_handle: command handle to map callback to user context.
/// transaction: transaction to endorse
///
/// cb: Callback that provides success or failure of command
///
/// #Returns
/// Error code as a u32
#[no_mangle]
pub extern "C" fn vcx_endorse_transaction(
    command_handle: CommandHandle,
    transaction: *const c_char,
    cb: Option<extern "C" fn(xcommand_handle: CommandHandle, err: u32)>,
) -> u32 {
    info!("vcx_endorse_transaction >>>");

    check_useful_c_str!(transaction, VcxErrorKind::InvalidOption);
    check_useful_c_callback!(cb, VcxErrorKind::InvalidOption);
    let pool_handle = match get_main_pool_handle() {
        Ok(handle) => handle,
        Err(err) => return err.into(),
    };
    let issuer_did: String = match settings::get_config_value(settings::CONFIG_INSTITUTION_DID) {
        Ok(err) => err,
        Err(err) => return err.into(),
    };
    trace!(
        "vcx_endorse_transaction(command_handle: {}, pool_handle: {:?}, issuer_did: {}, transaction: {})",
        command_handle,
        pool_handle,
        issuer_did,
        transaction
    );

    execute_async::<BoxFuture<'static, Result<(), ()>>>(Box::pin(async move {
        match aries_vcx::libindy::utils::ledger::endorse_transaction(get_main_wallet_handle(), pool_handle, &issuer_did, &transaction).await {
            Ok(()) => {
                trace!(
                    "vcx_endorse_transaction(command_handle: {}, issuer_did: {}, rc: {})",
                    command_handle,
                    issuer_did,
                    error::SUCCESS.message
                );

                cb(command_handle, error::SUCCESS.code_num);
            }
            Err(err) => {
                error!(
                    "vcx_endorse_transaction(command_handle: {}, issuer_did: {}, rc: {})",
                    command_handle, issuer_did, err
                );

                cb(command_handle, err.into());
            }
        };

        Ok(())
    }));

    error::SUCCESS.code_num
}

#[no_mangle]
pub extern "C" fn vcx_rotate_verkey(
    command_handle: CommandHandle,
    did: *const c_char,
    cb: Option<extern "C" fn(xcommand_handle: CommandHandle, err: u32)>,
) -> u32 {
    info!("vcx_rotate_verkey >>>");

    check_useful_c_str!(did, VcxErrorKind::InvalidOption);
    check_useful_c_callback!(cb, VcxErrorKind::InvalidOption);
    trace!("vcx_rotate_verkey(command_handle: {}, did: {})", command_handle, did);

    let pool_handle = match get_main_pool_handle() {
        Ok(handle) => handle,
        Err(err) => return err.into(),
    };

    execute_async::<BoxFuture<'static, Result<(), ()>>>(Box::pin(async move {
        match aries_vcx::libindy::utils::signus::rotate_verkey(get_main_wallet_handle(), pool_handle, &did).await {
            Ok(()) => {
                trace!(
                    "vcx_rotate_verkey_cb(command_handle: {}, rc: {})",
                    command_handle,
                    error::SUCCESS.message
                );
                cb(command_handle, error::SUCCESS.code_num);
            }
            Err(err) => {
                error!("vcx_rotate_verkey_cb(command_handle: {}, rc: {})", command_handle, err);

                cb(command_handle, err.into());
            }
        };

        Ok(())
    }));

    error::SUCCESS.code_num
}

#[no_mangle]
pub extern "C" fn vcx_rotate_verkey_start(
    command_handle: CommandHandle,
    did: *const c_char,
    cb: Option<extern "C" fn(xcommand_handle: CommandHandle, err: u32, temp_vk: *const c_char)>,
) -> u32 {
    info!("vcx_rotate_verkey_start >>>");

    check_useful_c_str!(did, VcxErrorKind::InvalidOption);
    check_useful_c_callback!(cb, VcxErrorKind::InvalidOption);
    trace!(
        "vcx_rotate_verkey_start(command_handle: {}, did: {})",
        command_handle,
        did
    );

    execute_async::<BoxFuture<'static, Result<(), ()>>>(Box::pin(async move {
        match aries_vcx::libindy::utils::signus::libindy_replace_keys_start(get_main_wallet_handle(), &did).await {
            Ok(temp_vk) => {
                trace!(
                    "vcx_rotate_verkey_start_cb(command_handle: {}, rc: {}, temp_vk: {})",
                    command_handle,
                    error::SUCCESS.message,
                    temp_vk
                );
                let temp_vk = CStringUtils::string_to_cstring(temp_vk);
                cb(command_handle, error::SUCCESS.code_num, temp_vk.as_ptr());
            }
            Err(err) => {
                error!(
                    "vcx_rotate_verkey_start_cb(command_handle: {}, rc: {})",
                    command_handle, err
                );

                cb(command_handle, err.into(), ptr::null_mut());
            }
        };

        Ok(())
    }));

    error::SUCCESS.code_num
}

#[no_mangle]
pub extern "C" fn vcx_rotate_verkey_apply(
    command_handle: CommandHandle,
    did: *const c_char,
    temp_vk: *const c_char,
    cb: Option<extern "C" fn(xcommand_handle: CommandHandle, err: u32)>,
) -> u32 {
    info!("vcx_rotate_verkey_apply >>>");

    check_useful_c_str!(did, VcxErrorKind::InvalidOption);
    check_useful_c_str!(temp_vk, VcxErrorKind::InvalidOption);
    check_useful_c_callback!(cb, VcxErrorKind::InvalidOption);
    trace!(
        "vcx_rotate_verkey_apply(command_handle: {}, did: {}, temp_vk: {:?})",
        command_handle,
        did,
        temp_vk
    );

    let pool_handle = match get_main_pool_handle() {
        Ok(handle) => handle,
        Err(err) => return err.into(),
    };

    execute_async::<BoxFuture<'static, Result<(), ()>>>(Box::pin(async move {
        match aries_vcx::libindy::utils::signus::rotate_verkey_apply(get_main_wallet_handle(), pool_handle, &did, &temp_vk).await {
            Ok(()) => {
                trace!(
                    "vcx_rotate_verkey_apply_cb(command_handle: {}, rc: {})",
                    command_handle,
                    error::SUCCESS.message
                );
                cb(command_handle, error::SUCCESS.code_num);
            }
            Err(err) => {
                error!(
                    "vcx_rotate_verkey_apply_cb(command_handle: {}, rc: {})",
                    command_handle, err
                );

                cb(command_handle, err.into());
            }
        };

        Ok(())
    }));

    error::SUCCESS.code_num
}

#[no_mangle]
pub extern "C" fn vcx_get_verkey_from_wallet(
    command_handle: CommandHandle,
    did: *const c_char,
    cb: Option<extern "C" fn(xcommand_handle: CommandHandle, err: u32, verkey: *const c_char)>,
) -> u32 {
    info!("vcx_get_verkey_from_wallet >>>");

    check_useful_c_str!(did, VcxErrorKind::InvalidOption);
    check_useful_c_callback!(cb, VcxErrorKind::InvalidOption);
    trace!(
        "vcx_get_verkey_from_wallet(command_handle: {}, did: {})",
        command_handle,
        did
    );

    execute_async::<BoxFuture<'static, Result<(), ()>>>(Box::pin(async move {
        match aries_vcx::libindy::utils::signus::get_verkey_from_wallet(get_main_wallet_handle(), &did).await {
            Ok(verkey) => {
                trace!(
                    "vcx_get_verkey_from_wallet_cb(command_handle: {}, rc: {}, verkey: {})",
                    command_handle,
                    error::SUCCESS.message,
                    verkey
                );
                let verkey = CStringUtils::string_to_cstring(verkey);
                cb(command_handle, error::SUCCESS.code_num, verkey.as_ptr());
            }
            Err(err) => {
                error!(
                    "vcx_get_verkey_from_wallet_cb(command_handle: {}, rc: {})",
                    command_handle, err
                );

                cb(command_handle, err.into(), ptr::null_mut());
            }
        };

        Ok(())
    }));

    error::SUCCESS.code_num
}

#[no_mangle]
pub extern "C" fn vcx_get_verkey_from_ledger(
    command_handle: CommandHandle,
    did: *const c_char,
    cb: Option<extern "C" fn(xcommand_handle: CommandHandle, err: u32, verkey: *const c_char)>,
) -> u32 {
    info!("vcx_get_verkey_from_ledger >>>");

    check_useful_c_str!(did, VcxErrorKind::InvalidOption);
    check_useful_c_callback!(cb, VcxErrorKind::InvalidOption);
    trace!(
        "vcx_get_verkey_from_ledger(command_handle: {}, did: {})",
        command_handle,
        did
    );

    let pool_handle = match get_main_pool_handle() {
        Ok(handle) => handle,
        Err(err) => return err.into(),
    };

    execute_async::<BoxFuture<'static, Result<(), ()>>>(Box::pin(async move {
        match aries_vcx::libindy::utils::signus::get_verkey_from_ledger(pool_handle, &did).await {
            Ok(verkey) => {
                trace!(
                    "vcx_get_verkey_from_ledger_cb(command_handle: {}, rc: {}, verkey: {})",
                    command_handle,
                    error::SUCCESS.message,
                    verkey
                );
                let verkey = CStringUtils::string_to_cstring(verkey);
                cb(command_handle, error::SUCCESS.code_num, verkey.as_ptr());
            }
            Err(err) => {
                error!(
                    "vcx_get_verkey_from_ledger_cb(command_handle: {}, rc: {})",
                    command_handle, err
                );

                cb(command_handle, err.into(), ptr::null_mut());
            }
        };

        Ok(())
    }));

    error::SUCCESS.code_num
}

#[no_mangle]
pub extern "C" fn vcx_get_ledger_txn(
    command_handle: CommandHandle,
    submitter_did: *const c_char,
    seq_no: i32,
    cb: Option<extern "C" fn(xcommand_handle: CommandHandle, err: u32, txn: *const c_char)>,
) -> u32 {
    info!("vcx_get_ledger_txn >>>");

    check_useful_opt_c_str!(submitter_did, VcxErrorKind::InvalidOption);
    check_useful_c_callback!(cb, VcxErrorKind::InvalidOption);
    trace!(
        "vcx_get_ledger_txn(command_handle: {}, submitter_did: {:?})",
        command_handle,
        submitter_did
    );

    let pool_handle = match get_main_pool_handle() {
        Ok(handle) => handle,
        Err(err) => return err.into(),
    };

    execute_async::<BoxFuture<'static, Result<(), ()>>>(Box::pin(async move {
        match aries_vcx::libindy::utils::anoncreds::get_ledger_txn(
            get_main_wallet_handle(),
            pool_handle,
            submitter_did.as_deref(),
            seq_no,
        )
        .await
        {
            Ok(txn) => {
                trace!(
                    "vcx_get_ledger_txn_cb(command_handle: {}, rc: {}, txn: {})",
                    command_handle,
                    error::SUCCESS.message,
                    txn
                );
                let txn = CStringUtils::string_to_cstring(txn);
                cb(command_handle, error::SUCCESS.code_num, txn.as_ptr());
            }
            Err(err) => {
                error!("vcx_get_ledger_txn_cb(command_handle: {}, rc: {})", command_handle, err);

                cb(command_handle, err.into(), ptr::null_mut());
            }
        };

        Ok(())
    }));

    error::SUCCESS.code_num
}

#[cfg(test)]
mod tests {
    use std::ffi::CString;

    use aries_vcx::agency_client::configuration::AgentProvisionConfig;
    use aries_vcx::agency_client::testing::mocking::AgencyMockDecrypted;
    use aries_vcx::utils::constants;
    use aries_vcx::utils::devsetup::SetupMocks;

    use crate::api_lib::utils::return_types_u32;
    use crate::api_lib::utils::timeout::TimeoutUtils;

    use super::*;

    fn _vcx_agent_provision_async_c_closure(config: &str) -> Result<Option<String>, u32> {
        let cb = return_types_u32::Return_U32_STR::new().unwrap();
        let rc = vcx_provision_cloud_agent(
            cb.command_handle,
            CString::new(config).unwrap().into_raw(),
            Some(cb.get_callback()),
        );
        if rc != error::SUCCESS.code_num {
            return Err(rc);
        }
        cb.receive(TimeoutUtils::some_short())
    }

    #[test]
    #[cfg(feature = "general_test")]
    fn test_provision_agent_async_c_closure() {
        let _setup = SetupMocks::init();

        let config = AgentProvisionConfig {
            agency_did: "Ab8TvZa3Q19VNkQVzAWVL7".into(),
            agency_verkey: "5LXaR43B1aQyeh94VBP8LG1Sgvjk7aNfqiksBCSjwqbf".into(),
            agency_endpoint: "https://enym-eagency.pdev.evernym.com".into(),
            agent_seed: None,
        };
        let result = _vcx_agent_provision_async_c_closure(&json!(config).to_string()).unwrap();
        let _config: serde_json::Value = serde_json::from_str(&result.unwrap()).unwrap();
    }

    #[test]
    #[cfg(feature = "general_test")]
    fn test_create_agent_fails_if_missing_agency_endpoint() {
        let _setup = SetupMocks::init();

        let config = json!({
            "agency_did":"Ab8TvZa3Q19VNkQVzAWVL7",
            "agency_verkey":"5LXaR43B1aQyeh94VBP8LG1Sgvjk7aNfqiksBCSjwqbf"
        })
        .to_string();

        let err = _vcx_agent_provision_async_c_closure(&config).unwrap_err();
        assert_eq!(err, error::INVALID_CONFIGURATION.code_num);
    }

    #[test]
    #[cfg(feature = "general_test")]
    fn test_messages_update_status() {
        let _setup = SetupMocks::init();

        AgencyMockDecrypted::set_next_decrypted_response(constants::GET_MESSAGES_DECRYPTED_RESPONSE);

        let status = CString::new("MS-103").unwrap().into_raw();
        let json = CString::new(r#"[{"pairwiseDID":"QSrw8hebcvQxiwBETmAaRs","uids":["mgrmngq"]}]"#)
            .unwrap()
            .into_raw();

        let cb = return_types_u32::Return_U32::new().unwrap();
        assert_eq!(
            vcx_messages_update_status(cb.command_handle, status, json, Some(cb.get_callback())),
            error::SUCCESS.code_num
        );
        cb.receive(TimeoutUtils::some_medium()).unwrap();
    }
}
