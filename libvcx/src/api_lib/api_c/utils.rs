use std::ptr;
use std::thread;

use libc::c_char;
use serde_json;
use futures::future::{FutureExt, BoxFuture};
use futures::executor::block_on;

use aries_vcx::agency_client::get_message::{parse_connection_handles, parse_status_codes};
use aries_vcx::agency_client::mocking::AgencyMock;
use aries_vcx::indy_sys::CommandHandle;
use aries_vcx::utils::constants::*;
use aries_vcx::utils::error;
use aries_vcx::utils::provision::AgentProvisionConfig;

use crate::api_lib::api_handle::connection;
use crate::api_lib::utils::cstring::CStringUtils;
use crate::api_lib::utils::runtime::{execute, execute_async};
use crate::error::prelude::*;

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
pub extern fn vcx_provision_cloud_agent(command_handle: CommandHandle,
                                        agency_config: *const c_char,
                                        cb: Option<extern fn(xcommand_handle: CommandHandle, err: u32, config: *const c_char)>) -> u32 {
    info!("vcx_provision_cloud_agent >>>");

    check_useful_c_callback!(cb, VcxErrorKind::InvalidOption);
    check_useful_c_str!(agency_config, VcxErrorKind::InvalidOption);

    trace!("vcx_provision_cloud_agent(command_handle: {}, agency_config: {})", command_handle, agency_config);

    let agency_config = match serde_json::from_str::<AgentProvisionConfig>(&agency_config) {
        Ok(agency_config) => agency_config,
        Err(err) => {
            error!("vcx_provision_cloud_agent >>> invalid agency configuration; err: {:?}", err);
            return error::INVALID_CONFIGURATION.code_num;
        }
    };

    thread::spawn(move || {
        match block_on(aries_vcx::utils::provision::provision_cloud_agent(&agency_config)) {
            Err(e) => {
                error!("vcx_provision_cloud_agent_cb(command_handle: {}, rc: {}, config: NULL", command_handle, e);
                cb(command_handle, e.into(), ptr::null_mut());
            }
            Ok(agency_config) => {
                let agency_config = serde_json::to_string(&agency_config).unwrap();
                // todo: no unwrap
                trace!("vcx_provision_cloud_agent_cb(command_handle: {}, rc: {}, config: {})",
                       command_handle, error::SUCCESS.message, agency_config);
                let msg = CStringUtils::string_to_cstring(agency_config);
                cb(command_handle, 0, msg.as_ptr());
            }
        }
    });

    error::SUCCESS.code_num
}

#[no_mangle]
pub extern fn vcx_set_next_agency_response(message_index: u32) {
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
/// pw_dids: optional, comma separated - DID's pointing to specific connection
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
#[deprecated(since = "0.12.0", note = "This is dangerous because downloaded messages are not \
authenticated and a message appearing to be received from certain connection might have been spoofed. \
Use vcx_connection_messages_download instead.")]
pub extern fn vcx_messages_download(command_handle: CommandHandle,
                                    message_status: *const c_char,
                                    uids: *const c_char,
                                    pw_dids: *const c_char,
                                    cb: Option<extern fn(xcommand_handle: CommandHandle, err: u32, messages: *const c_char)>) -> u32 {
    info!("vcx_messages_download >>>");

    check_useful_c_callback!(cb, VcxErrorKind::InvalidOption);

    let message_status = if !message_status.is_null() {
        check_useful_c_str!(message_status, VcxErrorKind::InvalidOption);
        let v: Vec<&str> = message_status.split(',').collect();
        let v = v.iter().map(|s| s.to_string()).collect::<Vec<String>>();
        Some(v.to_owned())
    } else {
        None
    };

    let uids = if !uids.is_null() {
        check_useful_c_str!(uids, VcxErrorKind::InvalidOption);
        let v: Vec<&str> = uids.split(',').collect();
        let v = v.iter().map(|s| s.to_string()).collect::<Vec<String>>();
        Some(v.to_owned())
    } else {
        None
    };

    let pw_dids = if !pw_dids.is_null() {
        check_useful_c_str!(pw_dids, VcxErrorKind::InvalidOption);
        let v: Vec<&str> = pw_dids.split(',').collect();
        let v = v.iter().map(|s| s.to_string()).collect::<Vec<String>>();
        Some(v.to_owned())
    } else {
        None
    };

    trace!("vcx_messages_download(command_handle: {}, message_status: {:?}, uids: {:?})",
           command_handle, message_status, uids);

    execute_async::<BoxFuture<'static, Result<(), ()>>>(async move {
        match aries_vcx::agency_client::get_message::download_messages_noauth(pw_dids, message_status, uids).await {
            Ok(x) => {
                match serde_json::to_string(&x) {
                    Ok(x) => {
                        trace!("vcx_messages_download_cb(command_handle: {}, rc: {}, messages: {})",
                               command_handle, error::SUCCESS.message, x);

                        let msg = CStringUtils::string_to_cstring(x);
                        cb(command_handle, error::SUCCESS.code_num, msg.as_ptr());
                    }
                    Err(e) => {
                        let err = VcxError::from_msg(VcxErrorKind::InvalidJson, format!("Cannot serialize messages: {}", e));
                        warn!("vcx_messages_download_cb(command_handle: {}, rc: {}, messages: {})",
                              command_handle, err, "null");

                        cb(command_handle, err.into(), ptr::null_mut());
                    }
                };
            }
            Err(e) => {
                warn!("vcx_messages_download_cb(command_handle: {}, rc: {}, messages: {})",
                      command_handle, e, "null");

                cb(command_handle, e.into(), ptr::null_mut());
            }
        };

        Ok(())
    }.boxed());

    error::SUCCESS.code_num
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
pub extern fn vcx_v2_messages_download(command_handle: CommandHandle,
                                       conn_handles: *const c_char,
                                       message_statuses: *const c_char,
                                       uids: *const c_char,
                                       cb: Option<extern fn(xcommand_handle: CommandHandle, err: u32, messages: *const c_char)>) -> u32 {
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
        Err(err) => return err.into()
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
        Err(_err) => return VcxError::from(VcxErrorKind::InvalidConnectionHandle).into()
    };

    let uids = if !uids.is_null() {
        check_useful_c_str!(uids, VcxErrorKind::InvalidOption);
        let v: Vec<&str> = uids.split(',').collect();
        let v = v.iter().map(|s| s.to_string()).collect::<Vec<String>>();
        Some(v.to_owned())
    } else {
        None
    };

    trace!("vcx_v2_messages_download(command_handle: {}, message_statuses: {:?}, uids: {:?})",
           command_handle, message_statuses, uids);

    execute_async::<BoxFuture<'static, Result<(), ()>>>(async move {
        match connection::download_messages(conn_handles, message_statuses, uids).await {
            Ok(x) => {
                match serde_json::to_string(&x) {
                    Ok(x) => {
                        trace!("vcx_v2_messages_download_cb(command_handle: {}, rc: {}, messages: {})",
                               command_handle, error::SUCCESS.message, x);

                        let msg = CStringUtils::string_to_cstring(x);
                        cb(command_handle, error::SUCCESS.code_num, msg.as_ptr());
                    }
                    Err(e) => {
                        let err = VcxError::from_msg(VcxErrorKind::InvalidJson, format!("Cannot serialize messages: {}", e));
                        warn!("vcx_v2_messages_download_cb(command_handle: {}, rc: {}, messages: {})",
                              command_handle, err, "null");

                        cb(command_handle, err.into(), ptr::null_mut());
                    }
                };
            }
            Err(e) => {
                warn!("vcx_messages_download_cb(command_handle: {}, rc: {}, messages: {})",
                      command_handle, e, "null");

                cb(command_handle, e.into(), ptr::null_mut());
            }
        };

        Ok(())
    }.boxed());

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
pub extern fn vcx_messages_update_status(command_handle: CommandHandle,
                                         message_status: *const c_char,
                                         msg_json: *const c_char,
                                         cb: Option<extern fn(xcommand_handle: CommandHandle, err: u32)>) -> u32 {
    info!("vcx_messages_update_status >>>");

    check_useful_c_callback!(cb, VcxErrorKind::InvalidOption);
    check_useful_c_str!(message_status, VcxErrorKind::InvalidOption);
    check_useful_c_str!(msg_json, VcxErrorKind::InvalidOption);

    trace!("vcx_messages_set_status(command_handle: {}, message_status: {:?}, uids: {:?})",
           command_handle, message_status, msg_json);

    execute_async::<BoxFuture<'static, Result<(), ()>>>(async move {
        match aries_vcx::agency_client::update_message::update_agency_messages(&message_status, &msg_json).await {
            Ok(()) => {
                trace!("vcx_messages_set_status_cb(command_handle: {}, rc: {})",
                       command_handle, error::SUCCESS.message);

                cb(command_handle, error::SUCCESS.code_num);
            }
            Err(e) => {
                warn!("vcx_messages_set_status_cb(command_handle: {}, rc: {})",
                      command_handle, e);

                cb(command_handle, e.into());
            }
        };

        Ok(())
    }.boxed());

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
pub extern fn vcx_pool_set_handle(handle: i32) -> i32 {
    if handle <= 0 { aries_vcx::libindy::utils::pool::set_pool_handle(None); } else { aries_vcx::libindy::utils::pool::set_pool_handle(Some(handle)); }

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
pub extern fn vcx_endorse_transaction(command_handle: CommandHandle,
                                      transaction: *const c_char,
                                      cb: Option<extern fn(xcommand_handle: CommandHandle, err: u32)>) -> u32 {
    info!("vcx_endorse_transaction >>>");

    check_useful_c_str!(transaction, VcxErrorKind::InvalidOption);
    check_useful_c_callback!(cb, VcxErrorKind::InvalidOption);
    trace!("vcx_endorse_transaction(command_handle: {}, transaction: {})",
           command_handle, transaction);

    execute(move || {
        match aries_vcx::libindy::utils::ledger::endorse_transaction(&transaction) {
            Ok(()) => {
                trace!("vcx_endorse_transaction(command_handle: {}, rc: {})",
                       command_handle, error::SUCCESS.message);

                cb(command_handle, error::SUCCESS.code_num);
            }
            Err(e) => {
                warn!("vcx_endorse_transaction(command_handle: {}, rc: {})",
                      command_handle, e);

                cb(command_handle, e.into());
            }
        };

        Ok(())
    });

    error::SUCCESS.code_num
}

#[cfg(test)]
mod tests {
    use std::ffi::CString;

    use aries_vcx::agency_client::mocking::AgencyMockDecrypted;
    use aries_vcx::utils::constants;
    use aries_vcx::utils::devsetup::SetupMocks;
    use aries_vcx::utils::provision::AgentProvisionConfig;

    use crate::api_lib::utils::return_types_u32;
    use crate::api_lib::utils::timeout::TimeoutUtils;

    use super::*;

    fn _vcx_agent_provision_async_c_closure(config: &str) -> Result<Option<String>, u32> {
        let cb = return_types_u32::Return_U32_STR::new().unwrap();
        let rc = vcx_provision_cloud_agent(cb.command_handle,
                                           CString::new(config).unwrap().into_raw(),
                                           Some(cb.get_callback()));
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
        }).to_string();

        let err = _vcx_agent_provision_async_c_closure(&config).unwrap_err();
        assert_eq!(err, error::INVALID_CONFIGURATION.code_num);
    }

    #[test]
    #[cfg(feature = "general_test")]
    #[allow(deprecated)]
    fn test_messages_download() {
        let _setup = SetupMocks::init();

        AgencyMockDecrypted::set_next_decrypted_response(constants::GET_MESSAGES_DECRYPTED_RESPONSE);

        let cb = return_types_u32::Return_U32_STR::new().unwrap();
        assert_eq!(vcx_messages_download(cb.command_handle, ptr::null_mut(), ptr::null_mut(), ptr::null_mut(), Some(cb.get_callback())), error::SUCCESS.code_num);
        cb.receive(TimeoutUtils::some_medium()).unwrap();
    }

    #[test]
    #[cfg(feature = "general_test")]
    fn test_messages_update_status() {
        let _setup = SetupMocks::init();

        AgencyMockDecrypted::set_next_decrypted_response(constants::GET_MESSAGES_DECRYPTED_RESPONSE);

        let status = CString::new("MS-103").unwrap().into_raw();
        let json = CString::new(r#"[{"pairwiseDID":"QSrw8hebcvQxiwBETmAaRs","uids":["mgrmngq"]}]"#).unwrap().into_raw();

        let cb = return_types_u32::Return_U32::new().unwrap();
        assert_eq!(vcx_messages_update_status(cb.command_handle,
                                              status,
                                              json,
                                              Some(cb.get_callback())),
                   error::SUCCESS.code_num);
        cb.receive(TimeoutUtils::some_medium()).unwrap();
    }
}
