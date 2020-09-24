use std::convert::TryInto;

use serde_json::Value;

use ::{serde_json, settings};
use api::VcxStateType;
use connection;
use error::prelude::*;
use issuer_credential::{CredentialMessage, CredentialOffer, PaymentInfo};
use messages::{
    self,
    get_message::{
        get_connection_messages,
        get_ref_msg,
        MessagePayload,
    },
    payload::{
        PayloadKinds,
        Payloads,
    },
    RemoteMessageType,
    thread::Thread,
};
use object_cache::ObjectCache;
use utils::{constants, error};
use utils::agent_info::{get_agent_attr, get_agent_info, MyAgentInfo};
use utils::constants::GET_MESSAGES_DECRYPTED_RESPONSE;
use utils::httpclient::{AgencyMock, AgencyMockDecrypted};
use credential_request::CredentialRequest;
use utils::libindy::payments::{pay_a_payee, PaymentTxn};
use utils::mockdata::mockdata_credex::ARIES_CREDENTIAL_OFFER;
use v3::{
    handlers::issuance::Holder,
    messages::issuance::credential_offer::CredentialOffer as CredentialOfferV3,
    messages::issuance::credential::Credential as CredentialV3
};
use settings::indy_mocks_enabled;

lazy_static! {
    static ref HANDLE_MAP: ObjectCache<Holder> = ObjectCache::<Holder>::new("credentials-cache");
}

// This enum is left only to avoid making breaking serialization changes
#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(tag = "version", content = "data")]
enum Credentials {
    #[serde(rename = "2.0")]
    V3(Holder)
}

#[derive(Clone, Serialize, Deserialize, Debug, PartialEq)]
pub struct Credential {
}

fn handle_err(err: VcxError) -> VcxError {
    if err.kind() == VcxErrorKind::InvalidHandle {
        VcxError::from(VcxErrorKind::InvalidCredentialHandle)
    } else {
        err
    }
}

fn create_credential_v3(source_id: &str, offer: &str) -> VcxResult<Option<Holder>> {
    trace!("create_credential_v3 >>> source_id: {}, offer: {}", source_id, secret!(&offer));

    let offer_message = ::serde_json::from_str::<serde_json::Value>(offer)
        .map_err(|err| VcxError::from_msg(VcxErrorKind::InvalidJson, format!("Cannot deserialize Message: {:?}", err)))?;

    let offer_message = match offer_message {
        serde_json::Value::Array(_) => return Err(VcxError::from_msg(VcxErrorKind::InvalidJson, "Received offer in legacy format")),
        offer => offer 
    };

    if let Ok(cred_offer) = serde_json::from_value::<CredentialOfferV3>(offer_message) {
        return Ok(Some(Holder::create(cred_offer, source_id)?));
    }

    // TODO: Return error in case of error
    Ok(None)
}

pub fn credential_create_with_offer(source_id: &str, offer: &str) -> VcxResult<u32> {
    trace!("credential_create_with_offer >>> source_id: {}, offer: {}", source_id, secret!(&offer));

    let cred_offer: CredentialOfferV3 = serde_json::from_str(offer)
        .map_err(|err| VcxError::from_msg(VcxErrorKind::InvalidJson,
                                          format!("Strict `aries` protocol is enabled. Can not parse `aries` formatted Credential Offer: {}", err)))?;

    let holder = Holder::create(cred_offer, source_id)?;
    return HANDLE_MAP.add(holder);
}

pub fn credential_create_with_msgid(source_id: &str, connection_handle: u32, msg_id: &str) -> VcxResult<(u32, String)> {
    trace!("credential_create_with_msgid >>> source_id: {}, connection_handle: {}, msg_id: {}", source_id, connection_handle, secret!(&msg_id));

    let offer = get_credential_offer_msg(connection_handle, &msg_id)?;
    trace!("credential_create_with_msgid ::: for msg_id {} found offer {}", msg_id, offer);

    let credential = create_credential_v3(source_id, &offer)?
            .ok_or(VcxError::from_msg(VcxErrorKind::InvalidConnectionHandle, format!("Connection can not be used for Proprietary Issuance protocol")))?;

    let handle = HANDLE_MAP.add(credential)?;

    debug!("inserting credential {} into handle map", source_id);
    Ok((handle, offer))
}

pub fn update_state(handle: u32, message: Option<String>, connection_handle: Option<u32>) -> VcxResult<u32> {
    HANDLE_MAP.get_mut(handle, |credential| {
        credential.update_state(message.clone(), connection_handle)?;
        Ok(error::SUCCESS.code_num)
    })
}

pub fn get_credential(handle: u32) -> VcxResult<String> {
    HANDLE_MAP.get(handle, |credential| {
        Ok(json!(credential.get_credential()?.1).to_string())
    })
}

pub fn delete_credential(handle: u32) -> VcxResult<u32> {
    let source_id = get_source_id(handle).unwrap_or_default();
    trace!("Credential::delete_credential >>> credential_handle: {}, source_id: {}", handle, source_id);

    HANDLE_MAP.get(handle, |credential| {
        trace!("Deleting a credential: credential_handle {}, source_id {}", handle, source_id);

        credential.delete_credential()?;
        Ok(error::SUCCESS.code_num)
    })
        .map(|_| error::SUCCESS.code_num)
        .or(Err(VcxError::from(VcxErrorKind::InvalidCredentialHandle)))
        .and(release(handle))
        .and_then(|_| Ok(error::SUCCESS.code_num))
}

pub fn get_credential_offer(handle: u32) -> VcxResult<String> {
    HANDLE_MAP.get(handle, |credential| {
        Err(VcxError::from(VcxErrorKind::InvalidCredentialHandle)) // TODO: implement
    })
}

pub fn get_state(handle: u32) -> VcxResult<u32> {
    HANDLE_MAP.get(handle, |credential| {
        Ok(credential.get_status())
    }).map_err(handle_err)
}

/// #Returns
/// Credential request message serialized as String
pub fn generate_credential_request_msg(handle: u32, _my_pw_did: &str, _their_pw_did: &str) -> VcxResult<String> {
    HANDLE_MAP.get_mut(handle, |credential| {
        Err(VcxError::from_msg(VcxErrorKind::ActionNotSupported, "This action is not implemented yet")) // TODO: implement
    }).map_err(handle_err)
}

pub fn send_credential_request(handle: u32, connection_handle: u32) -> VcxResult<u32> {
    trace!("Credential::send_credential_request >>> credential_handle: {}, connection_handle: {}", handle, connection_handle);
    HANDLE_MAP.get_mut(handle, |credential| {
        credential.send_request(connection_handle)?;
        let new_credential = credential.clone(); // TODO: Why are we doing this exactly?
        *credential = new_credential;
        Ok(error::SUCCESS.code_num)
    }).map_err(handle_err)
}

fn get_credential_offer_msg(connection_handle: u32, msg_id: &str) -> VcxResult<String> {
    trace!("get_credential_offer_msg >>> connection_handle: {}, msg_id: {}", connection_handle, msg_id);

    if indy_mocks_enabled() {
        AgencyMockDecrypted::set_next_decrypted_response(GET_MESSAGES_DECRYPTED_RESPONSE);
        AgencyMockDecrypted::set_next_decrypted_message(ARIES_CREDENTIAL_OFFER);
    }
    let credential_offer = Holder::get_credential_offer_message(connection_handle, msg_id)?;

    return serde_json::to_string(&credential_offer).
        map_err(|err| {
            VcxError::from_msg(VcxErrorKind::InvalidState, format!("Cannot serialize Offers: {:?}", err))
        });
}

pub fn get_credential_offer_messages(connection_handle: u32) -> VcxResult<String> {
    trace!("Credential::get_credential_offer_messages >>> connection_handle: {}", connection_handle);

    AgencyMockDecrypted::set_next_decrypted_response(GET_MESSAGES_DECRYPTED_RESPONSE);
    AgencyMockDecrypted::set_next_decrypted_message(ARIES_CREDENTIAL_OFFER);

    let credential_offers = Holder::get_credential_offer_messages(connection_handle)?;

    Ok(json!(credential_offers).to_string())
}

fn _set_cred_offer_ref_message(payload: &MessagePayload, my_vk: &str, msg_id: &str) -> VcxResult<Vec<Value>> {
    let (offer, thread) = Payloads::decrypt(my_vk, payload)?;

    let (mut offer, payment_info) = parse_json_offer(&offer)?;

    offer.msg_ref_id = Some(msg_id.to_owned());
    if let Some(tr) = thread {
        offer.thread_id = tr.thid.clone();
    }

    let mut payload = Vec::new();
    payload.push(json!(offer));
    if let Some(p) = payment_info { payload.push(json!(p)); }

    Ok(payload)
}

pub fn parse_json_offer(offer: &str) -> VcxResult<(CredentialOffer, Option<PaymentInfo>)> {
    let paid_offer: Value = serde_json::from_str(offer)
        .map_err(|err| VcxError::from_msg(VcxErrorKind::InvalidJson, format!("Cannot deserialize offer: {}", err)))?;

    let mut payment: Option<PaymentInfo> = None;
    let mut offer: Option<CredentialOffer> = None;

    if let Some(i) = paid_offer.as_array() {
        for entry in i.iter() {
            if entry.get("libindy_offer").is_some() {
                offer = Some(serde_json::from_value(entry.clone())
                    .map_err(|err| VcxError::from_msg(VcxErrorKind::InvalidJson, format!("Cannot deserialize offer: {}", err)))?);
            }

            if entry.get("payment_addr").is_some() {
                payment = Some(serde_json::from_value(entry.clone())
                    .map_err(|err| VcxError::from_msg(VcxErrorKind::InvalidJson, format!("Cannot deserialize payment address: {}", err)))?);
            }
        }
    }
    Ok((offer.ok_or(VcxError::from(VcxErrorKind::InvalidJson))?, payment))
}

pub fn release(handle: u32) -> VcxResult<()> {
    HANDLE_MAP.release(handle).map_err(handle_err)
}

pub fn release_all() {
    HANDLE_MAP.drain().ok();
}

pub fn is_valid_handle(handle: u32) -> bool {
    HANDLE_MAP.has_handle(handle)
}

pub fn to_string(handle: u32) -> VcxResult<String> {
    HANDLE_MAP.get(handle, |credential| {
        serde_json::to_string(&Credentials::V3(credential.clone()))
            .map_err(|err| VcxError::from_msg(VcxErrorKind::InvalidState, format!("cannot serialize Credential credentialect: {:?}", err)))
    })
}

pub fn get_source_id(handle: u32) -> VcxResult<String> {
    HANDLE_MAP.get(handle, |credential| {
        Ok(credential.get_source_id())
    }).map_err(handle_err)
}

pub fn from_string(credential_data: &str) -> VcxResult<u32> {
    let credential: Credentials = serde_json::from_str(credential_data) 
        .map_err(|err| VcxError::from_msg(VcxErrorKind::InvalidJson, format!("Cannot deserialize Credential: {:?}", err)))?;

    match credential {
        Credentials::V3(credential) => HANDLE_MAP.add(credential),
        _ => Err(VcxError::from_msg(VcxErrorKind::InvalidJson, "Found credential of unsupported version"))
    } 
}

pub fn is_payment_required(handle: u32) -> VcxResult<bool> {
    HANDLE_MAP.get(handle, |_| {
        Ok(false)
    }).map_err(handle_err)
}

pub fn submit_payment(handle: u32) -> VcxResult<(PaymentTxn, String)> {
    HANDLE_MAP.get(handle, |_| {
        Err(VcxError::from_msg(VcxErrorKind::ActionNotSupported, "Aries protocol currently doesn't support payments"))
    }).map_err(handle_err)
}

pub fn get_payment_information(handle: u32) -> VcxResult<Option<PaymentInfo>> {
    HANDLE_MAP.get(handle, |_| {
        Ok(None)
    }).map_err(handle_err)
}

pub fn get_credential_status(handle: u32) -> VcxResult<u32> {
    HANDLE_MAP.get(handle, |credential| {
        credential.get_credential_status()
    })
}

#[cfg(test)]
pub mod tests {
    use utils::devsetup::*;
    use utils::mockdata::mockdata_credex::{ARIES_CREDENTIAL_RESPONSE, CREDENTIAL_SM_FINISHED, CREDENTIAL_SM_OFFER_RECEIVED};

    use super::*;

    pub const BAD_CREDENTIAL_OFFER: &str = r#"{"version": "0.1","to_did": "LtMgSjtFcyPwenK9SHCyb8","from_did": "LtMgSjtFcyPwenK9SHCyb8","claim": {"account_num": ["8BEaoLf8TBmK4BUyX8WWnA"],"name_on_account": ["Alice"]},"schema_seq_no": 48,"issuer_did": "Pd4fnFtRBcMKRVC2go5w3j","claim_name": "Account Certificate","claim_id": "3675417066","msg_ref_id": "ymy5nth"}"#;

    fn _get_offer(handle: u32) -> String {
        let offers = get_credential_offer_messages(handle).unwrap();
        let offers: serde_json::Value = serde_json::from_str(&offers).unwrap();
        let offer = serde_json::to_string(&offers[0]).unwrap();
        offer
    }

    #[test]
    #[cfg(feature = "general_test")]
    fn test_credential_create_with_offer() {
        let _setup = SetupDefaults::init();

        let handle = credential_create_with_offer("test_credential_create_with_offer", ARIES_CREDENTIAL_OFFER).unwrap();
        assert!(handle > 0);
    }

    #[test]
    #[cfg(feature = "general_test")]
    fn test_credential_create_with_bad_offer() {
        let _setup = SetupDefaults::init();

        let err = credential_create_with_offer("test_credential_create_with_bad_offer", BAD_CREDENTIAL_OFFER).unwrap_err();
        assert_eq!(err.kind(), VcxErrorKind::InvalidJson);
    }

    #[test]
    #[cfg(feature = "general_test")]
    fn test_credential_serialize_deserialize() {
        let _setup = SetupDefaults::init();

        let handle1 = credential_create_with_offer("test_credential_serialize_deserialize", ARIES_CREDENTIAL_OFFER).unwrap();
        let cred_original_state = get_state(handle1).unwrap();
        let cred_original_serialized = to_string(handle1).unwrap();
        release(handle1).unwrap();

        let handle2 = from_string(&cred_original_serialized).unwrap();
        let cred_restored_serialized = to_string(handle2).unwrap();
        let cred_restored_state = get_state(handle2).unwrap();

        assert_eq!(cred_original_state, cred_restored_state);
        assert_eq!(cred_original_serialized, cred_restored_serialized);
    }

    #[test]
    #[cfg(feature = "general_test")]
    fn full_credential_test() {
        let _setup = SetupStrictAriesMocks::init();

        info!("full_credential_test:: going to build_test_connection");
        let handle_conn = connection::tests::build_test_connection_inviter_requested();

        info!("full_credential_test:: going to _get_offer");
        let offer = _get_offer(handle_conn);

        info!("full_credential_test:: going to credential_create_with_offer");
        let handle_cred = credential_create_with_offer("TEST_CREDENTIAL", &offer).unwrap();
        assert_eq!(VcxStateType::VcxStateRequestReceived as u32, get_state(handle_cred).unwrap());

        info!("full_credential_test:: going to send_credential_request");
        send_credential_request(handle_cred, handle_conn).unwrap();
        assert_eq!(VcxStateType::VcxStateOfferSent as u32, get_state(handle_cred).unwrap());

        AgencyMockDecrypted::set_next_decrypted_response(GET_MESSAGES_DECRYPTED_RESPONSE);
        AgencyMockDecrypted::set_next_decrypted_message(ARIES_CREDENTIAL_RESPONSE);

        info!("full_credential_test:: going to update_state, should receive credential");
        update_state(handle_cred, None, Some(handle_conn)).unwrap();
        assert_eq!(get_state(handle_cred).unwrap(), VcxStateType::VcxStateAccepted as u32);

        info!("full_credential_test:: going to get_credential");
        let msg = get_credential(handle_cred).unwrap();
        let msg_value: serde_json::Value = serde_json::from_str(&msg).unwrap();

        info!("full_credential_test:: going to deserialize CredentialMessage: {:?}", msg_value);
        let _credential_struct: CredentialV3 = serde_json::from_str(msg_value.to_string().as_str()).unwrap();
    }

    #[test]
    #[cfg(feature = "general_test")]
    #[cfg(feature = "to_restore")] // todo: generate_credential_request_msg is not implemented for v3
    fn test_get_request_msg() {
        let _setup = SetupAriesMocks::init();

        let connection_h = connection::tests::build_test_connection_inviter_invited();

        let offer = _get_offer(connection_h);

        let my_pw_did = ::connection::get_pw_did(connection_h).unwrap();
        let their_pw_did = ::connection::get_their_pw_did(connection_h).unwrap();

        let c_h = credential_create_with_offer("TEST_CREDENTIAL", &offer).unwrap();
        assert_eq!(VcxStateType::VcxStateRequestReceived as u32, get_state(c_h).unwrap());

        let msg = generate_credential_request_msg(c_h, &my_pw_did, &their_pw_did).unwrap();
        ::serde_json::from_str::<CredentialRequest>(&msg).unwrap();
    }

    #[test]
    #[cfg(feature = "general_test")]
    fn test_get_credential_offer() {
        let _setup = SetupAriesMocks::init();

        let connection_h = connection::tests::build_test_connection_inviter_invited();

        let offer = get_credential_offer_messages(connection_h).unwrap();
        let o: serde_json::Value = serde_json::from_str(&offer).unwrap();
        println!("Serialized credential offer: {:?}", &o[0]);
        let _credential_offer: CredentialOfferV3 = serde_json::from_str(&o[0].to_string()).unwrap();
    }

    #[test]
    #[cfg(feature = "general_test")]
    #[cfg(feature = "to_restore")] // get_credential_offer not implemented for aries
    fn test_get_credential_offer_and_deserialize() {
        let _setup = SetupAriesMocks::init();

        let handle = from_string(CREDENTIAL_SM_OFFER_RECEIVED).unwrap();
        let offer_string = get_credential_offer(handle).unwrap();
        serde_json::Value::from(offer_string);
    }

    #[test]
    #[cfg(feature = "general_test")]
    fn test_get_credential_and_deserialize() {
        let _setup = SetupAriesMocks::init();

        let handle = from_string(CREDENTIAL_SM_FINISHED).unwrap();
        let cred_string: String = get_credential(handle).unwrap();
        let cred_value: Value = serde_json::from_str(&cred_string).unwrap();
        let _credential_struct: CredentialV3 = serde_json::from_str(cred_value.to_string().as_str()).unwrap();
    }
}
