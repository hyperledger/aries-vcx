use std::ptr;

use futures::future::BoxFuture;
use libc::c_char;

use aries_vcx::error::{VcxError, VcxErrorKind};
use aries_vcx::indy_sys::CommandHandle;
use aries_vcx::libindy;
use aries_vcx::utils::error;

use crate::api_lib::api_handle::connection::*;
use crate::api_lib::api_handle::connection;
use crate::api_lib::global::wallet::get_main_wallet_handle;
use crate::api_lib::utils;
use crate::api_lib::utils::cstring::CStringUtils;
use crate::api_lib::utils::error::set_current_error_vcx;
use crate::api_lib::utils::runtime::{execute, execute_async};

/*
    Tha API represents a pairwise connection with another identity owner.
    Once the connection, is established communication can happen securely and privately.
    Credentials and Presentations are exchanged using this object.

    # States

    The set of object states, messages and transitions depends on the communication method is used.
    The communication method can be specified as a config option on one of *_init functions.

    aries:
        Inviter:
            VcxStateType::VcxStateInitialized - once `vcx_connection_create` (create Connection object) is called.

            VcxStateType::VcxStateOfferSent - once `vcx_connection_connect` (prepared Connection invite) is called.

            VcxStateType::VcxStateRequestReceived - once `ConnectionRequest` messages is received.
                                                    accept `ConnectionRequest` and send `ConnectionResponse` message.
                                                    use `vcx_connection_update_state` or `vcx_connection_update_state_with_message` functions for state updates.

            VcxStateType::VcxStateAccepted - once `Ack` messages is received.
                                             use `vcx_connection_update_state` or `vcx_connection_update_state_with_message` functions for state updates.

            VcxStateType::VcxStateNone - once `vcx_connection_delete_connection` (delete Connection object) is called
                                            OR
                                        `ConnectionProblemReport` messages is received on state updates.

        Invitee:
            VcxStateType::VcxStateOfferSent - once `vcx_connection_create_with_invite` (create Connection object with invite) is called.

            VcxStateType::VcxStateRequestReceived - once `vcx_connection_connect` (accept `ConnectionInvite` and send `ConnectionRequest` message) is called.

            VcxStateType::VcxStateAccepted - once `ConnectionResponse` messages is received.
                                             send `Ack` message if requested.
                                             use `vcx_connection_update_state` or `vcx_connection_update_state_with_message` functions for state updates.

            VcxStateType::VcxStateNone - once `vcx_connection_delete_connection` (delete Connection object) is called
                                            OR
                                        `ConnectionProblemReport` messages is received on state updates.

    # Transitions

    aries - RFC: https://github.com/hyperledger/aries-rfcs/tree/7b6b93acbaf9611d3c892c4bada142fe2613de6e/features/0036-issue-credential
        Inviter:
            VcxStateType::None - `vcx_connection_create` - VcxStateType::VcxStateInitialized

            VcxStateType::VcxStateInitialized - `vcx_connection_connect` - VcxStateType::VcxStateOfferSent

            VcxStateType::VcxStateOfferSent - received `ConnectionRequest` - VcxStateType::VcxStateRequestReceived
            VcxStateType::VcxStateOfferSent - received `ConnectionProblemReport` - VcxStateType::VcxStateNone

            VcxStateType::VcxStateRequestReceived - received `Ack` - VcxStateType::VcxStateAccepted
            VcxStateType::VcxStateRequestReceived - received `ConnectionProblemReport` - VcxStateType::VcxStateNone

            VcxStateType::VcxStateAccepted - received `Ping`, `PingResponse`, `Query`, `Disclose` - VcxStateType::VcxStateAccepted

            any state - `vcx_connection_delete_connection` - VcxStateType::VcxStateNone


        Invitee:
            VcxStateType::None - `vcx_connection_create_with_invite` - VcxStateType::VcxStateOfferSent

            VcxStateType::VcxStateOfferSent - `vcx_connection_connect` - VcxStateType::VcxStateRequestReceived
            VcxStateType::VcxStateOfferSent - received `ConnectionProblemReport` - VcxStateType::VcxStateNone

            VcxStateType::VcxStateRequestReceived - received `ConnectionResponse` - VcxStateType::VcxStateAccepted
            VcxStateType::VcxStateRequestReceived - received `ConnectionProblemReport` - VcxStateType::VcxStateNone

            VcxStateType::VcxStateAccepted - received `Ping`, `PingResponse`, `Query`, `Disclose` - VcxStateType::VcxStateAccepted

            any state - `vcx_connection_delete_connection` - VcxStateType::VcxStateNone

    # Messages

    aries:
        Invitation - https://github.com/hyperledger/aries-rfcs/tree/master/features/0160-connection-protocol#0-invitation-to-connect
        ConnectionRequest - https://github.com/hyperledger/aries-rfcs/tree/master/features/0160-connection-protocol#1-connection-request
        ConnectionResponse - https://github.com/hyperledger/aries-rfcs/tree/master/features/0160-connection-protocol#2-connection-response
        ConnectionProblemReport - https://github.com/hyperledger/aries-rfcs/tree/master/features/0160-connection-protocol#error-message-example
        Ack - https://github.com/hyperledger/aries-rfcs/tree/master/features/0015-acks#explicit-acks
        Ping - https://github.com/hyperledger/aries-rfcs/tree/master/features/0048-trust-ping#messages
        PingResponse - https://github.com/hyperledger/aries-rfcs/tree/master/features/0048-trust-ping#messages
        Query - https://github.com/hyperledger/aries-rfcs/tree/master/features/0031-discover-features#query-message-type
        Disclose - https://github.com/hyperledger/aries-rfcs/tree/master/features/0031-discover-features#disclose-message-type
*/

#[no_mangle]
pub extern fn vcx_generate_public_invite(command_handle: CommandHandle,
                                         public_did: *const c_char,
                                         label: *const c_char,
                                         cb: Option<extern fn(xcommand_handle: CommandHandle, err: u32, public_invite: *const c_char)>) -> u32 {
    info!("vcx_generate_public_invite >>> ");

    check_useful_c_callback!(cb, VcxErrorKind::InvalidOption);
    check_useful_c_str!(public_did, VcxErrorKind::InvalidOption);
    check_useful_c_str!(label, VcxErrorKind::InvalidOption);

    trace!("vcx_generate_public_invite(command_handle: {}, public_did: {}, label: {})", command_handle, public_did, label);

    execute(move || {
        match connection::generate_public_invitation(&public_did, &label) {
            Ok(public_invite) => {
                trace!("vcx_generate_public_invite_cb(command_handle: {}, rc: {}, public_invite: {})",
                       command_handle, error::SUCCESS.message, public_invite);
                let public_invite = CStringUtils::string_to_cstring(public_invite);
                cb(command_handle, error::SUCCESS.code_num, public_invite.as_ptr());
            }
            Err(err) => {
                set_current_error_vcx(&err);
                error!("vcx_generate_public_invite_cb(command_handle: {}, rc: {}, public_invite: {})",
                      command_handle, err, 0);
                cb(command_handle, err.into(), ptr::null());
            }
        }
        Ok(())
    });

    error::SUCCESS.code_num
}

/// Delete a Connection object from the agency and release its handle.
///
/// NOTE: This eliminates the connection and any ability to use it for any communication.
///
/// # Params
/// command_handle: command handle to map callback to user context.
///
/// connection_handle: handle of the connection to delete.
///
/// cb: Callback that provides feedback of the api call.
///
/// # Returns
/// Error code as a u32
#[no_mangle]
#[allow(unused_assignments)]
pub extern fn vcx_connection_delete_connection(command_handle: CommandHandle,
                                               connection_handle: u32,
                                               cb: Option<extern fn(
                                                   xcommand_handle: CommandHandle,
                                                   err: u32)>) -> u32 {
    info!("vcx_delete_connection >>>");

    check_useful_c_callback!(cb, VcxErrorKind::InvalidOption);

    trace!("vcx_connection_delete_connection(command_handle: {}, connection_handle: {})", command_handle, connection_handle);
    execute_async::<BoxFuture<'static, Result<(), ()>>>(Box::pin(async move {
        match delete_connection(connection_handle).await {
            Ok(_) => {
                trace!("vcx_connection_delete_connection_cb(command_handle: {}, rc: {})", command_handle, error::SUCCESS.message);
                cb(command_handle, error::SUCCESS.code_num);
            }
            Err(err) => {
                set_current_error_vcx(&err);
                trace!("vcx_connection_delete_connection_cb(command_handle: {}, rc: {})", command_handle, err);
                cb(command_handle, err.into());
            }
        }

        Ok(())
    }));

    error::SUCCESS.code_num
}

/// Create a Connection object that provides a pairwise connection for an institution's user
///
/// # Params
/// command_handle: command handle to map callback to user context.
///
/// source_id: institution's personal identification for the connection
///
/// cb: Callback that provides connection handle and error status of request
///
/// # Returns
/// Error code as a u32
#[no_mangle]
#[allow(unused_assignments)]
pub extern fn vcx_connection_create(command_handle: CommandHandle,
                                    source_id: *const c_char,
                                    cb: Option<extern fn(xcommand_handle: CommandHandle, err: u32, connection_handle: u32)>) -> u32 {
    info!("vcx_connection_create >>>");

    check_useful_c_callback!(cb, VcxErrorKind::InvalidOption);
    check_useful_c_str!(source_id, VcxErrorKind::InvalidOption);

    trace!("vcx_connection_create(command_handle: {}, source_id: {})", command_handle, source_id);

    execute_async::<BoxFuture<'static, Result<(), ()>>>(Box::pin(async move {
        match create_connection(&source_id).await {
            Ok(handle) => {
                trace!("vcx_connection_create_cb(command_handle: {}, rc: {}, handle: {}) source_id: {}",
                       command_handle, error::SUCCESS.message, handle, source_id);
                cb(command_handle, error::SUCCESS.code_num, handle);
            }
            Err(err) => {
                set_current_error_vcx(&err);
                error!("vcx_connection_create_cb(command_handle: {}, rc: {}, handle: {}) source_id: {}",
                      command_handle, err, 0, source_id);
                cb(command_handle, err.into(), 0);
            }
        };

        Ok(())
    }));

    error::SUCCESS.code_num
}

/// Create a Connection object from the given invite_details that provides a pairwise connection.
///
/// # Params
/// command_handle: command handle to map callback to user context.
///
/// source_id: institution's personal identification for the connection
///
/// invite_details: A string representing a json object which is provided by an entity that wishes to make a connection.
///
/// cb: Callback that provides connection handle and error status of request
///
/// # Examples
/// invite_details -> depends on communication method:
///     aries: https://github.com/hyperledger/aries-rfcs/tree/master/features/0160-connection-protocol#0-invitation-to-connect
///      {
///         "@type": "did:sov:BzCbsNYhMrjHiqZDTUASHg;spec/connections/1.0/invitation",
///         "label": "Alice",
///         "recipientKeys": ["8HH5gYEeNc3z7PYXmd54d4x6qAfCNrqQqEB3nS7Zfu7K"],
///         "serviceEndpoint": "https://example.com/endpoint",
///         "routingKeys": ["8HH5gYEeNc3z7PYXmd54d4x6qAfCNrqQqEB3nS7Zfu7K"]
///      }
///
/// # Returns
/// Error code as a u32
#[no_mangle]
pub extern fn vcx_connection_create_with_invite(command_handle: CommandHandle,
                                                source_id: *const c_char,
                                                invite_details: *const c_char,
                                                cb: Option<extern fn(xcommand_handle: CommandHandle, err: u32, connection_handle: u32)>) -> u32 {
    info!("vcx_connection_create_with_invite >>>");

    check_useful_c_callback!(cb, VcxErrorKind::InvalidOption);
    check_useful_c_str!(source_id, VcxErrorKind::InvalidOption);
    check_useful_c_str!(invite_details, VcxErrorKind::InvalidOption);
    trace!("vcx_connection_create_with_invite(command_handle: {}, source_id: {})", command_handle, source_id);
    execute_async::<BoxFuture<'static, Result<(), ()>>>(Box::pin(async move {
        match create_connection_with_invite(&source_id, &invite_details).await {
            Ok(handle) => {
                trace!("vcx_connection_create_with_invite_cb(command_handle: {}, rc: {}, handle: {}) source_id: {}",
                       command_handle, error::SUCCESS.message, handle, source_id);
                cb(command_handle, error::SUCCESS.code_num, handle);
            }
            Err(err) => {
                set_current_error_vcx(&err);
                error!("vcx_connection_create_with_invite_cb(command_handle: {}, rc: {}, handle: {}) source_id: {}",
                      command_handle, err, 0, source_id);
                cb(command_handle, err.into(), 0);
            }
        };

        Ok(())
    }));

    error::SUCCESS.code_num
}

#[no_mangle]
pub extern fn vcx_connection_create_with_connection_request(command_handle: CommandHandle,
                                                            source_id: *const c_char,
                                                            agent_handle: u32,
                                                            request: *const c_char,
                                                            cb: Option<extern fn(xcommand_handle: CommandHandle, err: u32, connection_handle: u32)>) -> u32 {
    info!("vcx_connection_create_with_connection_request >>>");

    check_useful_c_callback!(cb, VcxErrorKind::InvalidOption);
    check_useful_c_str!(source_id, VcxErrorKind::InvalidOption);
    check_useful_c_str!(request, VcxErrorKind::InvalidOption);

    trace!("vcx_connection_create_with_connection_request(command_handle: {}, agent_handle: {}, request: {}) source_id: {}", command_handle, agent_handle, request, source_id);

    execute_async::<BoxFuture<'static, Result<(), ()>>>(Box::pin(async move {
        match create_with_request(&request, agent_handle).await {
            Ok(handle) => {
                trace!("vcx_connection_create_with_connection_request_cb(command_handle: {}, rc: {}, handle: {:?}) source_id: {}",
                       command_handle, error::SUCCESS.message, handle, source_id);
                cb(command_handle, error::SUCCESS.code_num, handle);
            }
            Err(err) => {
                set_current_error_vcx(&err);
                error!("vcx_connection_create_with_connection_request_cb(command_handle: {}, rc: {}, handle: {}) source_id: {}",
                      command_handle, err, 0, source_id);
                cb(command_handle, err.into(), 0);
            }
        };

        Ok(())
    }));

    error::SUCCESS.code_num
}

/// Establishes connection between institution and its user
///
/// # Params
/// command_handle: command handle to map callback to user context.
///
/// connection_handle: Connection handle that identifies connection object
///
/// connection_options: Provides details indicating if the connection will be established by text or QR Code
///
/// # Examples connection_options ->
/// "{"connection_type":"SMS","phone":"123","use_public_did":true}"
///     OR:
/// "{"connection_type":"QR","phone":"","use_public_did":false}"
///
/// cb: Callback that provides error status of request
///
/// #Returns
/// Error code as a u32
#[no_mangle]
pub extern fn vcx_connection_connect(command_handle: CommandHandle,
                                     connection_handle: u32,
                                     connection_options: *const c_char,
                                     cb: Option<extern fn(xcommand_handle: CommandHandle, err: u32, invite_details: *const c_char)>) -> u32 {
    info!("vcx_connection_connect >>>");

    check_useful_c_callback!(cb, VcxErrorKind::InvalidOption);

    let _options = if !connection_options.is_null() {
        check_useful_opt_c_str!(connection_options, VcxErrorKind::InvalidOption);
        connection_options.to_owned()
    } else {
        None
    };

    let source_id = get_source_id(connection_handle).unwrap_or_default();
    trace!("vcx_connection_connect(command_handle: {}, connection_handle: {}, source_id: {:?}",
           command_handle, connection_handle, source_id);

    execute_async::<BoxFuture<'static, Result<(), ()>>>(Box::pin(async move {
        match connect(connection_handle).await {
            Ok(invitation) => {
                let invitation = invitation.unwrap_or(String::from("{}"));
                trace!("vcx_connection_connect_cb(command_handle: {}, connection_handle: {}, rc: {}, details: {}), source_id: {}",
                       command_handle, connection_handle, error::SUCCESS.message, invitation, source_id);
                let invitation = CStringUtils::string_to_cstring(invitation);
                cb(command_handle, error::SUCCESS.code_num, invitation.as_ptr());
            }
            Err(err) => {
                set_current_error_vcx(&err);
                error!("vcx_connection_connect_cb(command_handle: {}, connection_handle: {}, rc: {}, details: {}, source_id: {})",
                      command_handle, connection_handle, err, "null", source_id);
                cb(command_handle, err.into(), ptr::null_mut());
            }
        };

        Ok(())
    }));

    error::SUCCESS.code_num
}

#[no_mangle]
pub extern fn vcx_connection_get_thread_id(command_handle: CommandHandle,
                                           connection_handle: u32,
                                           cb: Option<extern fn(xcommand_handle: CommandHandle, err: u32, thread_id: *const c_char)>) -> u32 {
    info!("vcx_connection_get_thread_id >>>");

    check_useful_c_callback!(cb, VcxErrorKind::InvalidOption);

    let source_id = get_source_id(connection_handle).unwrap_or_default();
    trace!("vcx_connection_get_thread_id(command_handle: {}, connection_handle: {}), source_id: {:?}",
           command_handle, connection_handle, source_id);

    execute(move || {
        match get_thread_id(connection_handle) {
            Ok(tid) => {
                trace!("vcx_connection_get_thread_id_cb(command_handle: {}, connection_handle: {}, rc: {}, thread_id: {}), source_id: {:?}",
                       command_handle, connection_handle, error::SUCCESS.message, tid, source_id);
                let tid = CStringUtils::string_to_cstring(tid);
                cb(command_handle, error::SUCCESS.code_num, tid.as_ptr());
            }
            Err(err) => {
                set_current_error_vcx(&err);
                error!("vcx_connection_get_thread_id_cb(command_handle: {}, connection_handle: {}, rc: {}, thread_id: {}), source_id: {:?}",
                      command_handle, connection_handle, err, "null", source_id);
                cb(command_handle, err.into(), ptr::null_mut());
            }
        };

        Ok(())
    });

    error::SUCCESS.code_num
}

/// Takes the Connection object and returns a json string of all its attributes
///
/// # Params
/// command_handle: command handle to map callback to user context.
///
/// connection_handle: Connection handle that identifies pairwise connection
///
/// cb: Callback that provides json string of the connection's attributes and provides error status
///
/// #Returns
/// Error code as a u32
#[no_mangle]
pub extern fn vcx_connection_serialize(command_handle: CommandHandle,
                                       connection_handle: u32,
                                       cb: Option<extern fn(xcommand_handle: CommandHandle, err: u32, serialized_data: *const c_char)>) -> u32 {
    info!("vcx_connection_serialize >>>");

    check_useful_c_callback!(cb, VcxErrorKind::InvalidOption);

    let source_id = get_source_id(connection_handle).unwrap_or_default();
    trace!("vcx_connection_serialize(command_handle: {}, connection_handle: {}), source_id: {:?}",
           command_handle, connection_handle, source_id);

    execute(move || {
        match to_string(connection_handle) {
            Ok(json) => {
                trace!("vcx_connection_serialize_cb(command_handle: {}, connection_handle: {}, rc: {}, state: {}), source_id: {:?}",
                       command_handle, connection_handle, error::SUCCESS.message, json, source_id);
                let msg = CStringUtils::string_to_cstring(json);
                cb(command_handle, error::SUCCESS.code_num, msg.as_ptr());
            }
            Err(err) => {
                set_current_error_vcx(&err);
                error!("vcx_connection_serialize_cb(command_handle: {}, connection_handle: {}, rc: {}, state: {}), source_id: {:?}",
                      command_handle, connection_handle, err, "null", source_id);
                cb(command_handle, err.into(), ptr::null_mut());
            }
        };

        Ok(())
    });

    error::SUCCESS.code_num
}

/// Takes a json string representing a connection object and recreates an object matching the json
///
/// #Params
/// command_handle: command handle to map callback to user context.
///
/// connection_data: json string representing a connection object. Is an output of `vcx_connection_serialize` function.
///
/// cb: Callback that provides credential handle and provides error status
///
/// #Returns
/// Error code as a u32
#[no_mangle]
pub extern fn vcx_connection_deserialize(command_handle: CommandHandle,
                                         connection_data: *const c_char,
                                         cb: Option<extern fn(xcommand_handle: CommandHandle, err: u32, connection_handle: u32)>) -> u32 {
    info!("vcx_connection_deserialize >>>");

    check_useful_c_callback!(cb, VcxErrorKind::InvalidOption);
    check_useful_c_str!(connection_data, VcxErrorKind::InvalidOption);

    trace!("vcx_connection_deserialize(command_handle: {}, connection_data: {})", command_handle, connection_data);

    execute(move || {
        let (rc, handle) = match from_string(&connection_data) {
            Ok(err) => {
                let source_id = get_source_id(err).unwrap_or_default();
                trace!("vcx_connection_deserialize_cb(command_handle: {}, rc: {}, handle: {}), source_id: {:?}",
                       command_handle, error::SUCCESS.message, err, source_id);
                (error::SUCCESS.code_num, err)
            }
            Err(err) => {
                set_current_error_vcx(&err);
                error!("vcx_connection_deserialize_cb(command_handle: {}, rc: {}, handle: {} )",
                      command_handle, err, 0);
                (err.into(), 0)
            }
        };

        cb(command_handle, rc, handle);

        Ok(())
    });

    error::SUCCESS.code_num
}

/// Query the agency for the received messages.
/// Checks for any messages changing state in the connection and updates the state attribute.
///
/// #Params
/// command_handle: command handle to map callback to user context.
///
/// connection_handle: was provided during creation. Used to identify connection object
///
/// cb: Callback that provides most current state of the credential and error status of request
///     Connection states:
///         1 - Initialized
///         2 - Request Sent
///         3 - Offer Received
///         4 - Accepted
///
/// #Returns
/// Error code as a u32
#[no_mangle]
pub extern fn vcx_connection_update_state(command_handle: CommandHandle,
                                          connection_handle: u32,
                                          cb: Option<extern fn(xcommand_handle: CommandHandle, err: u32, state: u32)>) -> u32 {
    info!("vcx_connection_update_state >>>");

    check_useful_c_callback!(cb, VcxErrorKind::InvalidOption);

    let source_id = get_source_id(connection_handle).unwrap_or_default();
    trace!("vcx_connection_update_state(command_handle: {}, connection_handle: {}, source_id: {:?}",
           command_handle, connection_handle, source_id);

    execute_async::<BoxFuture<'static, Result<(), ()>>>(Box::pin(async move {
        let rc = match update_state(connection_handle).await {
            Ok(err) => {
                trace!("vcx_connection_update_state_cb(command_handle: {}, rc: {}, connection_handle: {}, state: {}), source_id: {:?}",
                       command_handle, error::SUCCESS.message, connection_handle, get_state(connection_handle), source_id);
                err
            }
            Err(err) => {
                set_current_error_vcx(&err);
                error!("vcx_connection_update_state_cb(command_handle: {}, rc: {}, connection_handle: {}, state: {}), source_id: {:?}",
                      command_handle, err, connection_handle, get_state(connection_handle), source_id);
                err.into()
            }
        };
        let state = get_state(connection_handle);
        warn!("vcx_connection_update_state >> return {}", state);
        cb(command_handle, rc, state);

        Ok(())
    }));

    error::SUCCESS.code_num
}

/// Update the state of the connection based on the given message.
///
/// #Params
/// command_handle: command handle to map callback to user context.
///
/// connection_handle: was provided during creation. Used to identify connection object
///
/// message: message to process.
///
/// cb: Callback that provides most current state of the connection and error status of request
///
/// #Returns
/// Error code as a u32
#[no_mangle]
pub extern fn vcx_connection_update_state_with_message(command_handle: CommandHandle,
                                                       connection_handle: u32,
                                                       message: *const c_char,
                                                       cb: Option<extern fn(xcommand_handle: CommandHandle, err: u32, state: u32)>) -> u32 {
    info!("vcx_connection_update_state_with_message >>>");

    check_useful_c_callback!(cb, VcxErrorKind::InvalidOption);
    check_useful_c_str!(message, VcxErrorKind::InvalidOption);

    let source_id = get_source_id(connection_handle).unwrap_or_default();
    trace!("vcx_connection_update_state_with_message(command_handle: {}, connection_handle: {}), source_id: {:?}",
           command_handle, connection_handle, source_id);

    execute_async::<BoxFuture<'static, Result<(), ()>>>(Box::pin(async move {
        let rc = match update_state_with_message(connection_handle, &message).await {
            Ok(err) => {
                trace!("vcx_connection_update_state_with_message_cb(command_handle: {}, rc: {}, connection_handle: {}, state: {}), source_id: {:?}",
                       command_handle, error::SUCCESS.message, connection_handle, get_state(connection_handle), source_id);
                err
            }
            Err(err) => {
                set_current_error_vcx(&err);
                error!("vcx_connection_update_state_with_message_cb(command_handle: {}, rc: {}, connection_handle: {}, state: {}), source_id: {:?}",
                      command_handle, err, connection_handle, get_state(connection_handle), source_id);
                err.into()
            }
        };

        let state = get_state(connection_handle);
        cb(command_handle, rc, state);

        Ok(())
    }));

    error::SUCCESS.code_num
}

/// Returns the current internal state of the connection. Does NOT query agency for state updates.
///     Possible states:
///         1 - Initialized
///         2 - Offer Sent
///         3 - Request Received
///         4 - Accepted
///
/// #Params
/// command_handle: command handle to map callback to user context.
///
/// connection_handle: Connection handle that was provided during creation. Used to access connection object
///
/// cb: Callback that provides most current state of the connection and error status of request
///
/// #Returns
#[no_mangle]
pub extern fn vcx_connection_get_state(command_handle: CommandHandle,
                                       connection_handle: u32,
                                       cb: Option<extern fn(xcommand_handle: CommandHandle, err: u32, state: u32)>) -> u32 {
    info!("vcx_connection_get_state >>>");

    check_useful_c_callback!(cb, VcxErrorKind::InvalidOption);

    let source_id = get_source_id(connection_handle).unwrap_or_default();
    trace!("vcx_connection_get_state(command_handle: {}, connection_handle: {}), source_id: {:?}",
           command_handle, connection_handle, source_id);

    execute_async::<BoxFuture<'static, Result<(), ()>>>(Box::pin(async move {
        trace!("vcx_connection_get_state_cb(command_handle: {}, rc: {}, connection_handle: {}, state: {}), source_id: {:?}",
               command_handle, error::SUCCESS.message, connection_handle, get_state(connection_handle), source_id);
        cb(command_handle, error::SUCCESS.code_num, get_state(connection_handle));

        Ok(())
    }));

    error::SUCCESS.code_num
}

/// Get the invite details that were sent or can be sent to the remote side.
///
/// #Params
/// command_handle: command handle to map callback to user context.
///
/// connection_handle: was provided during creation. Used to identify connection object
///
/// abbreviated: deprecated, has not effect
///
/// cb: Callback that provides the json string of details
///
/// # Example
/// details -> depends on communication method:
///     aries:
///      {
///         "label": "Alice",
///         "serviceEndpoint": "https://example.com/endpoint",
///         "recipientKeys": ["8HH5gYEeNc3z7PYXmd54d4x6qAfCNrqQqEB3nS7Zfu7K"],
///         "routingKeys": ["8HH5gYEeNc3z7PYXmd54d4x6qAfCNrqQqEB3nS7Zfu7K"],
///         "protocols": [
///             {"pid": "did:sov:BzCbsNYhMrjHiqZDTUASHg;spec/connections/1.0", "roles": "Invitee"},
///             ...
///         ] - optional array. The set of protocol supported by remote side. Is filled after DiscoveryFeatures process was completed.
/////    }
///
/// #Returns
/// Error code as a u32
#[no_mangle]
pub extern fn vcx_connection_invite_details(command_handle: CommandHandle,
                                            connection_handle: u32,
                                            _abbreviated: bool,
                                            cb: Option<extern fn(xcommand_handle: CommandHandle, err: u32, details: *const c_char)>) -> u32 {
    info!("vcx_connection_invite_details >>>");

    check_useful_c_callback!(cb, VcxErrorKind::InvalidOption);

    let source_id = get_source_id(connection_handle).unwrap_or_default();
    trace!("vcx_connection_invite_details(command_handle: {}, connection_handle: {}), source_id: {:?}",
           command_handle, connection_handle, source_id);

    execute(move || {
        match get_invite_details(connection_handle) {
            Ok(str) => {
                trace!("vcx_connection_invite_details_cb(command_handle: {}, connection_handle: {}, rc: {}, details: {}), source_id: {:?}",
                       command_handle, connection_handle, error::SUCCESS.message, str, source_id);
                let msg = CStringUtils::string_to_cstring(str);
                cb(command_handle, error::SUCCESS.code_num, msg.as_ptr());
            }
            Err(err) => {
                set_current_error_vcx(&err);
                error!("vcx_connection_invite_details_cb(command_handle: {}, connection_handle: {}, rc: {}, details: {}, source_id: {:?})",
                      command_handle, connection_handle, err, "null", source_id);
                cb(command_handle, err.into(), ptr::null_mut());
            }
        };

        Ok(())
    });

    error::SUCCESS.code_num
}

/// Send a message to the specified connection
///
/// #params
///
/// command_handle: command handle to map callback to user context.
///
/// connection_handle: connection to use to send the message.
///                    Was provided during creation. Used to identify connection object.
///                    Note that connection must be in Accepted state.
///
/// msg: actual message to send
///
/// send_msg_options: deprecated, has not effect
///
/// cb: Callback that provides id of retrieved response message
///
/// #Returns
/// Error code as a u32
#[no_mangle]
pub extern fn vcx_connection_send_message(command_handle: CommandHandle,
                                          connection_handle: u32,
                                          msg: *const c_char,
                                          _send_msg_options: *const c_char,
                                          cb: Option<extern fn(xcommand_handle: CommandHandle, err: u32, msg_id: *const c_char)>) -> u32 {
    info!("vcx_connection_send_message >>>");

    check_useful_c_callback!(cb, VcxErrorKind::InvalidOption);
    check_useful_c_str!(msg, VcxErrorKind::InvalidOption);

    trace!("vcx_connection_send_message(command_handle: {}, connection_handle: {}, msg: {})",
           command_handle, connection_handle, msg);

    execute_async::<BoxFuture<'static, Result<(), ()>>>(Box::pin(async move {
        match send_generic_message(connection_handle, &msg).await {
            Ok(err) => {
                trace!("vcx_connection_send_message_cb(command_handle: {}, rc: {}, msg_id: {})",
                       command_handle, error::SUCCESS.message, err);

                let msg_id = CStringUtils::string_to_cstring(err);
                cb(command_handle, error::SUCCESS.code_num, msg_id.as_ptr());
            }
            Err(err) => {
                error!("vcx_connection_send_message_cb(command_handle: {}, rc: {})",
                      command_handle, err);

                cb(command_handle, err.into(), ptr::null_mut());
            }
        };

        Ok(())
    }));

    error::SUCCESS.code_num
}

/// Send trust ping message to the specified connection to prove that two agents have a functional pairwise channel.
///
/// Note that this function is useful in case `aries` communication method is used.
/// In other cases it returns ActionNotSupported error.
///
/// #params
///
/// command_handle: command handle to map callback to user context.
///
/// connection_handle: connection to use to send ping message.
///                    Was provided during creation. Used to identify connection object.
///                    Note that connection must be in Accepted state.
///
/// comment: (Optional) human-friendly description of the ping.
///
/// cb: Callback that provides success or failure of request
///
/// #Returns
/// Error code as a u32
#[no_mangle]
pub extern fn vcx_connection_send_ping(command_handle: u32,
                                       connection_handle: u32,
                                       comment: *const c_char,
                                       cb: Option<extern fn(xcommand_handle: u32, err: u32)>) -> u32 {
    info!("vcx_connection_send_ping >>>");

    check_useful_opt_c_str!(comment, VcxErrorKind::InvalidOption);
    check_useful_c_callback!(cb, VcxErrorKind::InvalidOption);

    trace!("vcx_connection_send_ping(command_handle: {}, connection_handle: {}, comment: {:?})",
           command_handle, connection_handle, comment);

    execute_async::<BoxFuture<'static, Result<(), ()>>>(Box::pin(async move {
        match send_ping(connection_handle, comment.as_deref()).await {
            Ok(()) => {
                trace!("vcx_connection_send_ping(command_handle: {}, rc: {})",
                       command_handle, error::SUCCESS.message);
                cb(command_handle, error::SUCCESS.code_num);
            }
            Err(err) => {
                error!("vcx_connection_send_ping(command_handle: {}, rc: {})",
                      command_handle, err);

                cb(command_handle, err.into());
            }
        };

        Ok(())
    }));

    error::SUCCESS.code_num
}

#[no_mangle]
pub extern fn vcx_connection_send_handshake_reuse(command_handle: u32,
                                                  connection_handle: u32,
                                                  oob_msg: *const c_char,
                                                  cb: Option<extern fn(xcommand_handle: u32, err: u32)>) -> u32 {
    info!("vcx_connection_send_handshake_reuse >>>");

    check_useful_c_str!(oob_msg, VcxErrorKind::InvalidOption);
    check_useful_c_callback!(cb, VcxErrorKind::InvalidOption);

    trace!("vcx_connection_send_handshake_reuse(command_handle: {}, connection_handle: {}, oob_msg: {})",
           command_handle, connection_handle, oob_msg);

    execute_async::<BoxFuture<'static, Result<(), ()>>>(Box::pin(async move {
        match send_handshake_reuse(connection_handle, &oob_msg).await {
            Ok(()) => {
                trace!("vcx_connection_send_handshake_reuse_cb(command_handle: {}, rc: {})",
                       command_handle, error::SUCCESS.message);
                cb(command_handle, error::SUCCESS.code_num);
            }
            Err(err) => {
                error!("vcx_connection_send_handshake_reuse_cb(command_handle: {}, rc: {})",
                      command_handle, err);

                cb(command_handle, err.into());
            }
        };

        Ok(())
    }));

    error::SUCCESS.code_num
}

/// Generate a signature for the specified data using connection pairwise keys
///
/// #params
///
/// command_handle: command handle to map callback to user context.
///
/// connection_handle: connection to use to sign the message.
///                    Was provided during creation. Used to identify connection object.
///
/// data_raw: raw data buffer for signature
///
/// data_len: length of data buffer
///
/// cb: Callback that provides the generated signature
///
/// # Example
/// data_raw -> [1, 2, 3, 4, 5, 6]
/// data_len -> 6
///
/// #Returns
/// Error code as a u32
#[no_mangle]
pub extern fn vcx_connection_sign_data(command_handle: CommandHandle,
                                       connection_handle: u32,
                                       data_raw: *const u8,
                                       data_len: u32,
                                       cb: Option<extern fn(command_handle_: CommandHandle,
                                                            err: u32,
                                                            signature_raw: *const u8,
                                                            signature_len: u32)>) -> u32 {
    trace!("vcx_connection_sign_data: >>> connection_handle: {}, data_raw: {:?}, data_len: {}",
           connection_handle, data_raw, data_len);

    check_useful_c_byte_array!(data_raw, data_len, VcxErrorKind::InvalidOption, VcxErrorKind::InvalidOption);
    check_useful_c_callback!(cb, VcxErrorKind::InvalidOption);

    trace!("vcx_connection_sign_data: entities >>> connection_handle: {}, data_raw: {:?}, data_len: {}",
           connection_handle, data_raw, data_len);

    execute_async::<BoxFuture<'static, Result<(), ()>>>(Box::pin(async move {
        let vk = match connection::get_pw_verkey(connection_handle) {
            Ok(err) => err,
            Err(err) => {
                error!("vcx_messages_sign_data_cb(command_handle: {}, rc: {}, signature: null)",
                      command_handle, err);
                cb(command_handle, err.into(), ptr::null_mut(), 0);
                return Ok(());
            }
        };

        match libindy::utils::crypto::sign(get_main_wallet_handle(), &vk, &data_raw).await {
            Ok(err) => {
                trace!("vcx_connection_sign_data_cb(command_handle: {}, connection_handle: {}, rc: {}, signature: {:?})",
                       command_handle, connection_handle, error::SUCCESS.message, err);

                let (signature_raw, signature_len) = utils::cstring::vec_to_pointer(&err);
                cb(command_handle, error::SUCCESS.code_num, signature_raw, signature_len);
            }
            Err(err) => {
                error!("vcx_messages_sign_data_cb(command_handle: {}, rc: {}, signature: null)",
                      command_handle, err);

                cb(command_handle, err.into(), ptr::null_mut(), 0);
            }
        };

        Ok(())
    }));

    error::SUCCESS.code_num
}

/// Verify the signature is valid for the specified data using connection pairwise keys
///
/// #params
///
/// command_handle: command handle to map callback to user context.
///
/// connection_handle: connection to use to verify signature.
///                    Was provided during creation. Used to identify connection object.
///
/// data_raw: raw data buffer for signature
///
/// data_len: length of data buffer
///
/// signature_raw: raw data buffer for signature
///
/// signature_len: length of data buffer
///
/// cb: Callback that specifies whether the signature was valid or not
///
/// # Example
/// data_raw -> [1, 2, 3, 4, 5, 6]
/// data_len -> 6
/// signature_raw -> [2, 3, 4, 5, 6, 7]
/// signature_len -> 6
///
/// #Returns
/// Error code as a u32
#[no_mangle]
pub extern fn vcx_connection_verify_signature(command_handle: CommandHandle,
                                              connection_handle: u32,
                                              data_raw: *const u8,
                                              data_len: u32,
                                              signature_raw: *const u8,
                                              signature_len: u32,
                                              cb: Option<extern fn(command_handle_: CommandHandle,
                                                                   err: u32,
                                                                   valid: bool)>) -> u32 {
    trace!("vcx_connection_verify_signature: >>> connection_handle: {}, data_raw: {:?}, data_len: {}, signature_raw: {:?}, signature_len: {}",
           connection_handle, data_raw, data_len, signature_raw, signature_len);

    check_useful_c_byte_array!(data_raw, data_len, VcxErrorKind::InvalidOption, VcxErrorKind::InvalidOption);
    check_useful_c_byte_array!(signature_raw, signature_len, VcxErrorKind::InvalidOption, VcxErrorKind::InvalidOption);
    check_useful_c_callback!(cb, VcxErrorKind::InvalidOption);

    trace!("vcx_connection_verify_signature: entities >>> connection_handle: {}, data_raw: {:?}, data_len: {}, signature_raw: {:?}, signature_len: {}",
           connection_handle, data_raw, data_len, signature_raw, signature_len);

    execute_async::<BoxFuture<'static, Result<(), ()>>>(Box::pin(async move {
        let vk = match connection::get_their_pw_verkey(connection_handle) {
            Ok(err) => err,
            Err(err) => {
                error!("vcx_connection_verify_signature_cb(command_handle: {}, rc: {}, valid: {})",
                      command_handle, err, false);
                cb(command_handle, err.into(), false);
                return Ok(());
            }
        };

        match libindy::utils::crypto::verify(&vk, &data_raw, &signature_raw).await {
            Ok(err) => {
                trace!("vcx_connection_verify_signature_cb(command_handle: {}, rc: {}, valid: {})",
                       command_handle, error::SUCCESS.message, err);

                cb(command_handle, error::SUCCESS.code_num, err);
            }
            Err(err) => {
                error!("vcx_connection_verify_signature_cb(command_handle: {}, rc: {}, valid: {})",
                      command_handle, err, false);

                cb(command_handle, err.into(), false);
            }
        };

        Ok(())
    }));

    error::SUCCESS.code_num
}

/// Releases the connection object by de-allocating memory
///
/// #Params
/// connection_handle: was provided during creation. Used to identify connection object
///
/// #Returns
/// Success
#[no_mangle]
pub extern fn vcx_connection_release(connection_handle: u32) -> u32 {
    info!("vcx_connection_release >>>");

    let source_id = get_source_id(connection_handle).unwrap_or_default();
    match release(connection_handle) {
        Ok(()) => {
            trace!("vcx_connection_release(connection_handle: {}, rc: {}), source_id: {:?}",
                   connection_handle, error::SUCCESS.message, source_id);
            error::SUCCESS.code_num
        }
        Err(err) => {
            error!("vcx_connection_release(connection_handle: {}), rc: {}), source_id: {:?}",
                  connection_handle, err, source_id);
            err.into()
        }
    }
}

/// Send discovery features message to the specified connection to discover which features it supports, and to what extent.
///
/// Note that this function is useful in case `aries` communication method is used.
/// In other cases it returns ActionNotSupported error.
///
/// #params
///
/// command_handle: command handle to map callback to user context.
///
/// connection_handle: connection to use to send message.
///                    Was provided during creation. Used to identify connection object.
///                    Note that connection must be in Accepted state.
///
/// query: (Optional) query string to match against supported message types.
///
/// comment: (Optional) human-friendly description of the query.
///
/// cb: Callback that provides success or failure of request
///
/// # Example
/// query -> `did:sov:BzCbsNYhMrjHiqZDTUASHg;spec/connections/`
///
/// comment -> `share please`
///
/// #Returns
/// Error code as a u32
#[no_mangle]
pub extern fn vcx_connection_send_discovery_features(command_handle: u32,
                                                     connection_handle: u32,
                                                     query: *const c_char,
                                                     comment: *const c_char,
                                                     cb: Option<extern fn(xcommand_handle: u32, err: u32)>) -> u32 {
    info!("vcx_connection_send_discovery_features >>>");

    check_useful_opt_c_str!(query, VcxErrorKind::InvalidOption);
    check_useful_opt_c_str!(comment, VcxErrorKind::InvalidOption);
    check_useful_c_callback!(cb, VcxErrorKind::InvalidOption);

    trace!("vcx_connection_send_discovery_features(command_handle: {}, connection_handle: {}, query: {:?}, comment: {:?})",
           command_handle, connection_handle, query, comment);

    execute_async::<BoxFuture<'static, Result<(), ()>>>(Box::pin(async move {
        match send_discovery_features(connection_handle, query.as_deref(), comment.as_deref()).await {
            Ok(()) => {
                trace!("vcx_connection_send_discovery_features(command_handle: {}, rc: {})",
                       command_handle, error::SUCCESS.message);
                cb(command_handle, error::SUCCESS.code_num);
            }
            Err(err) => {
                error!("vcx_connection_send_discovery_features(command_handle: {}, rc: {})",
                      command_handle, err);

                cb(command_handle, err.into());
            }
        };

        Ok(())
    }));

    error::SUCCESS.code_num
}

/// Get the information about the connection state.
///
/// Note: This method can be used for `aries` communication method only.
///     For other communication method it returns ActionNotSupported error.
///
/// #Params
/// command_handle: command handle to map callback to user context.
///
/// connection_handle: was provided during creation. Used to identify connection object
///
/// cb: Callback that provides the json string of connection information
///
/// # Example
/// info ->
///      {
///         "current": {
///             "did": <str>
///             "recipientKeys": array<str>
///             "routingKeys": array<str>
///             "serviceEndpoint": <str>,
///             "protocols": array<str> -  The set of protocol supported by current side.
///         },
///         "remote: { <Option> - details about remote connection side
///             "did": <str> - DID of remote side
///             "recipientKeys": array<str> - Recipient keys
///             "routingKeys": array<str> - Routing keys
///             "serviceEndpoint": <str> - Endpoint
///             "protocols": array<str> - The set of protocol supported by side. Is filled after DiscoveryFeatures process was completed.
///          }
///    }
///
/// #Returns
/// Error code as a u32
#[no_mangle]
pub extern fn vcx_connection_info(command_handle: CommandHandle,
                                  connection_handle: u32,
                                  cb: Option<extern fn(xcommand_handle: CommandHandle, err: u32, info: *const c_char)>) -> u32 {
    info!("vcx_connection_info >>>");

    check_useful_c_callback!(cb, VcxErrorKind::InvalidOption);

    let source_id = get_source_id(connection_handle).unwrap_or_default();
    trace!("vcx_connection_info(command_handle: {}, connection_handle: {}), source_id: {:?}",
           command_handle, connection_handle, source_id);

    execute(move || {
        match get_connection_info(connection_handle) {
            Ok(info) => {
                trace!("vcx_connection_info(command_handle: {}, connection_handle: {}, rc: {}, info: {}), source_id: {:?}",
                       command_handle, connection_handle, error::SUCCESS.message, info, source_id);
                let info = CStringUtils::string_to_cstring(info);
                cb(command_handle, error::SUCCESS.code_num, info.as_ptr());
            }
            Err(err) => {
                set_current_error_vcx(&err);
                error!("vcx_connection_info(command_handle: {}, connection_handle: {}, rc: {}, info: {}, source_id: {:?})",
                      command_handle, connection_handle, err, "null", source_id);
                cb(command_handle, err.into(), ptr::null_mut());
            }
        };

        Ok(())
    });

    error::SUCCESS.code_num
}

/// Retrieves pw_did from Connection object
///
/// #Params
/// command_handle: command handle to map callback to user context.
///
/// connection_handle: Connection handle that identifies pairwise connection
///
/// cb: Callback that provides your pw_did for this connection
///
/// #Returns
/// Error code as a u32
#[no_mangle]
pub extern fn vcx_connection_get_pw_did(command_handle: u32,
                                        connection_handle: u32,
                                        cb: Option<extern fn(xcommand_handle: u32, err: u32, serialized_data: *const c_char)>) -> u32 {
    info!("vcx_connection_get_pw_did >>>");

    check_useful_c_callback!(cb, VcxErrorKind::InvalidOption);

    let source_id = get_source_id(connection_handle).unwrap_or_default();
    trace!("vcx_connection_get_pw_did(command_handle: {}, connection_handle: {}), source_id: {:?}",
           command_handle, connection_handle, source_id);

    execute(move || {
        match get_pw_did(connection_handle) {
            Ok(json) => {
                trace!("vcx_connection_get_pw_did_cb(command_handle: {}, connection_handle: {}, rc: {}, pw_did: {}), source_id: {:?}",
                       command_handle, connection_handle, error::SUCCESS.message, json, source_id);
                let msg = CStringUtils::string_to_cstring(json);
                cb(command_handle, error::SUCCESS.code_num, msg.as_ptr());
            }
            Err(err) => {
                set_current_error_vcx(&err);
                error!("vcx_connection_get_pw_did_cb(command_handle: {}, connection_handle: {}, rc: {}, pw_did: {}), source_id: {:?}",
                      command_handle, connection_handle, err, "null", source_id);
                cb(command_handle, err.into(), ptr::null_mut());
            }
        };

        Ok(())
    });

    error::SUCCESS.code_num
}

/// Retrieves their_pw_did from Connection object
///
/// #Params
/// command_handle: command handle to map callback to user context.
///
/// connection_handle: Connection handle that identifies pairwise connection
///
/// cb: Callback that provides your pw_did for this connection
///
/// #Returns
/// Error code as a u32
#[no_mangle]
pub extern fn vcx_connection_get_their_pw_did(command_handle: u32,
                                              connection_handle: u32,
                                              cb: Option<extern fn(xcommand_handle: u32, err: u32, serialized_data: *const c_char)>) -> u32 {
    info!("vcx_connection_get_pw_did >>>");

    check_useful_c_callback!(cb, VcxErrorKind::InvalidOption);

    let source_id = get_source_id(connection_handle).unwrap_or_default();
    trace!("vcx_connection_get_their_pw_did(command_handle: {}, connection_handle: {}), source_id: {:?}",
           command_handle, connection_handle, source_id);

    execute(move || {
        match get_their_pw_did(connection_handle) {
            Ok(json) => {
                trace!("vcx_connection_get_their_pw_did_cb(command_handle: {}, connection_handle: {}, rc: {}, their_pw_did: {}), source_id: {:?}",
                       command_handle, connection_handle, error::SUCCESS.message, json, source_id);
                let msg = CStringUtils::string_to_cstring(json);
                cb(command_handle, error::SUCCESS.code_num, msg.as_ptr());
            }
            Err(err) => {
                set_current_error_vcx(&err);
                error!("vcx_connection_get_their_pw_did_cb(command_handle: {}, connection_handle: {}, rc: {}, their_pw_did: {}), source_id: {:?}",
                      command_handle, connection_handle, err, "null", source_id);
                cb(command_handle, err.into(), ptr::null_mut());
            }
        };

        Ok(())
    });

    error::SUCCESS.code_num
}

#[no_mangle]
pub extern fn vcx_connection_messages_download(command_handle: CommandHandle,
                                               connection_handle: u32,
                                               message_statuses: *const c_char,
                                               uids: *const c_char,
                                               cb: Option<extern fn(xcommand_handle: CommandHandle, err: u32, messages: *const c_char)>) -> u32 {
    info!("vcx_connection_messages_download >>>");

    check_useful_c_callback!(cb, VcxErrorKind::InvalidOption);

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

    let connection_handles: Vec<u32> = Vec::from([connection_handle]);

    trace!("vcx_connection_messages_download(command_handle: {}, message_statuses: {:?}, uids: {:?})",
           command_handle, message_statuses, uids);

    execute_async::<BoxFuture<'static, Result<(), ()>>>(Box::pin(async move {
        match connection::download_messages(connection_handles, message_statuses, uids).await {
            Ok(err) => {
                match serde_json::to_string(&err) {
                    Ok(err) => {
                        trace!("vcx_connection_messages_download_cb(command_handle: {}, rc: {}, messages: {})",
                               command_handle, error::SUCCESS.message, err);

                        let msg = CStringUtils::string_to_cstring(err);
                        cb(command_handle, error::SUCCESS.code_num, msg.as_ptr());
                    }
                    Err(err) => {
                        let err = VcxError::from_msg(VcxErrorKind::InvalidJson, format!("Cannot serialize messages: {}", err));
                        error!("vcx_connection_messages_download_cb(command_handle: {}, rc: {}, messages: {})",
                              command_handle, err, "null");

                        cb(command_handle, err.into(), ptr::null_mut());
                    }
                };
            }
            Err(err) => {
                error!("vcx_connection_messages_download_cb(command_handle: {}, rc: {}, messages: {})",
                      command_handle, err, "null");

                cb(command_handle, err.into(), ptr::null_mut());
            }
        };

        Ok(())
    }));

    error::SUCCESS.code_num
}

#[cfg(test)]
#[cfg(feature = "general_test")]
mod tests {
    use std::ffi::CString;
    use std::ptr;

    use serde_json::Value;

    use aries_vcx::agency_client::testing::mocking::AgencyMockDecrypted;
    use aries_vcx::utils::constants::{DELETE_CONNECTION_DECRYPTED_RESPONSE, GET_MESSAGES_DECRYPTED_RESPONSE};
    use aries_vcx::utils::devsetup::SetupMocks;
    use aries_vcx::utils::error;
    use aries_vcx::utils::error::SUCCESS;
    use aries_vcx::utils::mockdata::mockdata_connection::{ARIES_CONNECTION_ACK, ARIES_CONNECTION_REQUEST, DEFAULT_SERIALIZED_CONNECTION};

    use crate::api_lib::api_handle::connection::tests::{build_test_connection_inviter_invited, build_test_connection_inviter_null, build_test_connection_inviter_requested};
    use crate::api_lib::utils::return_types_u32;
    use crate::api_lib::utils::timeout::TimeoutUtils;
    use crate::api_lib::VcxStateType;

    use super::*;

    #[test]
    #[cfg(feature = "general_test")]
    fn test_vcx_connection_create() {
        let _setup = SetupMocks::init();

        let cb = return_types_u32::Return_U32_U32::new().unwrap();
        let _rc = vcx_connection_create(cb.command_handle,
                                        CString::new("test_create").unwrap().into_raw(),
                                        Some(cb.get_callback()));

        assert!(cb.receive(TimeoutUtils::some_medium()).unwrap() > 0);
    }

    #[test]
    #[cfg(feature = "general_test")]
    fn test_vcx_connection_create_fails() {
        let _setup = SetupMocks::init();

        let rc = vcx_connection_create(0,
                                       CString::new("test_create_fails").unwrap().into_raw(),
                                       None);
        assert_eq!(rc, error::INVALID_OPTION.code_num);
        let cb = return_types_u32::Return_U32_U32::new().unwrap();
        let rc = vcx_connection_create(cb.command_handle,
                                       ptr::null(),
                                       Some(cb.get_callback()));
        assert_eq!(rc, error::INVALID_OPTION.code_num);
    }

    #[tokio::test]
    #[cfg(feature = "general_test")]
    async fn test_vcx_connection_connect() {
        let _setup = SetupMocks::init();

        let cb = return_types_u32::Return_U32_STR::new().unwrap();
        let rc = vcx_connection_connect(cb.command_handle, 0, CString::new("{}").unwrap().into_raw(), Some(cb.get_callback()));
        assert!(cb.receive(TimeoutUtils::some_custom(1)).is_err());
        assert_eq!(rc, error::SUCCESS.code_num);

        let handle = build_test_connection_inviter_null().await;
        assert!(handle > 0);

        let cb = return_types_u32::Return_U32_STR::new().unwrap();
        let rc = vcx_connection_connect(cb.command_handle, handle, CString::new("{}").unwrap().into_raw(), Some(cb.get_callback()));
        assert_eq!(rc, error::SUCCESS.code_num);
        let invite_details = cb.receive(TimeoutUtils::some_medium()).unwrap();
        assert!(invite_details.is_some());
    }

    #[tokio::test]
    #[cfg(feature = "general_test")]
    async fn test_vcx_connection_connect_returns_invitation() {
        let _setup = SetupMocks::init();

        let handle = build_test_connection_inviter_null().await;
        let invitation = connect(handle).await.unwrap().unwrap();
        let invitation: Value = serde_json::from_str(&invitation).unwrap();
        assert!(invitation["serviceEndpoint"].is_string());
        assert!(invitation["recipientKeys"].is_array());
        assert!(invitation["routingKeys"].is_array());
        assert!(invitation["@type"].is_string());
        assert!(invitation["@id"].is_string());
    }

    #[tokio::test]
    #[cfg(feature = "general_test")]
    async fn test_vcx_connection_update_state() {
        let _setup = SetupMocks::init();

        let handle = build_test_connection_inviter_invited().await;
        assert!(handle > 0);

        AgencyMockDecrypted::set_next_decrypted_response(GET_MESSAGES_DECRYPTED_RESPONSE);
        AgencyMockDecrypted::set_next_decrypted_message(ARIES_CONNECTION_REQUEST);

        let cb = return_types_u32::Return_U32_U32::new().unwrap();
        let rc = vcx_connection_update_state(cb.command_handle, handle, Some(cb.get_callback()));
        assert_eq!(rc, error::SUCCESS.code_num);
        assert_eq!(cb.receive(TimeoutUtils::some_medium()).unwrap(), VcxStateType::VcxStateRequestReceived as u32);

        AgencyMockDecrypted::set_next_decrypted_response(GET_MESSAGES_DECRYPTED_RESPONSE);
        AgencyMockDecrypted::set_next_decrypted_message(ARIES_CONNECTION_ACK);

        let cb = return_types_u32::Return_U32_U32::new().unwrap();
        let rc = vcx_connection_update_state(cb.command_handle, handle, Some(cb.get_callback()));
        assert_eq!(rc, error::SUCCESS.code_num);
        assert_eq!(cb.receive(TimeoutUtils::some_medium()).unwrap(), VcxStateType::VcxStateAccepted as u32);
    }

    #[tokio::test]
    #[cfg(feature = "general_test")]
    async fn test_vcx_connection_update_state_with_message() {
        let _setup = SetupMocks::init();

        let handle = build_test_connection_inviter_requested().await;
        assert!(handle > 0);

        let cb = return_types_u32::Return_U32_U32::new().unwrap();
        let rc = vcx_connection_update_state_with_message(cb.command_handle, handle, CString::new(ARIES_CONNECTION_REQUEST).unwrap().into_raw(), Some(cb.get_callback()));
        assert_eq!(rc, error::SUCCESS.code_num);
        assert_eq!(cb.receive(TimeoutUtils::some_medium()).unwrap(), VcxStateType::VcxStateRequestReceived as u32);

        let cb = return_types_u32::Return_U32_U32::new().unwrap();
        let rc = vcx_connection_update_state_with_message(cb.command_handle, handle, CString::new(ARIES_CONNECTION_ACK).unwrap().into_raw(), Some(cb.get_callback()));
        assert_eq!(rc, error::SUCCESS.code_num);
        assert_eq!(cb.receive(TimeoutUtils::some_medium()).unwrap(), VcxStateType::VcxStateAccepted as u32);
    }

    #[test]
    #[cfg(feature = "general_test")]
    fn test_vcx_connection_update_state_fails() {
        let _setup = SetupMocks::init();

        let rc = vcx_connection_update_state(0, 0, None);
        assert_eq!(rc, error::INVALID_OPTION.code_num);
    }

    #[tokio::test]
    #[cfg(feature = "general_test")]
    async fn test_vcx_connection_serialize() {
        let _setup = SetupMocks::init();

        let handle = build_test_connection_inviter_requested().await;
        assert!(handle > 0);

        let cb = return_types_u32::Return_U32_STR::new().unwrap();
        let rc = vcx_connection_serialize(cb.command_handle, handle, Some(cb.get_callback()));
        assert_eq!(rc, error::SUCCESS.code_num);

        cb.receive(TimeoutUtils::some_medium()).unwrap().unwrap();
    }

    #[tokio::test]
    #[cfg(feature = "general_test")]
    async fn test_vcx_connection_release() {
        let _setup = SetupMocks::init();

        let handle = build_test_connection_inviter_requested().await;

        let rc = vcx_connection_release(handle);
        assert_eq!(rc, error::SUCCESS.code_num);

        let unknown_handle = handle + 1;
        assert_eq!(vcx_connection_release(unknown_handle), error::INVALID_CONNECTION_HANDLE.code_num);

        let cb = return_types_u32::Return_U32_STR::new().unwrap();
        let rc = vcx_connection_connect(0, handle, CString::new("{}").unwrap().into_raw(), Some(cb.get_callback()));
        assert!(cb.receive(TimeoutUtils::some_custom(1)).is_err());
        assert_eq!(rc, error::SUCCESS.code_num);
    }

    #[test]
    #[cfg(feature = "general_test")]
    fn test_vcx_connection_deserialize_succeeds() {
        let _setup = SetupMocks::init();

        let cb = return_types_u32::Return_U32_U32::new().unwrap();
        let err = vcx_connection_deserialize(cb.command_handle,
                                             CString::new(DEFAULT_SERIALIZED_CONNECTION).unwrap().into_raw(),
                                             Some(cb.get_callback()));
        assert_eq!(err, SUCCESS.code_num);
        let handle = cb.receive(TimeoutUtils::some_short()).unwrap();
        assert!(handle > 0);
    }

    #[tokio::test]
    #[cfg(feature = "general_test")]
    async fn test_vcx_connection_get_state() {
        let _setup = SetupMocks::init();

        let handle = build_test_connection_inviter_invited().await;

        AgencyMockDecrypted::set_next_decrypted_response(GET_MESSAGES_DECRYPTED_RESPONSE);
        AgencyMockDecrypted::set_next_decrypted_message(ARIES_CONNECTION_REQUEST);

        let cb = return_types_u32::Return_U32_U32::new().unwrap();
        let rc = vcx_connection_update_state(cb.command_handle, handle, Some(cb.get_callback()));
        assert_eq!(rc, error::SUCCESS.code_num);
        assert_eq!(cb.receive(TimeoutUtils::some_medium()).unwrap(), VcxStateType::VcxStateRequestReceived as u32);

        let cb = return_types_u32::Return_U32_U32::new().unwrap();
        let rc = vcx_connection_get_state(cb.command_handle, handle, Some(cb.get_callback()));
        assert_eq!(rc, error::SUCCESS.code_num);
        assert_eq!(cb.receive(TimeoutUtils::some_medium()).unwrap(), VcxStateType::VcxStateRequestReceived as u32)
    }

    #[tokio::test]
    #[cfg(feature = "general_test")]
    async fn test_vcx_connection_delete_connection() {
        let _setup = SetupMocks::init();

        let connection_handle = build_test_connection_inviter_requested().await;

        AgencyMockDecrypted::set_next_decrypted_response(DELETE_CONNECTION_DECRYPTED_RESPONSE);

        let cb = return_types_u32::Return_U32::new().unwrap();
        assert_eq!(vcx_connection_delete_connection(cb.command_handle, connection_handle, Some(cb.get_callback())), error::SUCCESS.code_num);
        cb.receive(TimeoutUtils::some_medium()).unwrap();

        assert_eq!(connection::get_source_id(connection_handle).unwrap_err().kind(), VcxErrorKind::InvalidHandle);
    }

    #[tokio::test]
    #[cfg(feature = "general_test")]
    async fn test_send_message() {
        let _setup = SetupMocks::init();

        let connection_handle = build_test_connection_inviter_requested().await;

        let msg = CString::new("MESSAGE").unwrap().into_raw();
        let send_msg_options = CString::new(json!({"msg_type":"type", "msg_title": "title", "ref_msg_id":null}).to_string()).unwrap().into_raw();
        let cb = return_types_u32::Return_U32_STR::new().unwrap();
        assert_eq!(vcx_connection_send_message(cb.command_handle, connection_handle, msg, send_msg_options, Some(cb.get_callback())), error::SUCCESS.code_num);
        cb.receive(TimeoutUtils::some_medium()).unwrap();
    }

    #[tokio::test]
    #[cfg(feature = "general_test")]
    async fn test_sign() {
        let _setup = SetupMocks::init();

        let connection_handle = connection::tests::build_test_connection_inviter_invited().await;

        let msg = format!("My message");
        let msg_len = msg.len();

        let cb = return_types_u32::Return_U32_BIN::new().unwrap();
        let cstr_msg = CString::new(msg).unwrap();
        assert_eq!(vcx_connection_sign_data(cb.command_handle,
                                            connection_handle,
                                            cstr_msg.as_ptr() as *const u8,
                                            msg_len as u32,
                                            Some(cb.get_callback())), error::SUCCESS.code_num);
        let _sig = cb.receive(TimeoutUtils::some_medium()).unwrap();
    }

    #[tokio::test]
    #[cfg(feature = "general_test")]
    async fn test_verify_signature() {
        let _setup = SetupMocks::init();

        let connection_handle = connection::tests::build_test_connection_inviter_requested().await;

        let msg = format!("My message");
        let msg_len = msg.len();

        let signature = format!("signature");
        let signature_length = signature.len();

        let cstr_msg = CString::new(msg).unwrap();
        let cstr_sig = CString::new(signature).unwrap();
        let cb = return_types_u32::Return_U32_BOOL::new().unwrap();
        assert_eq!(vcx_connection_verify_signature(cb.command_handle,
                                                   connection_handle,
                                                   cstr_msg.as_ptr() as *const u8,
                                                   msg_len as u32,
                                                   cstr_sig.as_ptr() as *const u8,
                                                   signature_length as u32,
                                                   Some(cb.get_callback())), error::SUCCESS.code_num);
        cb.receive(TimeoutUtils::some_medium()).unwrap();
    }
}
