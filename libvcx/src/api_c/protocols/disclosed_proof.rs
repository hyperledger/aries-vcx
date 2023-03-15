use std::ptr;

use futures::future::BoxFuture;
use libc::c_char;

use libvcx_core::api_vcx::api_handle::disclosed_proof;
use libvcx_core::errors;
use libvcx_core::errors::error::{LibvcxError, LibvcxErrorKind};

use crate::api_c::cutils::cstring::CStringUtils;
use crate::api_c::cutils::current_error::set_current_error_vcx;
use crate::api_c::cutils::runtime::{execute, execute_async};
use crate::api_c::types::CommandHandle;
use crate::error::SUCCESS_ERR_CODE;

/*
    APIs in this module are called by a prover throughout the request-proof-and-verify process.
    Assumes that pairwise connection between Verifier and Prover is already established.

    # State

    The set of object states, messages and transitions depends on the communication method is used.
    The communication method can be specified as a config option on one of *_init functions.

    aries:
        VcxStateType::VcxStateRequestReceived - once `vcx_disclosed_proof_create_with_request` (create DisclosedProof object) is called.

        VcxStateType::VcxStateRequestReceived - once `vcx_disclosed_proof_generate_proof` is called.

        VcxStateType::VcxStateOfferSent - once `vcx_disclosed_proof_send_proof` (send `Presentation` message) is called.
        VcxStateType::None - once `vcx_disclosed_proof_decline_presentation_request` (send `PresentationReject` or `PresentationProposal` message) is called.

        VcxStateType::VcxStateAccepted - once `Ack` messages is received.
        VcxStateType::None - once `ProblemReport` messages is received.

    # Transitions

    aries: RFC - https://github.com/hyperledger/aries-rfcs/tree/7b6b93acbaf9611d3c892c4bada142fe2613de6e/features/0037-present-proof#propose-presentation
        VcxStateType::None - `vcx_disclosed_proof_create_with_request` - VcxStateType::VcxStateRequestReceived

        VcxStateType::VcxStateRequestReceived - `vcx_disclosed_proof_generate_proof` - VcxStateType::VcxStateRequestReceived

        VcxStateType::VcxStateRequestReceived - `vcx_disclosed_proof_send_proof` - VcxStateType::VcxStateAccepted
        VcxStateType::VcxStateRequestReceived - `vcx_disclosed_proof_decline_presentation_request` - VcxStateType::None

        VcxStateType::VcxStateOfferSent - received `Ack` - VcxStateType::VcxStateAccepted
        VcxStateType::VcxStateOfferSent - received `ProblemReport` - VcxStateType::None

    # Messages

    aries:
        PresentationRequest - https://github.com/hyperledger/aries-rfcs/tree/7b6b93acbaf9611d3c892c4bada142fe2613de6e/features/0037-present-proof#request-presentation
        Presentation - https://github.com/hyperledger/aries-rfcs/tree/7b6b93acbaf9611d3c892c4bada142fe2613de6e/features/0037-present-proof#presentation
        PresentationProposal - https://github.com/hyperledger/aries-rfcs/tree/7b6b93acbaf9611d3c892c4bada142fe2613de6e/features/0037-present-proof#propose-presentation
        Ack - https://github.com/hyperledger/aries-rfcs/tree/master/features/0015-acks#explicit-acks
*/

/// Create a Proof object for fulfilling a corresponding proof request
///
/// #Params
/// command_handle: command handle to map callback to user context.
///
/// source_id: Institution's identification for the proof, should be unique.
///
/// req: proof request received via "vcx_get_proof_requests"
///
/// cb: Callback that provides proof handle or error status
///
/// # Example proof_req -> "{"@topic":{"mid":9,"tid":1},"@type":{"name":"PROOF_REQUEST","version":"1.0"},"msg_ref_id":"ymy5nth","proof_request_data":{"name":"AccountCertificate","nonce":"838186471541979035208225","requested_attributes":{"business_2":{"name":"business"},"email_1":{"name":"email"},"name_0":{"name":"name"}},"requested_predicates":{},"version":"0.1"}}"
///
/// #Returns
/// Error code as u32
#[no_mangle]
#[allow(unused_variables, unused_mut)]
pub extern "C" fn vcx_disclosed_proof_create_with_request(
    command_handle: CommandHandle,
    source_id: *const c_char,
    proof_req: *const c_char,
    cb: Option<extern "C" fn(xcommand_handle: CommandHandle, err: u32, handle: u32)>,
) -> u32 {
    info!("vcx_disclosed_proof_create_with_request >>>");

    check_useful_c_callback!(cb, LibvcxErrorKind::InvalidOption);
    check_useful_c_str!(source_id, LibvcxErrorKind::InvalidOption);
    check_useful_c_str!(proof_req, LibvcxErrorKind::InvalidOption);

    trace!(
        "vcx_disclosed_proof_create_with_request(command_handle: {}, source_id: {}, proof_req: {})",
        command_handle,
        source_id,
        proof_req
    );

    execute(move || {
        match disclosed_proof::create_with_proof_request(&source_id, &proof_req) {
            Ok(err) => {
                trace!(
                    "vcx_disclosed_proof_create_with_request_cb(command_handle: {}, rc: {}, handle: {}) source_id: {}",
                    command_handle,
                    SUCCESS_ERR_CODE,
                    err,
                    source_id
                );
                cb(command_handle, 0, err);
            }
            Err(err) => {
                set_current_error_vcx(&err);
                error!(
                    "vcx_disclosed_proof_create_with_request_cb(command_handle: {}, rc: {}, handle: {}) source_id: {}",
                    command_handle, err, 0, source_id
                );
                cb(command_handle, err.into(), 0);
            }
        };

        Ok(())
    });

    SUCCESS_ERR_CODE
}

/// Create a proof based off of a known message id for a given connection.
///
/// #Params
/// command_handle: command handle to map callback to user context.
///
/// source_id: Institution's personal identification for the proof, should be unique.
///
/// connection: connection to query for proof request
///
/// msg_id:  id of the message that contains the proof request
///
/// cb: Callback that provides proof handle and proof request or error status
///
/// #Returns
/// Error code as a u32
#[no_mangle]
#[allow(unused_variables, unused_mut)]
pub extern "C" fn vcx_disclosed_proof_create_with_msgid(
    command_handle: CommandHandle,
    source_id: *const c_char,
    connection_handle: u32,
    msg_id: *const c_char,
    cb: Option<extern "C" fn(xcommand_handle: CommandHandle, err: u32, proof_handle: u32, proof_req: *const c_char)>,
) -> u32 {
    info!("vcx_disclosed_proof_create_with_msgid >>>");

    check_useful_c_callback!(cb, LibvcxErrorKind::InvalidOption);
    check_useful_c_str!(source_id, LibvcxErrorKind::InvalidOption);
    check_useful_c_str!(msg_id, LibvcxErrorKind::InvalidOption);

    trace!(
        "vcx_disclosed_proof_create_with_msgid(command_handle: {}, source_id: {}, connection_handle: {}, msg_id: {})",
        command_handle,
        source_id,
        connection_handle,
        msg_id
    );

    execute_async::<BoxFuture<'static, Result<(), ()>>>(Box::pin(async move {
        match disclosed_proof::create_with_msgid(&source_id, connection_handle, &msg_id).await {
            Ok((handle, request)) => {
                trace!("vcx_disclosed_proof_create_with_msgid_cb(command_handle: {}, rc: {}, handle: {}, proof_req: {}) source_id: {}", command_handle, SUCCESS_ERR_CODE, handle, request, source_id);
                let msg = CStringUtils::string_to_cstring(request);
                cb(command_handle, SUCCESS_ERR_CODE, handle, msg.as_ptr())
            }
            Err(err) => {
                set_current_error_vcx(&err);
                cb(command_handle, err.into(), 0, ptr::null());
            }
        };

        Ok(())
    }));

    SUCCESS_ERR_CODE
}

/// Send a proof to the connection, called after having received a proof request
///
/// #params
/// command_handle: command handle to map callback to API user context.
///
/// proof_handle: proof handle that was provided duration creation.  Used to identify proof object.
///
/// connection_handle: Connection handle that identifies pairwise connection
///
/// cb: Callback that provides error status of proof send request
///
/// #Returns
/// Error code as u32
#[no_mangle]
pub extern "C" fn vcx_disclosed_proof_send_proof(
    command_handle: CommandHandle,
    proof_handle: u32,
    connection_handle: u32,
    cb: Option<extern "C" fn(xcommand_handle: CommandHandle, err: u32)>,
) -> u32 {
    info!("vcx_disclosed_proof_send_proof >>>");

    check_useful_c_callback!(cb, LibvcxErrorKind::InvalidOption);

    let source_id = disclosed_proof::get_source_id(proof_handle).unwrap_or_default();
    trace!(
        "vcx_disclosed_proof_send_proof(command_handle: {}, proof_handle: {}, connection_handle: {}) source_id: {}",
        command_handle,
        proof_handle,
        connection_handle,
        source_id
    );

    execute_async::<BoxFuture<'static, Result<(), ()>>>(Box::pin(async move {
        match disclosed_proof::send_proof(proof_handle, connection_handle).await {
            Ok(_) => {
                trace!(
                    "vcx_disclosed_proof_send_proof_cb(command_handle: {}, rc: {}) source_id: {}",
                    command_handle,
                    SUCCESS_ERR_CODE,
                    source_id
                );
                cb(command_handle, SUCCESS_ERR_CODE);
            }
            Err(err) => {
                set_current_error_vcx(&err);
                error!(
                    "vcx_disclosed_proof_send_proof_cb(command_handle: {}, rc: {}) source_id: {}",
                    command_handle, err, source_id
                );
                cb(command_handle, err.into());
            }
        };

        Ok(())
    }));

    SUCCESS_ERR_CODE
}

/// Send a proof rejection to the connection, called after having received a proof request
///
/// #params
/// command_handle: command handle to map callback to API user context.
///
/// proof_handle: proof handle that was provided duration creation.  Used to identify proof object.
///
/// connection_handle: Connection handle that identifies pairwise connection
///
/// cb: Callback that provides error status of proof send request
///
/// #Returns
/// Error code as u32
#[no_mangle]
pub extern "C" fn vcx_disclosed_proof_reject_proof(
    command_handle: CommandHandle,
    proof_handle: u32,
    connection_handle: u32,
    cb: Option<extern "C" fn(xcommand_handle: CommandHandle, err: u32)>,
) -> u32 {
    info!("vcx_disclosed_proof_reject_proof >>>");

    check_useful_c_callback!(cb, LibvcxErrorKind::InvalidOption);

    let source_id = disclosed_proof::get_source_id(proof_handle).unwrap_or_default();
    trace!(
        "vcx_disclosed_proof_reject_proof(command_handle: {}, proof_handle: {}, connection_handle: {}) source_id: {}",
        command_handle,
        proof_handle,
        connection_handle,
        source_id
    );

    execute_async::<BoxFuture<'static, Result<(), ()>>>(Box::pin(async move {
        match disclosed_proof::reject_proof(proof_handle, connection_handle).await {
            Ok(_) => {
                trace!(
                    "vcx_disclosed_proof_reject_proof_cb(command_handle: {}, rc: {}) source_id: {}",
                    command_handle,
                    SUCCESS_ERR_CODE,
                    source_id
                );
                cb(command_handle, SUCCESS_ERR_CODE);
            }
            Err(err) => {
                set_current_error_vcx(&err);
                error!(
                    "vcx_disclosed_proof_reject_proof_cb(command_handle: {}, rc: {}) source_id: {}",
                    command_handle, err, source_id
                );
                cb(command_handle, err.into());
            }
        };

        Ok(())
    }));

    SUCCESS_ERR_CODE
}

/// Get the proof message for sending.
///
/// #params
/// command_handle: command handle to map callback to API user context.
///
/// proof_handle: proof handle that was provided duration creation.  Used to identify proof object.
///
/// cb: Callback that provides error status of proof send request
///
/// #Returns
/// Error code as u32
#[no_mangle]
pub extern "C" fn vcx_disclosed_proof_get_proof_msg(
    command_handle: CommandHandle,
    proof_handle: u32,
    cb: Option<extern "C" fn(xcommand_handle: CommandHandle, err: u32, msg: *const c_char)>,
) -> u32 {
    info!("vcx_disclosed_proof_get_proof_msg >>>");

    check_useful_c_callback!(cb, LibvcxErrorKind::InvalidOption);

    let source_id = disclosed_proof::get_source_id(proof_handle).unwrap_or_default();
    trace!(
        "vcx_disclosed_proof_get_proof_msg(command_handle: {}, proof_handle: {}) source_id: {}",
        command_handle,
        proof_handle,
        source_id
    );

    execute(move || {
        match disclosed_proof::get_presentation_msg(proof_handle) {
            Ok(msg) => {
                let msg = CStringUtils::string_to_cstring(msg);
                trace!(
                    "vcx_disclosed_proof_get_proof_msg_cb(command_handle: {}, rc: {}) source_id: {}",
                    command_handle,
                    SUCCESS_ERR_CODE,
                    source_id
                );
                cb(command_handle, SUCCESS_ERR_CODE, msg.as_ptr());
            }
            Err(err) => {
                set_current_error_vcx(&err);
                error!(
                    "vcx_disclosed_proof_get_proof_msg_cb(command_handle: {}, rc: {}) source_id: {}",
                    command_handle, err, source_id
                );
                cb(command_handle, err.into(), ptr::null_mut());
            }
        };

        Ok(())
    });

    SUCCESS_ERR_CODE
}

/// Get the reject proof message for sending.
///
/// #params
/// command_handle: command handle to map callback to API user context.
///
/// proof_handle: proof handle that was provided duration creation.  Used to identify proof object.
///
/// cb: Callback that provides error status of proof send request
///
/// #Returns
/// Error code as u32
#[no_mangle]
pub extern "C" fn vcx_disclosed_proof_get_reject_msg(
    command_handle: CommandHandle,
    proof_handle: u32,
    cb: Option<extern "C" fn(xcommand_handle: CommandHandle, err: u32, msg: *const c_char)>,
) -> u32 {
    info!("vcx_disclosed_proof_get_reject_msg >>>");

    check_useful_c_callback!(cb, LibvcxErrorKind::InvalidOption);

    let source_id = disclosed_proof::get_source_id(proof_handle).unwrap_or_default();
    trace!(
        "vcx_disclosed_proof_get_reject_msg(command_handle: {}, proof_handle: {}) source_id: {}",
        command_handle,
        proof_handle,
        source_id
    );

    execute(move || {
        match disclosed_proof::generate_reject_proof_msg(proof_handle) {
            Ok(msg) => {
                let msg = CStringUtils::string_to_cstring(msg);
                trace!(
                    "vcx_disclosed_proof_get_reject_msg_cb(command_handle: {}, rc: {}) source_id: {}",
                    command_handle,
                    SUCCESS_ERR_CODE,
                    source_id
                );
                cb(command_handle, SUCCESS_ERR_CODE, msg.as_ptr());
            }
            Err(err) => {
                set_current_error_vcx(&err);
                error!(
                    "vcx_disclosed_proof_get_reject_msg_cb(command_handle: {}, rc: {}) source_id: {}",
                    command_handle, err, source_id
                );
                cb(command_handle, err.into(), ptr::null_mut());
            }
        };

        Ok(())
    });

    SUCCESS_ERR_CODE
}

/// Queries agency for all pending proof requests from the given connection.
///
/// #Params
/// command_handle: command handle to map callback to user context.
///
/// connection_handle: Connection to query for proof requests.
///
/// cb: Callback that provides any proof requests and error status of query
/// # Example requests -> "[{'@topic': {'tid': 0, 'mid': 0}, '@type': {'version': '1.0', 'name': 'PROOF_REQUEST'}, 'proof_request_data': {'name': 'proof_req', 'nonce': '118065925949165739229152', 'version': '0.1', 'requested_predicates': {}, 'non_revoked': None, 'requested_attributes': {'attribute_0': {'name': 'name', 'restrictions': {'$or': [{'issuer_did': 'did'}]}}}, 'ver': '1.0'}, 'thread_id': '40bdb5b2'}]"
///
/// #Returns
/// Error code as a u32
#[no_mangle]
pub extern "C" fn vcx_disclosed_proof_get_requests(
    command_handle: CommandHandle,
    connection_handle: u32,
    cb: Option<extern "C" fn(xcommand_handle: CommandHandle, err: u32, requests: *const c_char)>,
) -> u32 {
    info!("vcx_disclosed_proof_get_requests >>>");

    check_useful_c_callback!(cb, LibvcxErrorKind::InvalidOption);

    trace!(
        "vcx_disclosed_proof_get_requests(command_handle: {}, connection_handle: {})",
        command_handle,
        connection_handle
    );

    execute_async::<BoxFuture<'static, Result<(), ()>>>(Box::pin(async move {
        match disclosed_proof::get_proof_request_messages(connection_handle).await {
            Ok(err) => {
                trace!(
                    "vcx_disclosed_proof_get_requests_cb(command_handle: {}, rc: {}, msg: {})",
                    command_handle,
                    SUCCESS_ERR_CODE,
                    err
                );
                let msg = CStringUtils::string_to_cstring(err);
                cb(command_handle, SUCCESS_ERR_CODE, msg.as_ptr());
            }
            Err(err) => {
                set_current_error_vcx(&err);
                error!(
                    "vcx_disclosed_proof_get_requests_cb(command_handle: {}, rc: {}, msg: {})",
                    command_handle, SUCCESS_ERR_CODE, err
                );
                cb(command_handle, err.into(), ptr::null_mut());
            }
        };

        Ok(())
    }));

    SUCCESS_ERR_CODE
}

/// Get the current state of the disclosed proof object
///
/// #Params
/// command_handle: command handle to map callback to user context.
///
/// proof_handle: Proof handle that was provided during creation. Used to access disclosed proof object
///
/// cb: Callback that provides most current state of the disclosed proof and error status of request
///     States:
///         3 - Request Received
///         4 - Accepted
///
/// #Returns
/// Error code as a u32
#[no_mangle]
pub extern "C" fn vcx_disclosed_proof_get_state(
    command_handle: CommandHandle,
    proof_handle: u32,
    cb: Option<extern "C" fn(xcommand_handle: CommandHandle, err: u32, state: u32)>,
) -> u32 {
    info!("vcx_disclosed_proof_get_state >>>");

    check_useful_c_callback!(cb, LibvcxErrorKind::InvalidOption);

    let source_id = disclosed_proof::get_source_id(proof_handle).unwrap_or_default();
    trace!(
        "vcx_disclosed_proof_get_state(command_handle: {}, proof_handle: {}), source_id: {:?}",
        command_handle,
        proof_handle,
        source_id
    );

    execute(move || {
        match disclosed_proof::get_state(proof_handle) {
            Ok(s) => {
                trace!(
                    "vcx_disclosed_proof_get_state_cb(command_handle: {}, rc: {}, state: {}) source_id: {}",
                    command_handle,
                    SUCCESS_ERR_CODE,
                    s,
                    source_id
                );
                cb(command_handle, SUCCESS_ERR_CODE, s)
            }
            Err(err) => {
                set_current_error_vcx(&err);
                error!(
                    "vcx_disclosed_proof_get_state_cb(command_handle: {}, rc: {}, state: {}) source_id: {}",
                    command_handle, err, 0, source_id
                );
                cb(command_handle, err.into(), 0)
            }
        };

        Ok(())
    });

    SUCCESS_ERR_CODE
}

#[no_mangle]
pub extern "C" fn vcx_disclosed_proof_get_proof_request_attachment(
    command_handle: CommandHandle,
    proof_handle: u32,
    cb: Option<extern "C" fn(xcommand_handle: CommandHandle, err: u32, attributes: *const c_char)>,
) -> u32 {
    info!("vcx_disclosed_proof_get_attachment >>>");

    check_useful_c_callback!(cb, LibvcxErrorKind::InvalidOption);

    let source_id = disclosed_proof::get_source_id(proof_handle).unwrap_or_default();
    trace!(
        "vcx_disclosed_proof_get_attachment(command_handle: {}, proof_handle: {}), source_id: {:?}",
        command_handle,
        proof_handle,
        source_id
    );

    execute(move || {
        match disclosed_proof::get_proof_request_attachment(proof_handle) {
            Ok(err) => {
                trace!(
                    "vcx_disclosed_proof_get_attachment_cb(command_handle: {}, rc: {}, attachment: {}) source_id: {}",
                    command_handle,
                    SUCCESS_ERR_CODE,
                    err,
                    source_id
                );
                let attrs = CStringUtils::string_to_cstring(err);
                cb(command_handle, SUCCESS_ERR_CODE, attrs.as_ptr());
            }
            Err(err) => {
                set_current_error_vcx(&err);
                error!(
                    "vcx_disclosed_proof_get_attachment_cb(command_handle: {}, rc: {}, attachment: {}) source_id: {}",
                    command_handle, SUCCESS_ERR_CODE, err, source_id
                );
                cb(command_handle, err.into(), ptr::null_mut());
            }
        };

        Ok(())
    });

    SUCCESS_ERR_CODE
}

#[no_mangle]
pub extern "C" fn vcx_v2_disclosed_proof_update_state(
    command_handle: CommandHandle,
    proof_handle: u32,
    connection_handle: u32,
    cb: Option<extern "C" fn(xcommand_handle: CommandHandle, err: u32, state: u32)>,
) -> u32 {
    info!("vcx_v2_disclosed_proof_update_state >>>");

    check_useful_c_callback!(cb, LibvcxErrorKind::InvalidOption);

    let source_id = disclosed_proof::get_source_id(proof_handle).unwrap_or_default();
    trace!(
        "vcx_v2_disclosed_proof_update_state(command_handle: {} proof_handle: {}, connection_handle: {}) source_id: {}",
        command_handle,
        proof_handle,
        connection_handle,
        source_id
    );

    execute_async::<BoxFuture<'static, Result<(), ()>>>(Box::pin(async move {
        match disclosed_proof::update_state(proof_handle, None, connection_handle).await {
            Ok(state) => {
                trace!(
                    "vcx_v2_disclosed_proof_update_state_cb(command_handle: {}, rc: {}, state: {}) source_id: {}",
                    command_handle,
                    SUCCESS_ERR_CODE,
                    state,
                    source_id
                );
                cb(command_handle, SUCCESS_ERR_CODE, state)
            }
            Err(err) => {
                error!(
                    "vcx_v2_disclosed_proof_update_state_cb(command_handle: {}, rc: {}, state: {}) source_id: {}",
                    command_handle, err, 0, source_id
                );
                cb(command_handle, err.into(), 0)
            }
        };

        Ok(())
    }));

    SUCCESS_ERR_CODE
}

/// Checks for any state change from the given message and updates the state attribute
///
/// #Params
/// command_handle: command handle to map callback to user context.
///
/// proof_handle: Credential handle that was provided during creation. Used to identify disclosed proof object
///
/// connection_handle: Connection handle of connection associated with this proof exchange interaction.
///
/// message: message to process for state changes
///
/// cb: Callback that provides most current state of the disclosed proof and error status of request
///
/// #Returns
/// Error code as a u32
#[no_mangle]
pub extern "C" fn vcx_v2_disclosed_proof_update_state_with_message(
    command_handle: CommandHandle,
    proof_handle: u32,
    connection_handle: u32,
    message: *const c_char,
    cb: Option<extern "C" fn(xcommand_handle: CommandHandle, err: u32, state: u32)>,
) -> u32 {
    info!("vcx_v2_disclosed_proof_update_state_with_message >>>");

    check_useful_c_callback!(cb, LibvcxErrorKind::InvalidOption);
    check_useful_c_str!(message, LibvcxErrorKind::InvalidOption);

    let source_id = disclosed_proof::get_source_id(proof_handle).unwrap_or_default();
    trace!(
        "vcx_v2_disclosed_proof_update_state_with_message(command_handle: {}, proof_handle: {}) source_id: {}",
        command_handle,
        proof_handle,
        source_id
    );

    execute_async::<BoxFuture<'static, Result<(), ()>>>(Box::pin(async move {
        match disclosed_proof::update_state(proof_handle, Some(&message), connection_handle).await {
            Ok(state) => {
                trace!("vcx_v2_disclosed_proof_update_state_with_message_cb(command_handle: {}, rc: {}, state: {}) source_id: {}", command_handle, SUCCESS_ERR_CODE, state, source_id);
                cb(command_handle, SUCCESS_ERR_CODE, state)
            }
            Err(err) => {
                error!("vcx_v2_disclosed_proof_update_state_with_message_cb(command_handle: {}, rc: {}, state: {}) source_id: {}", command_handle, err, 0, source_id);
                cb(command_handle, err.into(), 0)
            }
        };

        Ok(())
    }));

    SUCCESS_ERR_CODE
}

/// Takes the disclosed proof object and returns a json string of all its attributes
///
/// #Params
/// command_handle: command handle to map callback to user context.
///
/// handle: Proof handle that was provided during creation. Used to identify the disclosed proof object
///
/// cb: Callback that provides json string of the disclosed proof's attributes and provides error status
///
/// #Returns
/// Error code as a u32
#[no_mangle]
pub extern "C" fn vcx_disclosed_proof_serialize(
    command_handle: CommandHandle,
    proof_handle: u32,
    cb: Option<extern "C" fn(xcommand_handle: CommandHandle, err: u32, data: *const c_char)>,
) -> u32 {
    info!("vcx_disclosed_proof_serialize >>>");

    check_useful_c_callback!(cb, LibvcxErrorKind::InvalidOption);

    let source_id = disclosed_proof::get_source_id(proof_handle).unwrap_or_default();
    trace!(
        "vcx_disclosed_proof_serialize(command_handle: {}, proof_handle: {}) source_id: {}",
        command_handle,
        proof_handle,
        source_id
    );

    execute(move || {
        match disclosed_proof::to_string(proof_handle) {
            Ok(serialized) => {
                trace!(
                    "vcx_disclosed_proof_serialize_cb(command_handle: {}, rc: {}, data: {}) source_id: {}",
                    command_handle,
                    SUCCESS_ERR_CODE,
                    serialized,
                    source_id
                );
                let c_serialized = CStringUtils::string_to_cstring(serialized);
                cb(command_handle, SUCCESS_ERR_CODE, c_serialized.as_ptr());
            }
            Err(err) => {
                set_current_error_vcx(&err);
                error!(
                    "vcx_disclosed_proof_serialize_cb(command_handle: {}, rc: {}, data: {}) source_id: {}",
                    command_handle, err, 0, source_id
                );
                cb(command_handle, err.into(), ptr::null_mut());
            }
        };

        Ok(())
    });

    SUCCESS_ERR_CODE
}

/// Takes a json string representing an disclosed proof object and recreates an object matching the json
///
/// #Params
/// command_handle: command handle to map callback to user context.
///
/// data: json string representing a disclosed proof object
///
///
/// cb: Callback that provides handle and provides error status
///
/// #Returns
/// Error code as a u32
#[no_mangle]
pub extern "C" fn vcx_disclosed_proof_deserialize(
    command_handle: CommandHandle,
    proof_data: *const c_char,
    cb: Option<extern "C" fn(xcommand_handle: CommandHandle, err: u32, handle: u32)>,
) -> u32 {
    info!("vcx_disclosed_proof_deserialize >>>");

    check_useful_c_callback!(cb, LibvcxErrorKind::InvalidOption);
    check_useful_c_str!(proof_data, LibvcxErrorKind::InvalidOption);

    trace!(
        "vcx_disclosed_proof_deserialize(command_handle: {}, proof_data: {})",
        command_handle,
        proof_data
    );

    execute(move || {
        match disclosed_proof::from_string(&proof_data) {
            Ok(handle) => {
                trace!(
                    "vcx_disclosed_proof_deserialize_cb(command_handle: {}, rc: {}, proof_handle: {}) source_id: {}",
                    command_handle,
                    SUCCESS_ERR_CODE,
                    handle,
                    disclosed_proof::get_source_id(handle).unwrap_or_default()
                );

                cb(command_handle, 0, handle);
            }
            Err(err) => {
                set_current_error_vcx(&err);
                error!(
                    "vcx_disclosed_proof_deserialize_cb(command_handle: {}, rc: {}, proof_handle: {}) source_id: {}",
                    command_handle, err, 0, ""
                );
                cb(command_handle, err.into(), 0);
            }
        };

        Ok(())
    });

    SUCCESS_ERR_CODE
}

/// Get credentials from wallet matching to the proof request associated with proof object
///
/// #Params
/// command_handle: command handle to map callback to user context.
///
/// handle: Proof handle that was provided during creation. Used to identify the disclosed proof object
///
/// cb: Callback that provides json string of the credentials in wallet associated with proof request
///
/// # Example
/// credentials -> "{'attrs': {'attribute_0': [{'cred_info': {'schema_id': 'id', 'cred_def_id': 'id', 'attrs': {'attr_name': 'attr_value', ...}, 'referent': '914c7e11'}}]}}"
///
/// #Returns
/// Error code as a u32
#[no_mangle]
pub extern "C" fn vcx_disclosed_proof_retrieve_credentials(
    command_handle: CommandHandle,
    proof_handle: u32,
    cb: Option<extern "C" fn(xcommand_handle: CommandHandle, err: u32, data: *const c_char)>,
) -> u32 {
    info!("vcx_disclosed_proof_retrieve_credentials >>>");

    check_useful_c_callback!(cb, LibvcxErrorKind::InvalidOption);

    let source_id = disclosed_proof::get_source_id(proof_handle).unwrap_or_default();
    trace!(
        "vcx_disclosed_proof_retrieve_credentials(command_handle: {}, proof_handle: {}) source_id: {}",
        command_handle,
        proof_handle,
        source_id
    );

    execute_async::<BoxFuture<'static, Result<(), ()>>>(Box::pin(async move {
        match disclosed_proof::retrieve_credentials(proof_handle).await {
            Ok(credentials) => {
                trace!(
                    "vcx_disclosed_proof_retrieve_credentials(command_handle: {}, rc: {}, data: {}) source_id: {}",
                    command_handle,
                    SUCCESS_ERR_CODE,
                    credentials,
                    source_id
                );
                let c_credentials = CStringUtils::string_to_cstring(credentials);
                cb(command_handle, SUCCESS_ERR_CODE, c_credentials.as_ptr());
            }
            Err(err) => {
                set_current_error_vcx(&err);
                error!(
                    "vcx_disclosed_proof_retrieve_credentials(command_handle: {}, rc: {}, data: {}) source_id: {}",
                    command_handle, err, 0, source_id
                );
                cb(command_handle, err.into(), ptr::null_mut());
            }
        };

        Ok(())
    }));

    SUCCESS_ERR_CODE
}

/// Accept proof request associated with proof object and generates a proof from the selected credentials and self attested attributes
///
/// #Params
/// command_handle: command handle to map callback to user context.
///
///
/// handle: Proof handle that was provided during creation. Used to identify the disclosed proof object
///
/// selected_credentials: a json string with a credential for each proof request attribute.
///     List of possible credentials for each attribute is returned from vcx_disclosed_proof_retrieve_credentials,
///         (user needs to select specific credential to use from list of credentials)
///         {
///             "attrs":{
///                 String:{// Attribute key: This may not be the same as the attr name ex. "age_1" where attribute name is "age"
///                     "credential": {
///                         "cred_info":{
///                             "referent":String,
///                             "attrs":{ String: String }, // ex. {"age": "111", "name": "Bob"}
///                             "schema_id": String,
///                             "cred_def_id": String,
///                             "rev_reg_id":Option<String>,
///                             "cred_rev_id":Option<String>,
///                             },
///                         "interval":Option<{to: Option<u64>, from:: Option<u64>}>
///                     }, // This is the exact credential information selected from list of
///                        // credentials returned from vcx_disclosed_proof_retrieve_credentials
///                     "tails_file": Option<"String">, // Path to tails file for this credential
///                 },
///            },
///           "predicates":{ TODO: will be implemented as part of IS-1095 ticket. }
///        }
///     // selected_credentials can be empty "{}" if the proof only contains self_attested_attrs
///
/// self_attested_attrs: a json string with attributes self attested by user
/// # Examples
/// self_attested_attrs -> "{"self_attested_attr_0":"attested_val"}" | "{}"
/// selected_credentials -> "{'attrs': {'attribute_0': {'credential': {'cred_info': {'cred_def_id': 'od', 'schema_id': 'id', 'referent': '0c212108-9433-4199-a21f-336a44164f38', 'attrs': {'attr_name': 'attr_value', ...}}}}}}"
/// cb: Callback that returns error status
///
/// #Returns
/// Error code as a u32
#[no_mangle]
pub extern "C" fn vcx_disclosed_proof_generate_proof(
    command_handle: CommandHandle,
    proof_handle: u32,
    selected_credentials: *const c_char,
    self_attested_attrs: *const c_char,
    cb: Option<extern "C" fn(xcommand_handle: CommandHandle, err: u32)>,
) -> u32 {
    info!("vcx_disclosed_proof_generate_proof >>>");

    check_useful_c_str!(selected_credentials, LibvcxErrorKind::InvalidOption);
    check_useful_c_str!(self_attested_attrs, LibvcxErrorKind::InvalidOption);
    check_useful_c_callback!(cb, LibvcxErrorKind::InvalidOption);

    let source_id = disclosed_proof::get_source_id(proof_handle).unwrap_or_default();
    trace!("vcx_disclosed_proof_generate_proof(command_handle: {}, proof_handle: {}, selected_credentials: {}, self_attested_attrs: {}) source_id: {}", command_handle, proof_handle, selected_credentials, self_attested_attrs, source_id);

    execute_async::<BoxFuture<'static, Result<(), ()>>>(Box::pin(async move {
        match disclosed_proof::generate_proof(proof_handle, &selected_credentials, &self_attested_attrs).await {
            Ok(()) => {
                trace!(
                    "vcx_disclosed_proof_generate_proof(command_handle: {}, rc: {}) source_id: {}",
                    command_handle,
                    SUCCESS_ERR_CODE,
                    source_id
                );
                cb(command_handle, SUCCESS_ERR_CODE);
            }
            Err(err) => {
                set_current_error_vcx(&err);
                error!(
                    "vcx_disclosed_proof_generate_proof(command_handle: {}, rc: {}) source_id: {}",
                    command_handle, err, source_id
                );
                cb(command_handle, err.into());
            }
        };

        Ok(())
    }));

    SUCCESS_ERR_CODE
}

/// Declines presentation request.
/// There are two ways of following interaction:
///     - Prover wants to propose using a different presentation - pass `proposal` parameter.
///     - Prover doesn't want to continue interaction - pass `reason` parameter.
/// Note that only one of these parameters can be passed.
///
/// Note that proposing of different presentation is supported for `aries` protocol only.
///
/// #Params
/// command_handle: command handle to map callback to user context.
///
/// proof_handle: Proof handle that was provided during creation. Used to identify the disclosed proof object
///
/// connection_handle: Connection handle that identifies pairwise connection
///
/// reason: (Optional) human-readable string that explain the reason of decline
///
/// proposal: (Optional) the proposed format of presentation request
/// (see https://github.com/hyperledger/aries-rfcs/tree/master/features/0037-present-proof#presentation-preview for details)
/// {
///    "attributes": [
///        {
///            "name": "<attribute_name>",
///            "cred_def_id": Optional("<cred_def_id>"),
///            "mime-type": Optional("<type>"),
///            "value": Optional("<value>")
///        },
///        // more attributes
///    ],
///    "predicates": [
///        {
///            "name": "<attribute_name>",
///            "cred_def_id": Optional("<cred_def_id>"),
///            "predicate": "<predicate>", - one of "<", "<=", ">=", ">"
///            "threshold": <threshold>
///        },
///        // more predicates
///    ]
/// }
///
/// # Example
///  proposal ->
///     {
///          "attributes": [
///              {
///                  "name": "first name"
///              }
///          ],
///          "predicates": [
///              {
///                  "name": "age",
///                  "predicate": ">",
///                  "threshold": 18
///              }
///          ]
///      }
///
/// cb: Callback that returns error status
///
/// #Returns
/// Error code as a u32
#[no_mangle]
pub extern "C" fn vcx_disclosed_proof_decline_presentation_request(
    command_handle: u32,
    proof_handle: u32,
    connection_handle: u32,
    reason: *const c_char,
    proposal: *const c_char,
    cb: Option<extern "C" fn(xcommand_handle: u32, err: u32)>,
) -> u32 {
    info!("vcx_disclosed_proof_decline_presentation_request >>>");

    check_useful_opt_c_str!(reason, LibvcxErrorKind::InvalidOption);
    check_useful_opt_c_str!(proposal, LibvcxErrorKind::InvalidOption);
    check_useful_c_callback!(cb, LibvcxErrorKind::InvalidOption);

    let source_id = disclosed_proof::get_source_id(proof_handle).unwrap_or_default();
    trace!("vcx_disclosed_proof_decline_presentation_request(command_handle: {}, proof_handle: {}, connection_handle: {}, reason: {:?}, proposal: {:?}) source_id: {}", command_handle, proof_handle, connection_handle, reason, proposal, source_id);

    execute_async::<BoxFuture<'static, Result<(), ()>>>(Box::pin(async move {
        match disclosed_proof::decline_presentation_request(
            proof_handle,
            connection_handle,
            reason.as_deref(),
            proposal.as_deref(),
        )
        .await
        {
            Ok(()) => {
                trace!(
                    "vcx_disclosed_proof_decline_presentation_request(command_handle: {}, rc: {}) source_id: {}",
                    command_handle,
                    SUCCESS_ERR_CODE,
                    source_id
                );
                cb(command_handle, SUCCESS_ERR_CODE);
            }
            Err(err) => {
                set_current_error_vcx(&err);
                error!(
                    "vcx_disclosed_proof_decline_presentation_request(command_handle: {}, rc: {}) source_id: {}",
                    command_handle, err, source_id
                );
                cb(command_handle, err.into());
            }
        };

        Ok(())
    }));

    SUCCESS_ERR_CODE
}

#[no_mangle]
pub extern "C" fn vcx_disclosed_proof_get_thread_id(
    command_handle: CommandHandle,
    proof_handle: u32,
    cb: Option<extern "C" fn(xcommand_handle: CommandHandle, err: u32, thread_id: *const c_char)>,
) -> u32 {
    info!("vcx_disclosed_proof_get_thread_id >>> proof_handle: {:?}", proof_handle);

    check_useful_c_callback!(cb, LibvcxErrorKind::InvalidOption);
    let source_id = disclosed_proof::get_source_id(proof_handle).unwrap_or_default();
    trace!(
        "vcx_disclosed_proof_get_thread_id(command_handle: {}, proof_handle: {}) source_id: {})",
        command_handle,
        proof_handle,
        source_id
    );

    execute(move || {
        match disclosed_proof::get_thread_id(proof_handle) {
            Ok(thread_id) => {
                trace!(
                    "vcx_disclosed_proof_get_thread_id_cb(commmand_handle: {}, rc: {}, thread_id: {}) source_id: {}",
                    command_handle,
                    SUCCESS_ERR_CODE,
                    thread_id,
                    source_id
                );
                let c_thread_id = CStringUtils::string_to_cstring(thread_id);
                cb(command_handle, SUCCESS_ERR_CODE, c_thread_id.as_ptr());
            }
            Err(err) => {
                error!(
                    "vcx_disclosed_proof_get_thread_id_cb(commmand_handle: {}, rc: {}, thread_id: {}) source_id: {}",
                    command_handle,
                    err,
                    "".to_string(),
                    source_id
                );
                cb(command_handle, err.into(), ptr::null_mut());
            }
        };

        Ok(())
    });

    SUCCESS_ERR_CODE
}

/// Releases the disclosed proof object by de-allocating memory
///
/// #Params
/// handle: Proof handle that was provided during creation. Used to access proof object
///
/// #Returns
/// Success
#[no_mangle]
pub extern "C" fn vcx_disclosed_proof_release(handle: u32) -> u32 {
    info!("vcx_disclosed_proof_release >>>");

    let source_id = disclosed_proof::get_source_id(handle).unwrap_or_default();
    match disclosed_proof::release(handle) {
        Ok(()) => {
            trace!(
                "vcx_disclosed_proof_release(handle: {}, rc: {}), source_id: {:?}",
                handle,
                SUCCESS_ERR_CODE,
                source_id
            );
            SUCCESS_ERR_CODE
        }
        Err(err) => {
            error!(
                "vcx_disclosed_proof_release(handle: {}, rc: {}), source_id: {:?}",
                handle, err, source_id
            );
            err.into()
        }
    }
}

#[cfg(test)]
mod tests {
    use std::ffi::CString;

    use serde_json::Value;

    use aries_vcx::agency_client::testing::mocking::AgencyMockDecrypted;
    use aries_vcx::utils::constants::{
        CREDS_FROM_PROOF_REQ, GET_MESSAGES_DECRYPTED_RESPONSE, V3_OBJECT_SERIALIZE_VERSION,
    };
    use aries_vcx::utils::devsetup::*;
    use aries_vcx::utils::mockdata::mock_settings::MockBuilder;
    use aries_vcx::utils::mockdata::mockdata_credex::ARIES_CREDENTIAL_REQUEST;
    use aries_vcx::utils::mockdata::mockdata_proof::ARIES_PROOF_REQUEST_PRESENTATION;
    use libvcx_core::api_vcx::api_handle::mediated_connection;
    use libvcx_core::api_vcx::api_handle::mediated_connection::test_utils::build_test_connection_inviter_requested;
    use libvcx_core::errors;

    use crate::api_c::cutils::return_types_u32;
    use crate::api_c::cutils::timeout::TimeoutUtils;
    use crate::aries_vcx::protocols::proof_presentation::prover::state_machine::ProverState;

    use super::*;

    pub const BAD_PROOF_REQUEST: &str = r#"{"version": "0.1","to_did": "LtMgSjtFcyPwenK9SHCyb8","from_did": "LtMgSjtFcyPwenK9SHCyb8","claim": {"account_num": ["8BEaoLf8TBmK4BUyX8WWnA"],"name_on_account": ["Alice"]},"schema_seq_no": 48,"issuer_did": "Pd4fnFtRBcMKRVC2go5w3j","claim_name": "Account Certificate","claim_id": "3675417066","msg_ref_id": "ymy5nth"}"#;

    fn _vcx_disclosed_proof_create_with_request_c_closure(proof_request: &str) -> Result<u32, u32> {
        let cb = return_types_u32::Return_U32_U32::new().unwrap();
        let rc = vcx_disclosed_proof_create_with_request(
            cb.command_handle,
            CString::new("test_create").unwrap().into_raw(),
            CString::new(proof_request).unwrap().into_raw(),
            Some(cb.get_callback()),
        );
        if rc != SUCCESS_ERR_CODE {
            return Err(rc);
        }
        cb.receive(TimeoutUtils::some_medium())
    }

    #[test]
    #[cfg(feature = "general_test")]
    fn test_vcx_proof_create_with_request_success() {
        let _setup = SetupMocks::init();

        let handle = _vcx_disclosed_proof_create_with_request_c_closure(ARIES_PROOF_REQUEST_PRESENTATION).unwrap();
        assert!(handle > 0);
    }

    #[test]
    #[cfg(feature = "general_test")]
    fn test_vcx_proof_create_with_request() {
        let _setup = SetupMocks::init();

        let err = _vcx_disclosed_proof_create_with_request_c_closure(BAD_PROOF_REQUEST).unwrap_err();
        assert_eq!(err, u32::from(LibvcxErrorKind::InvalidJson));
    }

    #[tokio::test]
    #[cfg(feature = "general_test")]
    async fn test_create_with_msgid() {
        let _setup = SetupMocks::init();

        let cxn = build_test_connection_inviter_requested().await;

        let cb = return_types_u32::Return_U32_U32_STR::new().unwrap();
        assert_eq!(
            vcx_disclosed_proof_create_with_msgid(
                cb.command_handle,
                CString::new("test_create_with_msgid").unwrap().into_raw(),
                cxn,
                CString::new("123").unwrap().into_raw(),
                Some(cb.get_callback()),
            ),
            SUCCESS_ERR_CODE
        );
        let (handle, disclosed_proof) = cb.receive(TimeoutUtils::some_medium()).unwrap();
        assert!(handle > 0 && disclosed_proof.is_some());
    }

    #[test]
    #[cfg(feature = "general_test")]
    fn test_vcx_disclosed_proof_serialize_and_deserialize() {
        let _setup = SetupMocks::init();

        let handle = _vcx_disclosed_proof_create_with_request_c_closure(ARIES_PROOF_REQUEST_PRESENTATION).unwrap();

        let cb = return_types_u32::Return_U32_STR::new().unwrap();
        assert_eq!(
            vcx_disclosed_proof_serialize(cb.command_handle, handle, Some(cb.get_callback())),
            SUCCESS_ERR_CODE
        );
        let s = cb.receive(TimeoutUtils::some_short()).unwrap().unwrap();

        let j: Value = serde_json::from_str(&s).unwrap();
        assert_eq!(j["version"], V3_OBJECT_SERIALIZE_VERSION);

        let cb = return_types_u32::Return_U32_U32::new().unwrap();
        assert_eq!(
            vcx_disclosed_proof_deserialize(
                cb.command_handle,
                CString::new(s).unwrap().into_raw(),
                Some(cb.get_callback()),
            ),
            SUCCESS_ERR_CODE
        );

        let handle = cb.receive(TimeoutUtils::some_short()).unwrap();
        assert!(handle > 0);
    }

    #[test]
    #[cfg(feature = "general_test")]
    fn test_generate_msg() {
        let _setup = SetupMocks::init();

        let handle = _vcx_disclosed_proof_create_with_request_c_closure(ARIES_PROOF_REQUEST_PRESENTATION).unwrap();
        let cb = return_types_u32::Return_U32::new().unwrap();
        assert_eq!(
            vcx_disclosed_proof_generate_proof(
                cb.command_handle,
                handle,
                CString::new("{}").unwrap().into_raw(),
                CString::new("{}").unwrap().into_raw(),
                Some(cb.get_callback()),
            ),
            SUCCESS_ERR_CODE
        );
        let _s = cb.receive(TimeoutUtils::some_medium()).unwrap();

        let cb = return_types_u32::Return_U32_STR::new().unwrap();
        assert_eq!(
            vcx_disclosed_proof_get_proof_msg(cb.command_handle, handle, Some(cb.get_callback())),
            SUCCESS_ERR_CODE
        );
        let _s = cb.receive(TimeoutUtils::some_medium()).unwrap();
    }

    #[tokio::test]
    #[cfg(feature = "general_test")]
    async fn test_vcx_send_proof() {
        let _setup = SetupMocks::init();

        let handle_proof =
            _vcx_disclosed_proof_create_with_request_c_closure(ARIES_PROOF_REQUEST_PRESENTATION).unwrap();
        assert_eq!(
            disclosed_proof::get_state(handle_proof).unwrap(),
            ProverState::PresentationRequestReceived as u32
        );

        let handle_conn = build_test_connection_inviter_requested().await;

        let cb = return_types_u32::Return_U32::new().unwrap();
        assert_eq!(
            vcx_disclosed_proof_send_proof(cb.command_handle, handle_proof, handle_conn, Some(cb.get_callback())),
            SUCCESS_ERR_CODE
        );
        cb.receive(TimeoutUtils::some_medium()).unwrap();
    }

    #[tokio::test]
    #[cfg(feature = "general_test")]
    async fn test_vcx_reject_proof_request() {
        let _setup = SetupMocks::init();

        let handle = _vcx_disclosed_proof_create_with_request_c_closure(ARIES_PROOF_REQUEST_PRESENTATION).unwrap();
        assert_eq!(
            disclosed_proof::get_state(handle).unwrap(),
            ProverState::PresentationRequestReceived as u32
        );

        let connection_handle = build_test_connection_inviter_requested().await;

        let cb = return_types_u32::Return_U32::new().unwrap();
        assert_eq!(
            vcx_disclosed_proof_reject_proof(cb.command_handle, handle, connection_handle, Some(cb.get_callback())),
            SUCCESS_ERR_CODE
        );
        cb.receive(TimeoutUtils::some_medium()).unwrap();
    }

    #[tokio::test]
    #[cfg(feature = "to_restore")]
    #[cfg(feature = "general_test")] // todo: generate_reject_proof_msg is not implemented for aries
    async fn test_vcx_get_reject_msg() {
        let _setup = SetupMocks::init();

        let handle = _vcx_disclosed_proof_create_with_request_c_closure(ARIES_PROOF_REQUEST_PRESENTATION).unwrap();

        assert_eq!(
            disclosed_proof::get_state(handle).unwrap(),
            ProverState::PresentationRequestReceived as u32
        );

        let _connection_handle = build_test_connection_inviter_requested().await;

        let cb = return_types_u32::Return_U32_STR::new().unwrap();
        assert_eq!(
            vcx_disclosed_proof_get_reject_msg(cb.command_handle, handle, Some(cb.get_callback())),
            SUCCESS_ERR_CODE
        );
        cb.receive(TimeoutUtils::some_medium()).unwrap();
    }

    #[tokio::test]
    #[cfg(feature = "general_test")]
    async fn test_vcx_proof_get_requests() {
        let _setup = SetupMocks::init();

        let cxn = build_test_connection_inviter_requested().await;

        AgencyMockDecrypted::set_next_decrypted_response(GET_MESSAGES_DECRYPTED_RESPONSE);
        AgencyMockDecrypted::set_next_decrypted_message(ARIES_CREDENTIAL_REQUEST);

        let cb = return_types_u32::Return_U32_STR::new().unwrap();
        assert_eq!(
            vcx_disclosed_proof_get_requests(cb.command_handle, cxn, Some(cb.get_callback())),
            SUCCESS_ERR_CODE as u32
        );
        cb.receive(TimeoutUtils::some_medium()).unwrap();
    }

    #[test]
    #[cfg(feature = "general_test")]
    fn test_vcx_proof_get_state() {
        let _setup = SetupMocks::init();

        let handle = _vcx_disclosed_proof_create_with_request_c_closure(ARIES_PROOF_REQUEST_PRESENTATION).unwrap();

        let cb = return_types_u32::Return_U32_U32::new().unwrap();
        assert_eq!(
            vcx_disclosed_proof_get_state(cb.command_handle, handle, Some(cb.get_callback())),
            SUCCESS_ERR_CODE
        );
        let state = cb.receive(TimeoutUtils::some_medium()).unwrap();
        assert_eq!(state, ProverState::PresentationRequestReceived as u32);
    }

    #[test]
    #[cfg(feature = "general_test")]
    fn test_vcx_disclosed_proof_retrieve_credentials() {
        let _setup = SetupMocks::init();
        let _mock_builder = MockBuilder::init().set_mock_creds_retrieved_for_proof_request(CREDS_FROM_PROOF_REQ);

        let proof_handle =
            _vcx_disclosed_proof_create_with_request_c_closure(ARIES_PROOF_REQUEST_PRESENTATION).unwrap();

        let cb = return_types_u32::Return_U32_STR::new().unwrap();
        assert_eq!(
            vcx_disclosed_proof_retrieve_credentials(cb.command_handle, proof_handle, Some(cb.get_callback())),
            SUCCESS_ERR_CODE
        );
        let _credentials = cb.receive(None).unwrap().unwrap();
    }

    #[test]
    #[cfg(feature = "general_test")]
    fn test_vcx_disclosed_proof_generate_proof() {
        let _setup = SetupMocks::init();

        let handle = _vcx_disclosed_proof_create_with_request_c_closure(ARIES_PROOF_REQUEST_PRESENTATION).unwrap();

        let cb = return_types_u32::Return_U32::new().unwrap();
        assert_eq!(
            vcx_disclosed_proof_generate_proof(
                cb.command_handle,
                handle,
                CString::new("{}").unwrap().into_raw(),
                CString::new("{}").unwrap().into_raw(),
                Some(cb.get_callback()),
            ),
            SUCCESS_ERR_CODE
        );
        cb.receive(TimeoutUtils::some_medium()).unwrap();
    }
}
