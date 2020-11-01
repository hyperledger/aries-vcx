use std::ptr;

use indy_sys::CommandHandle;
use libc::c_char;

use connection;
use credential;
use error::prelude::*;
use utils::cstring::CStringUtils;
use utils::error;
use utils::threadpool::spawn;

/*
    The API represents a Holder side in credential issuance process.
    Assumes that pairwise connection between Issuer and Holder is already established.

    # State

    The set of object states, messages and transitions depends on the communication method is used.
    The communication method can be specified as a config option on one of *_init functions.
        VcxStateType::VcxStateRequestReceived - once `vcx_credential_create_with_offer` (create Credential object) is called.

        VcxStateType::VcxStateOfferSent - once `vcx_credential_send_request` (send `CredentialRequest` message) is called.

        VcxStateType::VcxStateAccepted - once `Credential` messages is received.
        VcxStateType::None - once `ProblemReport` messages is received.
                                                use `vcx_credential_update_state` or `vcx_credential_update_state_with_message` functions for state updates.

    # Transitions
    RFC - https://github.com/hyperledger/aries-rfcs/tree/7b6b93acbaf9611d3c892c4bada142fe2613de6e/features/0036-issue-credential
        VcxStateType::None - `vcx_credential_create_with_offer` - VcxStateType::VcxStateRequestReceived

        VcxStateType::VcxStateRequestReceived - `vcx_issuer_send_credential_offer` - VcxStateType::VcxStateOfferSent

        VcxStateType::VcxStateOfferSent - received `Credential` - VcxStateType::VcxStateAccepted
        VcxStateType::VcxStateOfferSent - received `ProblemReport` - VcxStateType::None

    # Messages
        CredentialProposal - https://github.com/hyperledger/aries-rfcs/tree/7b6b93acbaf9611d3c892c4bada142fe2613de6e/features/0036-issue-credential#propose-credential
        CredentialOffer - https://github.com/hyperledger/aries-rfcs/tree/7b6b93acbaf9611d3c892c4bada142fe2613de6e/features/0036-issue-credential#offer-credential
        CredentialRequest - https://github.com/hyperledger/aries-rfcs/tree/7b6b93acbaf9611d3c892c4bada142fe2613de6e/features/0036-issue-credential#request-credential
        Credential - https://github.com/hyperledger/aries-rfcs/tree/7b6b93acbaf9611d3c892c4bada142fe2613de6e/features/0036-issue-credential#issue-credential
        ProblemReport - https://github.com/hyperledger/aries-rfcs/tree/7b6b93acbaf9611d3c892c4bada142fe2613de6e/features/0035-report-problem#the-problem-report-message-type
        Ack - https://github.com/hyperledger/aries-rfcs/tree/master/features/0015-acks#explicit-acks
*/

/// Retrieve Payment Transaction Information for this Credential. Typically this will include
/// how much payment is requried by the issuer, which needs to be provided by the prover, before the issuer will
/// issue the credential to the prover. Ideally a prover would want to know how much payment is being asked before
/// submitting the credential request (which triggers the payment to be made).
///
/// #Params
/// command_handle: command handle to map callback to user context.
///
/// credential_handle: credential handle that was provided during creation. Used to identify credential object
///
/// cb: Callback that provides Payment Info of a Credential
///
/// # Example:
/// payment_info ->
///     {
///         "payment_required":"one-time",
///         "payment_addr":"pov:null:OsdjtGKavZDBuG2xFw2QunVwwGs5IB3j",
///         "price":1
///     }
///
/// #Returns
/// Error code as a u32
#[no_mangle]
#[allow(unused_variables, unused_mut)]
pub extern fn vcx_credential_get_payment_info(command_handle: CommandHandle,
                                              credential_handle: u32,
                                              cb: Option<extern fn(xcommand_handle: CommandHandle, err: u32, *const c_char)>) -> u32 {
    info!("vcx_credential_get_payment_info >>>");

    check_useful_c_callback!(cb, VcxErrorKind::InvalidOption);
    spawn(move || {
        error!("Payments not supported anymore");
        cb(command_handle, 1, ptr::null());
        Ok(())
    });

    error::SUCCESS.code_num
}

/// Create a Credential object that requests and receives a credential for an institution
///
/// #Params
/// command_handle: command handle to map callback to user context.
///
/// source_id: Institution's personal identification for the credential, should be unique.
///
/// offer: credential offer received via "vcx_credential_get_offers"
///
/// # Example
/// offer -> depends on communication method:
///     aries:
///         {"@type":"did:sov:BzCbsNYhMrjHiqZDTUASHg;spec/issue-credential/1.0/offer-credential", "@id":"<uuid-of-offer-message>", "comment":"somecomment", "credential_preview":<json-ldobject>, "offers~attach":[{"@id":"libindy-cred-offer-0", "mime-type":"application/json", "data":{"base64":"<bytesforbase64>"}}]}
///
/// cb: Callback that provides credential handle or error status
///
/// #Returns
/// Error code as a u32
#[no_mangle]
#[allow(unused_variables, unused_mut)]
pub extern fn vcx_credential_create_with_offer(command_handle: CommandHandle,
                                               source_id: *const c_char,
                                               offer: *const c_char,
                                               cb: Option<extern fn(xcommand_handle: CommandHandle, err: u32, credential_handle: u32)>) -> u32 {
    info!("vcx_credential_create_with_offer >>>");

    check_useful_c_callback!(cb, VcxErrorKind::InvalidOption);
    check_useful_c_str!(source_id, VcxErrorKind::InvalidOption);
    check_useful_c_str!(offer, VcxErrorKind::InvalidOption);

    trace!("vcx_credential_create_with_offer(command_handle: {}, source_id: {}, offer: {})",
           command_handle, source_id, secret!(&offer));

    spawn(move || {
        match credential::credential_create_with_offer(&source_id, &offer) {
            Ok(x) => {
                trace!("vcx_credential_create_with_offer_cb(command_handle: {}, source_id: {}, rc: {}, handle: {})",
                       command_handle, source_id, error::SUCCESS.message, x);
                cb(command_handle, error::SUCCESS.code_num, x)
            }
            Err(x) => {
                warn!("vcx_credential_create_with_offer_cb(command_handle: {}, source_id: {}, rc: {}, handle: {})",
                      command_handle, source_id, x, 0);
                cb(command_handle, x.into(), 0);
            }
        };

        Ok(())
    });

    error::SUCCESS.code_num
}


/// Retrieve information about a stored credential in user's wallet, including credential id and the credential itself.
///
/// #Params
/// command_handle: command handle to map callback to user context.
///
/// credential_handle: credential handle that was provided during creation. Used to identify credential object
///
/// cb: Callback that provides error status of api call, or returns the credential in json format of "{uuid:credential}".
///
/// # Example
/// credential -> depends on communication method:
///     aries:
///         https://github.com/hyperledger/aries-rfcs/tree/master/features/0036-issue-credential#issue-credential
///
/// #Returns
/// Error code as a u32
#[no_mangle]
#[allow(unused_variables, unused_mut)]
pub extern fn vcx_get_credential(command_handle: CommandHandle,
                                 credential_handle: u32,
                                 cb: Option<extern fn(xcommand_handle: CommandHandle, err: u32, credential: *const c_char)>) -> u32 {
    info!("vcx_get_credential >>>");

    check_useful_c_callback!(cb, VcxErrorKind::InvalidOption);
    if !credential::is_valid_handle(credential_handle) {
        return VcxError::from(VcxErrorKind::InvalidCredentialHandle).into();
    }

    let source_id = credential::get_source_id(credential_handle).unwrap_or_default();
    trace!("vcx_get_credential(command_handle: {}, credential_handle: {}) source_id: {})",
           command_handle, credential_handle, source_id);

    spawn(move || {
        match credential::get_credential(credential_handle) {
            Ok(s) => {
                trace!("vcx_get_credential_cb(commmand_handle: {}, rc: {}, msg: {}) source_id: {}",
                       command_handle, error::SUCCESS.code_num, s, source_id);
                let msg = CStringUtils::string_to_cstring(s);
                cb(command_handle, error::SUCCESS.code_num, msg.as_ptr());
            }
            Err(e) => {
                error!("vcx_get_credential_cb(commmand_handle: {}, rc: {}, msg: {}) source_id: {}",
                       command_handle, e, "".to_string(), source_id);
                cb(command_handle, e.into(), ptr::null_mut());
            }
        };

        Ok(())
    });

    error::SUCCESS.code_num
}

/// Delete a Credential from the wallet and release its handle.
///
/// # Params
/// command_handle: command handle to map callback to user context.
///
/// credential_handle: handle of the credential to delete.
///
/// cb: Callback that provides error status of delete credential request
///
/// # Returns
/// Error code as a u32
#[no_mangle]
#[allow(unused_assignments)]
pub extern fn vcx_delete_credential(command_handle: CommandHandle,
                                    credential_handle: u32,
                                    cb: Option<extern fn(
                                        xcommand_handle: CommandHandle,
                                        err: u32)>) -> u32 {
    info!("vcx_delete_credential >>>");

    check_useful_c_callback!(cb, VcxErrorKind::InvalidOption);
    if !credential::is_valid_handle(credential_handle) {
        return VcxError::from(VcxErrorKind::InvalidCredentialHandle).into();
    }

    let source_id = credential::get_source_id(credential_handle).unwrap_or_default();
    trace!("vcx_delete_credential(command_handle: {}, credential_handle: {}), source_id: {})", command_handle, credential_handle, source_id);

    spawn(move || {
        match credential::delete_credential(credential_handle) {
            Ok(_) => {
                trace!("vcx_delete_credential_cb(command_handle: {}, rc: {}), credential_handle: {}, source_id: {})", command_handle, error::SUCCESS.message, credential_handle, source_id);
                cb(command_handle, error::SUCCESS.code_num);
            }
            Err(e) => {
                trace!("vcx_delete_credential_cb(command_handle: {}, rc: {}), credential_handle: {}, source_id: {})", command_handle, e, credential_handle, source_id);
                cb(command_handle, e.into());
            }
        }

        Ok(())
    });

    error::SUCCESS.code_num
}

#[no_mangle]
pub extern fn vcx_credential_get_attributes(command_handle: CommandHandle,
                                            credential_handle: u32,
                                            cb: Option<extern fn(xcommand_handle: CommandHandle, err: u32, attributes: *const c_char)>) -> u32 {
    info!("vcx_credential_get_attributes >>> credential_handle: {:?}", credential_handle);

    check_useful_c_callback!(cb, VcxErrorKind::InvalidOption);
    if !credential::is_valid_handle(credential_handle) {
        return VcxError::from(VcxErrorKind::InvalidCredentialHandle).into();
    }

    let source_id = credential::get_source_id(credential_handle).unwrap_or_default();
    trace!("vcx_credential_get_attributes(command_handle: {}, credential_handle: {}) source_id: {})",
           command_handle, credential_handle, source_id);

    spawn(move || {
        match credential::get_attributes(credential_handle) {
            Ok(s) => {
                trace!("vcx_credential_get_attribute_cb(commmand_handle: {}, rc: {}, attributes: {}) source_id: {}",
                       command_handle, error::SUCCESS.code_num, s, source_id);
                let attrs = CStringUtils::string_to_cstring(s);
                cb(command_handle, error::SUCCESS.code_num, attrs.as_ptr());
            }
            Err(e) => {
                error!("vcx_credential_get_attributes_cb(commmand_handle: {}, rc: {}, attributes: {}) source_id: {}",
                       command_handle, e, "".to_string(), source_id);
                cb(command_handle, e.into(), ptr::null_mut());
            }
        };

        Ok(())
    });

    error::SUCCESS.code_num
}

#[no_mangle]
pub extern fn vcx_credential_get_attachment(command_handle: CommandHandle,
                                            credential_handle: u32,
                                            cb: Option<extern fn(xcommand_handle: CommandHandle, err: u32, attachment: *const c_char)>) -> u32 {
    info!("vcx_credential_get_attachment >>> credential_handle: {:?}", credential_handle);

    check_useful_c_callback!(cb, VcxErrorKind::InvalidOption);
    if !credential::is_valid_handle(credential_handle) {
        return VcxError::from(VcxErrorKind::InvalidCredentialHandle).into();
    }

    let source_id = credential::get_source_id(credential_handle).unwrap_or_default();
    trace!("vcx_credential_get_attachment(command_handle: {}, credential_handle: {}) source_id: {})",
           command_handle, credential_handle, source_id);

    spawn(move || {
        match credential::get_attachment(credential_handle) {
            Ok(s) => {
                trace!("vcx_credential_get_attachment_cb(commmand_handle: {}, rc: {}, attachment: {}) source_id: {}",
                       command_handle, error::SUCCESS.code_num, s, source_id);
                let attach = CStringUtils::string_to_cstring(s);
                cb(command_handle, error::SUCCESS.code_num, attach.as_ptr());
            }
            Err(e) => {
                error!("vcx_credential_get_attachment_cb(commmand_handle: {}, rc: {}, attachment: {}) source_id: {}",
                       command_handle, e, "".to_string(), source_id);
                cb(command_handle, e.into(), ptr::null_mut());
            }
        };

        Ok(())
    });

    error::SUCCESS.code_num
}

#[no_mangle]
pub extern fn vcx_credential_get_tails_location(command_handle: CommandHandle,
                                 credential_handle: u32,
                                 cb: Option<extern fn(xcommand_handle: CommandHandle, err: u32, location: *const c_char)>) -> u32 {
    info!("vcx_credential_get_tails_location >>> credential_handle: {:?}", credential_handle);

    check_useful_c_callback!(cb, VcxErrorKind::InvalidOption);
    if !credential::is_valid_handle(credential_handle) {
        return VcxError::from(VcxErrorKind::InvalidCredentialHandle).into();
    }

    let source_id = credential::get_source_id(credential_handle).unwrap_or_default();
    trace!("vcx_credential_get_tails_location(command_handle: {}, credential_handle: {}) source_id: {})",
           command_handle, credential_handle, source_id);

    spawn(move || {
        match credential::get_tails_location(credential_handle) {
            Ok(s) => {
                trace!("vcx_credential_get_tails_location_cb(commmand_handle: {}, rc: {}, location: {}) source_id: {}",
                       command_handle, error::SUCCESS.code_num, s, source_id);
                let location = CStringUtils::string_to_cstring(s);
                cb(command_handle, error::SUCCESS.code_num, location.as_ptr());
            }
            Err(e) => {
                error!("vcx_credential_get_tails_location_cb(commmand_handle: {}, rc: {}, location: {}) source_id: {}",
                       command_handle, e, "".to_string(), source_id);
                cb(command_handle, e.into(), ptr::null_mut());
            }
        };

        Ok(())
    });

    error::SUCCESS.code_num
}

#[no_mangle]
pub extern fn vcx_credential_get_tails_hash(command_handle: CommandHandle,
                                 credential_handle: u32,
                                 cb: Option<extern fn(xcommand_handle: CommandHandle, err: u32, hash: *const c_char)>) -> u32 {
    info!("vcx_credential_get_tails_hash >>> credential_handle: {:?}", credential_handle);

    check_useful_c_callback!(cb, VcxErrorKind::InvalidOption);
    if !credential::is_valid_handle(credential_handle) {
        return VcxError::from(VcxErrorKind::InvalidCredentialHandle).into();
    }

    let source_id = credential::get_source_id(credential_handle).unwrap_or_default();
    trace!("vcx_credential_get_tails_hash(command_handle: {}, credential_handle: {}) source_id: {})",
           command_handle, credential_handle, source_id);

    spawn(move || {
        match credential::get_tails_hash(credential_handle) {
            Ok(s) => {
                trace!("vcx_credential_get_tails_hash_cb(commmand_handle: {}, rc: {}, hash: {}) source_id: {}",
                       command_handle, error::SUCCESS.code_num, s, source_id);
                let hash = CStringUtils::string_to_cstring(s);
                cb(command_handle, error::SUCCESS.code_num, hash.as_ptr());
            }
            Err(e) => {
                error!("vcx_credential_get_tails_hash_cb(commmand_handle: {}, rc: {}, hash: {}) source_id: {}",
                       command_handle, e, "".to_string(), source_id);
                cb(command_handle, e.into(), ptr::null_mut());
            }
        };

        Ok(())
    });

    error::SUCCESS.code_num
}

#[no_mangle]
pub extern fn vcx_credential_get_rev_reg_id(command_handle: CommandHandle,
                                 credential_handle: u32,
                                 cb: Option<extern fn(xcommand_handle: CommandHandle, err: u32, hash: *const c_char)>) -> u32 {
    info!("vcx_credential_get_rev_reg_id >>> credential_handle: {:?}", credential_handle);

    check_useful_c_callback!(cb, VcxErrorKind::InvalidOption);
    if !credential::is_valid_handle(credential_handle) {
        return VcxError::from(VcxErrorKind::InvalidCredentialHandle).into();
    }

    let source_id = credential::get_source_id(credential_handle).unwrap_or_default();
    trace!("vcx_credential_get_rev_reg_id(command_handle: {}, credential_handle: {}) source_id: {})",
           command_handle, credential_handle, source_id);

    spawn(move || {
        match credential::get_rev_reg_id(credential_handle) {
            Ok(s) => {
                trace!("vcx_credential_get_rev_reg_id_cb(commmand_handle: {}, rc: {}, hash: {}) source_id: {}",
                       command_handle, error::SUCCESS.code_num, s, source_id);
                let rev_reg_id = CStringUtils::string_to_cstring(s);
                cb(command_handle, error::SUCCESS.code_num, rev_reg_id.as_ptr());
            }
            Err(e) => {
                error!("vcx_credential_get_rev_reg_id_cb(commmand_handle: {}, rc: {}, hash: {}) source_id: {}",
                       command_handle, e, "".to_string(), source_id);
                cb(command_handle, e.into(), ptr::null_mut());
            }
        };

        Ok(())
    });

    error::SUCCESS.code_num
}

/// Create a Credential object based off of a known message id for a given connection.
///
/// #Params
/// command_handle: command handle to map callback to user context.
///
/// source_id: Institution's personal identification for the credential, should be unique.
///
/// connection_handle: connection to query for credential offer
///
/// msg_id: msg_id that contains the credential offer
///
/// cb: Callback that provides credential handle or error status
///
/// #Returns
/// Error code as a u32
#[no_mangle]
#[allow(unused_variables, unused_mut)]
pub extern fn vcx_credential_create_with_msgid(command_handle: CommandHandle,
                                               source_id: *const c_char,
                                               connection_handle: u32,
                                               msg_id: *const c_char,
                                               cb: Option<extern fn(xcommand_handle: CommandHandle, err: u32, credential_handle: u32, offer: *const c_char)>) -> u32 {
    info!("vcx_credential_create_with_msgid >>>");

    check_useful_c_callback!(cb, VcxErrorKind::InvalidOption);
    check_useful_c_str!(source_id, VcxErrorKind::InvalidOption);
    check_useful_c_str!(msg_id, VcxErrorKind::InvalidOption);

    trace!("vcx_credential_create_with_msgid(command_handle: {}, source_id: {}, connection_handle: {}, msg_id: {})",
           command_handle, source_id, connection_handle, msg_id);

    spawn(move || {
        match credential::credential_create_with_msgid(&source_id, connection_handle, &msg_id) {
            Ok((handle, offer_string)) => {
                let c_offer = CStringUtils::string_to_cstring(offer_string);
                trace!("vcx_credential_create_with_offer_cb(command_handle: {}, source_id: {}, rc: {}, handle: {}) source_id: {}",
                       command_handle, source_id, error::SUCCESS.message, handle, source_id);
                cb(command_handle, error::SUCCESS.code_num, handle, c_offer.as_ptr())
            }
            Err(e) => {
                warn!("vcx_credential_create_with_offer_cb(command_handle: {}, source_id: {}, rc: {}, handle: {}) source_id: {}",
                      command_handle, source_id, e, 0, source_id);
                cb(command_handle, e.into(), 0, ptr::null_mut());
            }
        };

        Ok(())
    });

    error::SUCCESS.code_num
}

/// Approves the credential offer and submits a credential request. The result will be a credential stored in the prover's wallet.
///
/// #params
/// command_handle: command handle to map callback to user context
///
/// credential_handle: credential handle that was provided during creation. Used to identify credential object
///
/// connection_handle: Connection handle that identifies pairwise connection
///
/// cb: Callback that provides error status of credential request
///
/// #Returns
/// Error code as a u32
#[no_mangle]
pub extern fn vcx_credential_send_request(command_handle: CommandHandle,
                                          credential_handle: u32,
                                          connection_handle: u32,
                                          _payment_handle: u32,
                                          cb: Option<extern fn(xcommand_handle: CommandHandle, err: u32)>) -> u32 {
    info!("vcx_credential_send_request >>>");

    check_useful_c_callback!(cb, VcxErrorKind::InvalidOption);

    if !credential::is_valid_handle(credential_handle) {
        return VcxError::from(VcxErrorKind::InvalidCredentialHandle).into();
    }

    if !connection::is_valid_handle(connection_handle) {
        return VcxError::from(VcxErrorKind::InvalidConnectionHandle).into();
    }

    let source_id = credential::get_source_id(credential_handle).unwrap_or_default();
    trace!("vcx_credential_send_request(command_handle: {}, credential_handle: {}, connection_handle: {}), source_id: {:?}",
           command_handle, credential_handle, connection_handle, source_id);

    spawn(move || {
        match credential::send_credential_request(credential_handle, connection_handle) {
            Ok(x) => {
                trace!("vcx_credential_send_request_cb(command_handle: {}, rc: {}) source_id: {}",
                       command_handle, x.to_string(), source_id);
                cb(command_handle, x);
            }
            Err(e) => {
                warn!("vcx_credential_send_request_cb(command_handle: {}, rc: {}) source_id: {}",
                      command_handle, e, source_id);
                cb(command_handle, e.into());
            }
        };

        Ok(())
    });

    error::SUCCESS.code_num
}

/// Approves the credential offer and gets the credential request message that can be sent to the specified connection
///
/// #params
/// command_handle: command handle to map callback to user context
///
/// credential_handle: credential handle that was provided during creation. Used to identify credential object
///
/// my_pw_did: Use Connection api (vcx_connection_get_pw_did) with specified connection_handle to retrieve your pw_did
///
/// their_pw_did: Use Connection api (vcx_connection_get_their_pw_did) with specified connection_handle to retrieve theri pw_did
///
/// cb: Callback that provides error status of credential request
///
/// #Returns
/// Error code as a u32
#[no_mangle]
pub extern fn vcx_credential_get_request_msg(command_handle: CommandHandle,
                                             credential_handle: u32,
                                             my_pw_did: *const c_char,
                                             their_pw_did: *const c_char,
                                             _payment_handle: u32,
                                             cb: Option<extern fn(xcommand_handle: CommandHandle, err: u32, msg: *const c_char)>) -> u32 {
    info!("vcx_credential_get_request_msg >>>");

    check_useful_c_str!(my_pw_did, VcxErrorKind::InvalidOption);
    check_useful_opt_c_str!(their_pw_did, VcxErrorKind::InvalidOption);
    check_useful_c_callback!(cb, VcxErrorKind::InvalidOption);

    if !credential::is_valid_handle(credential_handle) {
        return VcxError::from(VcxErrorKind::InvalidCredentialHandle).into();
    }

    let source_id = credential::get_source_id(credential_handle).unwrap_or_default();
    trace!("vcx_credential_get_request_msg(command_handle: {}, credential_handle: {}, my_pw_did: {}, their_pw_did: {:?}), source_id: {:?}",
           command_handle, credential_handle, my_pw_did, their_pw_did, source_id);

    spawn(move || {
        match credential::generate_credential_request_msg(credential_handle, &my_pw_did, &their_pw_did.unwrap_or_default()) {
            Ok(msg) => {
                let msg = CStringUtils::string_to_cstring(msg);
                trace!("vcx_credential_get_request_msg_cb(command_handle: {}, rc: {}) source_id: {}",
                       command_handle, error::SUCCESS.message, source_id);
                cb(command_handle, error::SUCCESS.code_num, msg.as_ptr());
            }
            Err(e) => {
                warn!("vcx_credential_get_request_msg_cb(command_handle: {}, rc: {}) source_id: {}",
                      command_handle, e, source_id);
                cb(command_handle, e.into(), ptr::null_mut());
            }
        };

        Ok(())
    });

    error::SUCCESS.code_num
}

/// Queries agency for credential offers from the given connection.
///
/// #Params
/// command_handle: command handle to map callback to user context.
///
/// connection_handle: Connection to query for credential offers.
///
/// cb: Callback that provides any credential offers and error status of query
///
/// # Example offers -> "[[{"msg_type": "CREDENTIAL_OFFER","version": "0.1","to_did": "...","from_did":"...","credential": {"account_num": ["...."],"name_on_account": ["Alice"]},"schema_seq_no": 48,"issuer_did": "...","credential_name": "Account Certificate","credential_id": "3675417066","msg_ref_id": "ymy5nth"}]]"
///
/// #Returns
/// Error code as a u32
#[no_mangle]
pub extern fn vcx_credential_get_offers(command_handle: CommandHandle,
                                        connection_handle: u32,
                                        cb: Option<extern fn(xcommand_handle: CommandHandle, err: u32, credential_offers: *const c_char)>) -> u32 {
    info!("vcx_credential_get_offers >>>");

    check_useful_c_callback!(cb, VcxErrorKind::InvalidOption);

    if !connection::is_valid_handle(connection_handle) {
        return VcxError::from(VcxErrorKind::InvalidConnectionHandle).into();
    }

    trace!("vcx_credential_get_offers(command_handle: {}, connection_handle: {})",
           command_handle, connection_handle);

    spawn(move || {
        match credential::get_credential_offer_messages(connection_handle) {
            Ok(x) => {
                trace!("vcx_credential_get_offers_cb(command_handle: {}, rc: {}, msg: {})",
                       command_handle, x.to_string(), x);
                let msg = CStringUtils::string_to_cstring(x);
                cb(command_handle, error::SUCCESS.code_num, msg.as_ptr());
            }
            Err(x) => {
                error!("vcx_credential_get_offers_cb(command_handle: {}, rc: {}, msg: null)",
                       command_handle, x);
                cb(command_handle, x.into(), ptr::null_mut());
            }
        };

        Ok(())
    });

    error::SUCCESS.code_num
}

/// Query the agency for the received messages.
/// Checks for any messages changing state in the credential object and updates the state attribute.
/// If it detects a credential it will store the credential in the wallet.
///
/// #Params
/// command_handle: command handle to map callback to user context.
///
/// credential_handle: Credential handle that was provided during creation. Used to identify credential object
///
/// cb: Callback that provides most current state of the credential and error status of request
///
/// #Returns
/// Error code as a u32
#[no_mangle]
#[deprecated(since = "0.12.0", note = "Use vcx_v2_credential_update_state instead.")]
pub extern fn vcx_credential_update_state(command_handle: CommandHandle,
                                          credential_handle: u32,
                                          cb: Option<extern fn(xcommand_handle: CommandHandle, err: u32, state: u32)>) -> u32 {
    info!("vcx_credential_update_state >>>");

    check_useful_c_callback!(cb, VcxErrorKind::InvalidOption);

    if !credential::is_valid_handle(credential_handle) {
        return VcxError::from(VcxErrorKind::InvalidCredentialHandle).into();
    }

    let source_id = credential::get_source_id(credential_handle).unwrap_or_default();
    trace!("vcx_credential_update_state(command_handle: {}, credential_handle: {}), source_id: {:?}",
           command_handle, credential_handle, source_id);

    spawn(move || {
        match credential::update_state(credential_handle, None, None) {
            Ok(_) => (),
            Err(e) => {
                error!("vcx_credential_update_state_cb(command_handle: {}, rc: {}, state: {}), source_id: {:?}",
                       command_handle, e, 0, source_id);
                cb(command_handle, e.into(), 0)
            }
        }

        match credential::get_state(credential_handle) {
            Ok(s) => {
                trace!("vcx_credential_update_state_cb(command_handle: {}, rc: {}, state: {}), source_id: {:?}",
                       command_handle, error::SUCCESS.message, s, source_id);
                cb(command_handle, error::SUCCESS.code_num, s)
            }
            Err(e) => {
                error!("vcx_credential_update_state_cb(command_handle: {}, rc: {}, state: {}), source_id: {:?}",
                       command_handle, e, 0, source_id);
                cb(command_handle, e.into(), 0)
            }
        };

        Ok(())
    });

    error::SUCCESS.code_num
}

#[no_mangle]
pub extern fn vcx_v2_credential_update_state(command_handle: CommandHandle,
                                             credential_handle: u32,
                                             connection_handle: u32,
                                             cb: Option<extern fn(xcommand_handle: CommandHandle, err: u32, state: u32)>) -> u32 {
    info!("vcx_v2_credential_update_state >>>");

    check_useful_c_callback!(cb, VcxErrorKind::InvalidOption);

    if !credential::is_valid_handle(credential_handle) {
        return VcxError::from(VcxErrorKind::InvalidCredentialHandle).into();
    }

    if !connection::is_valid_handle(connection_handle) {
        return VcxError::from(VcxErrorKind::InvalidConnectionHandle).into();
    }

    let source_id = credential::get_source_id(credential_handle).unwrap_or_default();
    trace!("vcx_v2_credential_update_state(command_handle: {}, credential_handle: {}, connection_handle: {}), source_id: {:?}",
           command_handle, credential_handle, connection_handle, source_id);

    spawn(move || {
        match credential::update_state(credential_handle, None, Some(connection_handle)) {
            Ok(_) => (),
            Err(e) => {
                error!("vcx_v2_credential_update_state_cb(command_handle: {}, rc: {}, state: {}), source_id: {:?}",
                       command_handle, e, 0, source_id);
                cb(command_handle, e.into(), 0)
            }
        }

        match credential::get_state(credential_handle) {
            Ok(s) => {
                trace!("vcx_v2_credential_update_state_cb(command_handle: {}, rc: {}, state: {}), source_id: {:?}",
                       command_handle, error::SUCCESS.message, s, source_id);
                cb(command_handle, error::SUCCESS.code_num, s)
            }
            Err(e) => {
                error!("vcx_v2_credential_update_state_cb(command_handle: {}, rc: {}, state: {}), source_id: {:?}",
                       command_handle, e, 0, source_id);
                cb(command_handle, e.into(), 0)
            }
        };

        Ok(())
    });

    error::SUCCESS.code_num
}

/// Update the state of the credential based on the given message.
///
/// #Params
/// command_handle: command handle to map callback to user context.
///
/// credential_handle: Credential handle that was provided during creation. Used to identify credential object
///
/// message: message to process for state changes
///
/// cb: Callback that provides most current state of the credential and error status of request
///
/// #Returns
/// Error code as a u32
#[no_mangle]
pub extern fn vcx_credential_update_state_with_message(command_handle: CommandHandle,
                                                       credential_handle: u32,
                                                       message: *const c_char,
                                                       cb: Option<extern fn(xcommand_handle: CommandHandle, err: u32, state: u32)>) -> u32 {
    info!("vcx_credential_update_state_with_message >>>");

    check_useful_c_callback!(cb, VcxErrorKind::InvalidOption);
    check_useful_c_str!(message, VcxErrorKind::InvalidOption);

    if !credential::is_valid_handle(credential_handle) {
        return VcxError::from(VcxErrorKind::InvalidCredentialHandle).into();
    }

    let source_id = credential::get_source_id(credential_handle).unwrap_or_default();
    trace!("vcx_credential_update_state_with_message(command_handle: {}, credential_handle: {}), source_id: {:?}",
           command_handle, credential_handle, source_id);

    spawn(move || {
        match credential::update_state(credential_handle, Some(&message), None) {
            Ok(_) => (),
            Err(e) => {
                error!("vcx_credential_update_state_with_message_cb(command_handle: {}, rc: {}, state: {}), source_id: {:?}",
                       command_handle, e, 0, source_id);
                cb(command_handle, e.into(), 0)
            }
        }

        match credential::get_state(credential_handle) {
            Ok(s) => {
                trace!("vcx_credential_update_state_with_message_cb(command_handle: {}, rc: {}, state: {}), source_id: {:?}",
                       command_handle, error::SUCCESS.message, s, source_id);
                cb(command_handle, error::SUCCESS.code_num, s)
            }
            Err(e) => {
                error!("vcx_credential_update_state_cb(command_handle: {}, rc: {}, state: {}), source_id: {:?}",
                       command_handle, e, 0, source_id);
                cb(command_handle, e.into(), 0)
            }
        };

        Ok(())
    });

    error::SUCCESS.code_num
}

/// Get the current state of the credential object
///
/// #Params
/// command_handle: command handle to map callback to user context.
///
/// proof_handle: Credential handle that was provided during creation.
///
/// cb: Callback that provides most current state of the credential and error status of request
///     Credential statuses:
///         2 - Request Sent
///         3 - Request Received
///         4 - Accepted
///
/// #Returns
#[no_mangle]
pub extern fn vcx_credential_get_state(command_handle: CommandHandle,
                                       handle: u32,
                                       cb: Option<extern fn(xcommand_handle: CommandHandle, err: u32, state: u32)>) -> u32 {
    info!("vcx_credential_get_state >>>");

    check_useful_c_callback!(cb, VcxErrorKind::InvalidOption);

    if !credential::is_valid_handle(handle) {
        return VcxError::from(VcxErrorKind::InvalidCredentialHandle).into();
    }

    let source_id = credential::get_source_id(handle).unwrap_or_default();
    trace!("vcx_credential_get_state(command_handle: {}, credential_handle: {}), source_id: {:?}",
           command_handle, handle, source_id);

    spawn(move || {
        match credential::get_state(handle) {
            Ok(s) => {
                trace!("vcx_credential_get_state_cb(command_handle: {}, rc: {}, state: {}), source_id: {:?}",
                       command_handle, error::SUCCESS.message, s, source_id);
                cb(command_handle, error::SUCCESS.code_num, s)
            }
            Err(e) => {
                error!("vcx_credential_get_state_cb(command_handle: {}, rc: {}, state: {}), source_id: {:?}",
                       command_handle, e, 0, source_id);
                cb(command_handle, e.into(), 0)
            }
        };

        Ok(())
    });

    error::SUCCESS.code_num
}


/// Takes the credential object and returns a json string of all its attributes
///
/// #Params
/// command_handle: command handle to map callback to user context.
///
/// handle: Credential handle that was provided during creation. Used to identify credential object
///
/// cb: Callback that provides json string of the credential's attributes and provides error status
///
/// #Returns
/// Error code as a u32
#[no_mangle]
pub extern fn vcx_credential_serialize(command_handle: CommandHandle,
                                       handle: u32,
                                       cb: Option<extern fn(xcommand_handle: CommandHandle, err: u32, data: *const c_char)>) -> u32 {
    info!("vcx_credential_serialize >>>");

    check_useful_c_callback!(cb, VcxErrorKind::InvalidOption);

    if !credential::is_valid_handle(handle) {
        return VcxError::from(VcxErrorKind::InvalidCredentialHandle).into();
    }

    let source_id = credential::get_source_id(handle).unwrap_or_default();
    trace!("vcx_credential_serialize(command_handle: {}, credential_handle: {}), source_id: {:?}",
           command_handle, handle, source_id);

    spawn(move || {
        match credential::to_string(handle) {
            Ok(x) => {
                trace!("vcx_credential_serialize_cb(command_handle: {}, rc: {}, data: {}), source_id: {:?}",
                       command_handle, error::SUCCESS.message, x, source_id);
                let msg = CStringUtils::string_to_cstring(x);
                cb(command_handle, error::SUCCESS.code_num, msg.as_ptr());
            }
            Err(x) => {
                error!("vcx_credential_serialize_cb(command_handle: {}, rc: {}, data: {}), source_id: {:?}",
                       command_handle, x, 0, source_id);
                cb(command_handle, x.into(), ptr::null_mut());
            }
        };

        Ok(())
    });

    error::SUCCESS.code_num
}

/// Takes a json string representing an credential object and recreates an object matching the json
///
/// #Params
/// command_handle: command handle to map callback to user context.
///
/// credential_data: json string representing a credential object
///
///
/// cb: Callback that provides credential handle and provides error status
///
/// #Returns
/// Error code as a u32
#[no_mangle]
pub extern fn vcx_credential_deserialize(command_handle: CommandHandle,
                                         credential_data: *const c_char,
                                         cb: Option<extern fn(xcommand_handle: CommandHandle, err: u32, handle: u32)>) -> u32 {
    info!("vcx_credential_deserialize >>>");

    check_useful_c_callback!(cb, VcxErrorKind::InvalidOption);
    check_useful_c_str!(credential_data, VcxErrorKind::InvalidOption);

    trace!("vcx_credential_deserialize(command_handle: {}, credential_data: {})",
           command_handle, credential_data);

    spawn(move || {
        match credential::from_string(&credential_data) {
            Ok(x) => {
                trace!("vcx_credential_deserialize_cb(command_handle: {}, rc: {}, credential_handle: {}) source_id: {}",
                       command_handle, error::SUCCESS.message, x, credential::get_source_id(x).unwrap_or_default());

                cb(command_handle, error::SUCCESS.code_num, x);
            }
            Err(x) => {
                error!("vcx_credential_deserialize_cb(command_handle: {}, rc: {}, credential_handle: {}) source_id: {}",
                       command_handle, x, 0, "");
                cb(command_handle, x.into(), 0);
            }
        };

        Ok(())
    });

    error::SUCCESS.code_num
}

/// Releases the credential object by de-allocating memory
///
/// #Params
/// handle: Credential handle that was provided during creation. Used to access credential object
///
/// #Returns
/// Success
#[no_mangle]
pub extern fn vcx_credential_release(handle: u32) -> u32 {
    info!("vcx_credential_release >>>");

    let source_id = credential::get_source_id(handle).unwrap_or_default();
    match credential::release(handle) {
        Ok(()) => {
            trace!("vcx_credential_release(handle: {}, rc: {}), source_id: {:?}",
                   handle, error::SUCCESS.message, source_id);
            error::SUCCESS.code_num
        }

        Err(e) => {
            error!("vcx_credential_release(handle: {}, rc: {}), source_id: {:?}",
                   handle, e, source_id);
            e.into()
        }
    }
}

#[no_mangle]
pub extern fn vcx_credential_get_payment_txn(command_handle: CommandHandle,
                                             handle: u32,
                                             cb: Option<extern fn(xcommand_handle: CommandHandle, err: u32, txn: *const c_char)>) -> u32 {
    info!("vcx_credential_get_payment_txn >>>");

    check_useful_c_callback!(cb, VcxErrorKind::InvalidOption);

    let source_id = credential::get_source_id(handle).unwrap_or_default();
    trace!("vcx_credential_get_payment_txn(command_handle: {}) source_id: {}", command_handle, source_id);

    spawn(move || {
        error!("Payments not supported yet");
        cb(command_handle, 1, ptr::null());
        Ok(())
    });

    error::SUCCESS.code_num
}

#[cfg(test)]
mod tests {
    extern crate serde_json;

    use std::ffi::CString;

    use serde_json::Value;

    use ::credential::tests::BAD_CREDENTIAL_OFFER;
    use agency_comm::mocking::AgencyMockDecrypted;
    use api::return_types_u32;
    use api::VcxStateType;
    use utils::constants::{GET_MESSAGES_DECRYPTED_RESPONSE, V3_OBJECT_SERIALIZE_VERSION};
    use utils::devsetup::*;
    use utils::mockdata::mockdata_credex::{ARIES_CREDENTIAL_OFFER, ARIES_CREDENTIAL_REQUEST, ARIES_CREDENTIAL_RESPONSE, CREDENTIAL_SM_FINISHED, CREDENTIAL_SM_OFFER_RECEIVED};
    use utils::timeout::TimeoutUtils;

    use super::*;

    fn _vcx_credential_create_with_offer_c_closure(offer: &str) -> Result<u32, u32> {
        let cb = return_types_u32::Return_U32_U32::new().unwrap();
        let rc = vcx_credential_create_with_offer(cb.command_handle,
                                                  CString::new("test_create").unwrap().into_raw(),
                                                  CString::new(offer).unwrap().into_raw(),
                                                  Some(cb.get_callback()));
        if rc != error::SUCCESS.code_num {
            return Err(rc);
        }

        let handle = cb.receive(TimeoutUtils::some_medium());
        handle
    }

    #[test]
    #[cfg(feature = "general_test")]
    fn test_vcx_credential_create_with_offer_success() {
        let _setup = SetupMocks::init();

        let handle = _vcx_credential_create_with_offer_c_closure(ARIES_CREDENTIAL_OFFER).unwrap();
        assert!(handle > 0);
    }

    #[test]
    #[cfg(feature = "general_test")]
    fn test_vcx_credential_create_with_offer_fails() {
        let _setup = SetupMocks::init();

        let err = _vcx_credential_create_with_offer_c_closure(BAD_CREDENTIAL_OFFER).unwrap_err();
        assert_eq!(err, error::INVALID_JSON.code_num);
    }

    #[test]
    #[cfg(feature = "general_test")]
    fn test_vcx_credential_serialize_and_deserialize() {
        let _setup = SetupMocks::init();

        let handle = _vcx_credential_create_with_offer_c_closure(ARIES_CREDENTIAL_OFFER).unwrap();

        let cb = return_types_u32::Return_U32_STR::new().unwrap();
        assert_eq!(vcx_credential_serialize(cb.command_handle,
                                            handle,
                                            Some(cb.get_callback())), error::SUCCESS.code_num);
        let credential_json = cb.receive(TimeoutUtils::some_short()).unwrap().unwrap();
        debug!("Serialized credential: {:?}", credential_json);

        let object: Value = serde_json::from_str(&credential_json).unwrap();
        assert_eq!(object["version"], V3_OBJECT_SERIALIZE_VERSION);

        let cb = return_types_u32::Return_U32_U32::new().unwrap();
        assert_eq!(vcx_credential_deserialize(cb.command_handle,
                                              CString::new(credential_json).unwrap().into_raw(),
                                              Some(cb.get_callback())), error::SUCCESS.code_num);
        let handle = cb.receive(TimeoutUtils::some_short()).unwrap();
        assert!(handle > 0);
    }

    #[test]
    #[cfg(feature = "general_test")]
    fn test_vcx_credential_send_request() {
        let _setup = SetupMocks::init();

        let handle = credential::credential_create_with_offer("test_send_request", ARIES_CREDENTIAL_OFFER).unwrap();
        assert_eq!(credential::get_state(handle).unwrap(), VcxStateType::VcxStateRequestReceived as u32);

        let connection_handle = connection::tests::build_test_connection_inviter_requested();

        let cb = return_types_u32::Return_U32::new().unwrap();
        assert_eq!(vcx_credential_send_request(cb.command_handle, handle, connection_handle, 0, Some(cb.get_callback())), error::SUCCESS.code_num);
        cb.receive(TimeoutUtils::some_medium()).unwrap();
    }

    #[test]
    #[cfg(feature = "general_test")]
    fn test_vcx_credential_get_new_offers() {
        let _setup = SetupMocks::init();

        let handle_conn = ::connection::tests::build_test_connection_invitee_completed();

        let cb = return_types_u32::Return_U32_STR::new().unwrap();
        assert_eq!(vcx_credential_get_offers(cb.command_handle,
                                             handle_conn,
                                             Some(cb.get_callback())),
                   error::SUCCESS.code_num as u32);
        cb.receive(TimeoutUtils::some_medium()).unwrap();
    }

    #[test]
    #[cfg(feature = "general_test")]
    fn test_vcx_credential_create() {
        let _setup = SetupMocks::init();

        let handle_conn = ::connection::tests::build_test_connection_invitee_completed();

        let cb = return_types_u32::Return_U32_U32_STR::new().unwrap();
        assert_eq!(vcx_credential_create_with_msgid(cb.command_handle,
                                                    CString::new("test_vcx_credential_create").unwrap().into_raw(),
                                                    handle_conn,
                                                    CString::new("123").unwrap().into_raw(),
                                                    Some(cb.get_callback())), error::SUCCESS.code_num);
        cb.receive(TimeoutUtils::some_medium()).unwrap();
    }

    #[test]
    #[cfg(feature = "general_test")]
    fn test_vcx_credential_get_state() {
        let _setup = SetupMocks::init();

        let handle = _vcx_credential_create_with_offer_c_closure(ARIES_CREDENTIAL_OFFER).unwrap();

        let cb = return_types_u32::Return_U32_U32::new().unwrap();
        assert_eq!(vcx_credential_get_state(cb.command_handle, handle, Some(cb.get_callback())), error::SUCCESS.code_num);
        assert_eq!(cb.receive(TimeoutUtils::some_medium()).unwrap(), VcxStateType::VcxStateRequestReceived as u32);
    }

    #[test]
    #[cfg(feature = "general_test")]
    fn test_vcx_credential_update_state() {
        let _setup = SetupMocks::init();

        let handle_conn = ::connection::tests::build_test_connection_inviter_requested();

        let handle_cred = _vcx_credential_create_with_offer_c_closure(ARIES_CREDENTIAL_OFFER).unwrap();
        assert_eq!(credential::get_state(handle_cred).unwrap(), VcxStateType::VcxStateRequestReceived as u32);
        debug!("credential handle = {}", handle_cred);

        let cb = return_types_u32::Return_U32::new().unwrap();
        assert_eq!(vcx_credential_send_request(cb.command_handle, handle_cred, handle_conn, 0, Some(cb.get_callback())), error::SUCCESS.code_num);
        cb.receive(TimeoutUtils::some_medium()).unwrap();
        assert_eq!(credential::get_state(handle_cred).unwrap(), VcxStateType::VcxStateOfferSent as u32);

        AgencyMockDecrypted::set_next_decrypted_response(GET_MESSAGES_DECRYPTED_RESPONSE);
        AgencyMockDecrypted::set_next_decrypted_message(ARIES_CREDENTIAL_RESPONSE);
        let cb = return_types_u32::Return_U32_U32::new().unwrap();
        assert_eq!(vcx_credential_update_state(cb.command_handle, handle_cred, Some(cb.get_callback())), error::SUCCESS.code_num);
        cb.receive(TimeoutUtils::some_medium()).unwrap();
        assert_eq!(credential::get_state(handle_cred).unwrap(), VcxStateType::VcxStateAccepted as u32);

        let cb = return_types_u32::Return_U32_STR::new().unwrap();
        assert_eq!(vcx_credential_get_rev_reg_id(cb.command_handle, handle_cred, Some(cb.get_callback())), error::SUCCESS.code_num);
        let rev_reg_id = cb.receive(TimeoutUtils::some_medium()).unwrap().unwrap();
        let rev_reg_id_expected = String::from("V4SGRU86Z58d6TV7PBUe6f:4:V4SGRU86Z58d6TV7PBUe6f:3:CL:1281:tag1:CL_ACCUM:tag1");
        assert_eq!(rev_reg_id, rev_reg_id_expected);
    }

    #[test]
    #[cfg(feature = "general_test")]
    #[cfg(feature = "to_restore")] // Missing implementation for v3 in generate_credential_request_msg
    fn test_vcx_credential_get_request_msg() {
        let _setup = SetupMocks::init();

        let handle_conn = ::connection::tests::build_test_connection_inviter_invited();

        let my_pw_did = CString::new(::connection::get_pw_did(handle_conn).unwrap()).unwrap().into_raw();
        let their_pw_did = CString::new(::connection::get_their_pw_did(handle_conn).unwrap()).unwrap().into_raw();

        let handle_cred = _vcx_credential_create_with_offer_c_closure(ARIES_CREDENTIAL_OFFER).unwrap();

        let cb = return_types_u32::Return_U32_STR::new().unwrap();
        assert_eq!(vcx_credential_get_request_msg(cb.command_handle, handle_cred, my_pw_did, their_pw_did, 0, Some(cb.get_callback())), error::SUCCESS.code_num);
        let msg = cb.receive(TimeoutUtils::some_medium()).unwrap().unwrap();

        ::serde_json::from_str::<CredentialRequest>(&msg).unwrap();
    }

    #[test]
    #[cfg(feature = "general_test")]
    fn test_get_credential() {
        let _setup = SetupMocks::init();

        let handle_cred = credential::from_string(CREDENTIAL_SM_FINISHED).unwrap();
        let cb = return_types_u32::Return_U32_STR::new().unwrap();
        assert_eq!(vcx_get_credential(cb.command_handle, handle_cred, Some(cb.get_callback())), error::SUCCESS.code_num);
        cb.receive(TimeoutUtils::some_medium()).unwrap().unwrap();

        let bad_handle = 1123;
        let cb = return_types_u32::Return_U32_STR::new().unwrap();
        assert_eq!(vcx_get_credential(cb.command_handle, bad_handle, Some(cb.get_callback())), error::INVALID_CREDENTIAL_HANDLE.code_num);
    }

    #[test]
    #[cfg(feature = "general_test")]
    fn test_vcx_credential_release() {
        let _setup = SetupMocks::init();

        let handle = _vcx_credential_create_with_offer_c_closure(ARIES_CREDENTIAL_OFFER).unwrap();

        assert_eq!(vcx_credential_release(handle + 1), error::INVALID_CREDENTIAL_HANDLE.code_num);

        assert_eq!(vcx_credential_release(handle), error::SUCCESS.code_num);

        assert_eq!(vcx_credential_release(handle), error::INVALID_CREDENTIAL_HANDLE.code_num);
    }
}
