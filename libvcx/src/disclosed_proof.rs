use std::convert::TryInto;

use serde_json;

use aries::{
    handlers::proof_presentation::prover::prover::Prover,
    messages::proof_presentation::presentation_request::PresentationRequest,
};
use connection;
use error::prelude::*;
use messages::{
    get_message::Message,
    payload::Payloads,
};
use messages::proofs::proof_request::ProofRequestMessage;
use settings;
use settings::indy_mocks_enabled;
use utils::constants::GET_MESSAGES_DECRYPTED_RESPONSE;
use utils::error;
use utils::httpclient::AgencyMockDecrypted;
use utils::mockdata::mockdata_proof::ARIES_PROOF_REQUEST_PRESENTATION;
use utils::object_cache::ObjectCache;

lazy_static! {
    static ref HANDLE_MAP: ObjectCache<Prover> = ObjectCache::<Prover>::new("disclosed-proofs-cache");
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(tag = "version", content = "data")]
enum DisclosedProofs {
    #[serde(rename = "2.0")]
    V3(Prover),
}

fn handle_err(err: VcxError) -> VcxError {
    if err.kind() == VcxErrorKind::InvalidHandle {
        VcxError::from(VcxErrorKind::InvalidDisclosedProofHandle)
    } else {
        err
    }
}

pub fn create_proof(source_id: &str, proof_req: &str) -> VcxResult<u32> {
    trace!("create_proof >>> source_id: {}, proof_req: {}", source_id, proof_req);
    debug!("creating disclosed proof with id: {}", source_id);

    let presentation_request: PresentationRequest = serde_json::from_str(proof_req)
        .map_err(|err| VcxError::from_msg(VcxErrorKind::InvalidJson,
                                          format!("Strict `aries` protocol is enabled. Can not parse `aries` formatted Presentation Request: {}", err)))?;

    let proof = Prover::create(source_id, presentation_request)?;
    HANDLE_MAP.add(proof)
}

pub fn create_proof_with_msgid(source_id: &str, connection_handle: u32, msg_id: &str) -> VcxResult<(u32, String)> {
    if !connection::is_v3_connection(connection_handle)? {
        return Err(VcxError::from_msg(VcxErrorKind::InvalidConnectionHandle, format!("Connection can not be used for Proprietary Issuance protocol")));
    };

    let proof_request = get_proof_request(connection_handle, &msg_id)?;

    let presentation_request: PresentationRequest = serde_json::from_str(&proof_request)
        .map_err(|err| VcxError::from_msg(VcxErrorKind::InvalidJson,
                                          format!("Strict `aries` protocol is enabled. Can not parse `aries` formatted Presentation Request: {}", err)))?;

    let proof = Prover::create(source_id, presentation_request)?;

    let handle = HANDLE_MAP.add(proof)?;

    debug!("inserting disclosed proof {} into handle map", source_id);
    Ok((handle, proof_request))
}

pub fn get_state(handle: u32) -> VcxResult<u32> {
    HANDLE_MAP.get(handle, |proof| {
        Ok(proof.state())
    }).or(Err(VcxError::from(VcxErrorKind::InvalidConnectionHandle)))
}

pub fn update_state(handle: u32, message: Option<String>, connection_handle: Option<u32>) -> VcxResult<u32> {
    HANDLE_MAP.get_mut(handle, |proof| {
        proof.update_state(message.as_ref().map(String::as_str), connection_handle)?;
        Ok(proof.state())
    })
}

pub fn to_string(handle: u32) -> VcxResult<String> {
    HANDLE_MAP.get(handle, |proof| {
        serde_json::to_string(&DisclosedProofs::V3(proof.clone()))
            .map_err(|err| VcxError::from_msg(VcxErrorKind::InvalidState, format!("cannot serialize DisclosedProof proofect: {:?}", err)))
    })
}

pub fn from_string(proof_data: &str) -> VcxResult<u32> {
    let proof: DisclosedProofs = serde_json::from_str(proof_data)
        .map_err(|err| VcxError::from_msg(VcxErrorKind::InvalidJson, format!("cannot deserialize DisclosedProofs object: {:?}", err)))?;

    match proof {
        DisclosedProofs::V3(proof) => HANDLE_MAP.add(proof)
    }
}

pub fn release(handle: u32) -> VcxResult<()> {
    HANDLE_MAP.release(handle).map_err(handle_err)
}

pub fn release_all() {
    HANDLE_MAP.drain().ok();
}

pub fn generate_proof_msg(handle: u32) -> VcxResult<String> {
    HANDLE_MAP.get(handle, |proof| {
        proof.generate_presentation_msg()
    })
}

pub fn send_proof(handle: u32, connection_handle: u32) -> VcxResult<u32> {
    HANDLE_MAP.get_mut(handle, |proof| {
        proof.send_presentation(connection_handle)?;
        let new_proof = proof.clone();
        *proof = new_proof;
        Ok(error::SUCCESS.code_num)
    })
}

pub fn generate_reject_proof_msg(handle: u32) -> VcxResult<String> {
    HANDLE_MAP.get_mut(handle, |_| {
        Err(VcxError::from_msg(VcxErrorKind::ActionNotSupported,
                               "Action generate_reject_proof_msg is not implemented for V3 disclosed proof."))
    })
}

pub fn reject_proof(handle: u32, connection_handle: u32) -> VcxResult<u32> {
    HANDLE_MAP.get_mut(handle, |proof| {
        proof.decline_presentation_request(connection_handle, Some(String::from("Presentation Request was rejected")), None)?;
        let new_proof = proof.clone();
        *proof = new_proof;
        Ok(error::SUCCESS.code_num)
    })
}

pub fn generate_proof(handle: u32, credentials: String, self_attested_attrs: String) -> VcxResult<u32> {
    HANDLE_MAP.get_mut(handle, |proof| {
        proof.generate_presentation(credentials.clone(), self_attested_attrs.clone())?;
        Ok(error::SUCCESS.code_num)
    }).map(|_| error::SUCCESS.code_num)
}

pub fn decline_presentation_request(handle: u32, connection_handle: u32, reason: Option<String>, proposal: Option<String>) -> VcxResult<u32> {
    HANDLE_MAP.get_mut(handle, |proof| {
        proof.decline_presentation_request(connection_handle, reason.clone(), proposal.clone())?;
        let new_proof = proof.clone();
        *proof = new_proof;
        Ok(error::SUCCESS.code_num)
    }).map(|_| error::SUCCESS.code_num)
}

pub fn retrieve_credentials(handle: u32) -> VcxResult<String> {
    HANDLE_MAP.get_mut(handle, |proof| {
        proof.retrieve_credentials()
    })
}

pub fn get_proof_request_data(handle: u32) -> VcxResult<String> {
    HANDLE_MAP.get_mut(handle, |proof| {
        proof.presentation_request_data()
    })
}

pub fn get_proof_request_attachment(handle: u32) -> VcxResult<String> {
    HANDLE_MAP.get_mut(handle, |proof| {
        proof.get_proof_request_attachment()
    })
}

pub fn is_valid_handle(handle: u32) -> bool {
    HANDLE_MAP.has_handle(handle)
}

fn get_proof_request(connection_handle: u32, msg_id: &str) -> VcxResult<String> {
    if !connection::is_v3_connection(connection_handle)? {
        return Err(VcxError::from_msg(VcxErrorKind::InvalidConnectionHandle, format!("Connection can not be used for Proprietary Issuance protocol")));
    };

    if indy_mocks_enabled() {
        AgencyMockDecrypted::set_next_decrypted_response(GET_MESSAGES_DECRYPTED_RESPONSE);
        AgencyMockDecrypted::set_next_decrypted_message(ARIES_PROOF_REQUEST_PRESENTATION);
    }

    let presentation_request = Prover::get_presentation_request(connection_handle, msg_id)?;
    serde_json::to_string_pretty(&presentation_request)
        .map_err(|err| VcxError::from_msg(VcxErrorKind::InvalidJson, format!("Cannot serialize message: {}", err)))
}

pub fn get_proof_request_messages(connection_handle: u32) -> VcxResult<String> {
    trace!("get_proof_request_messages >>> connection_handle: {}", connection_handle);

    if !connection::is_v3_connection(connection_handle)? {
        return Err(VcxError::from_msg(VcxErrorKind::InvalidConnectionHandle, format!("Connection can not be used for Proprietary Issuance protocol")));
    }

    let presentation_requests = Prover::get_presentation_request_messages(connection_handle)?;
    Ok(json!(presentation_requests).to_string())
}

fn _parse_proof_req_message(message: &Message, my_vk: &str) -> VcxResult<ProofRequestMessage> {
    let payload = message.payload.as_ref()
        .ok_or(VcxError::from_msg(VcxErrorKind::InvalidHttpResponse, "Cannot get payload"))?;

    let (request, thread) = Payloads::decrypt(&my_vk, payload)?;

    let mut request: ProofRequestMessage = serde_json::from_str(&request)
        .map_err(|err| VcxError::from_msg(VcxErrorKind::InvalidHttpResponse, format!("Cannot deserialize proof request: {}", err)))?;

    request.msg_ref_id = Some(message.uid.to_owned());
    request.thread_id = thread.and_then(|tr| tr.thid.clone());

    Ok(request)
}

pub fn get_source_id(handle: u32) -> VcxResult<String> {
    HANDLE_MAP.get(handle, |proof| {
        Ok(proof.get_source_id())
    }).map_err(handle_err)
}

pub fn get_presentation_status(handle: u32) -> VcxResult<u32> {
    HANDLE_MAP.get(handle, |proof| {
        Ok(proof.presentation_status())
    })
}

#[cfg(test)]
mod tests {
    extern crate serde_json;

    use serde_json::Value;
    #[cfg(feature = "pool_tests")]
    use time;

    use api::VcxStateType;
    use utils::{
        constants::{ADDRESS_CRED_DEF_ID, ADDRESS_CRED_ID, ADDRESS_CRED_REV_ID,
                    ADDRESS_REV_REG_ID, ADDRESS_SCHEMA_ID, ARIES_PROVER_CREDENTIALS, ARIES_PROVER_SELF_ATTESTED_ATTRS,
                    CRED_DEF_ID, CRED_REV_ID, GET_MESSAGES_DECRYPTED_RESPONSE, LICENCE_CRED_ID, REV_REG_ID,
                    REV_STATE_JSON, SCHEMA_ID, TEST_TAILS_FILE},
        get_temp_dir_path,
    };
    use utils::devsetup::*;
    use utils::httpclient::AgencyMockDecrypted;
    use utils::mockdata::mock_settings::MockBuilder;
    use utils::mockdata::mockdata_proof;
    use utils::mockdata::mockdata_proof::{ARIES_PROOF_PRESENTATION_ACK, ARIES_PROOF_REQUEST_PRESENTATION};
    use aries::messages::proof_presentation::presentation_request::PresentationRequestData;

    use super::*;

    fn _get_proof_request_messages(connection_h: u32) -> String {
        let requests = get_proof_request_messages(connection_h).unwrap();
        let requests: Value = serde_json::from_str(&requests).unwrap();
        let requests = serde_json::to_string(&requests[0]).unwrap();
        requests
    }

    #[test]
    #[cfg(feature = "general_test")]
    fn test_create_proof() {
        let _setup = SetupStrictAriesMocks::init();

        assert!(create_proof("1", ARIES_PROOF_REQUEST_PRESENTATION).unwrap() > 0);
    }

    #[test]
    #[cfg(feature = "general_test")]
    fn test_create_fails() {
        let _setup = SetupStrictAriesMocks::init();

        assert_eq!(create_proof("1", "{}").unwrap_err().kind(), VcxErrorKind::InvalidJson);
    }

    #[test]
    #[cfg(feature = "general_test")]
    fn test_proof_cycle() {
        let _setup = SetupStrictAriesMocks::init();

        let connection_h = connection::tests::build_test_connection_inviter_requested();

        AgencyMockDecrypted::set_next_decrypted_response(GET_MESSAGES_DECRYPTED_RESPONSE);
        AgencyMockDecrypted::set_next_decrypted_message(ARIES_PROOF_REQUEST_PRESENTATION);

        let request = _get_proof_request_messages(connection_h);

        let handle_proof = create_proof("TEST_CREDENTIAL", &request).unwrap();
        assert_eq!(VcxStateType::VcxStateRequestReceived as u32, get_state(handle_proof).unwrap());

        let _mock_builder = MockBuilder::init().
            set_mock_generate_indy_proof("{\"selected\":\"credentials\"}");

        generate_proof(handle_proof, String::from("{\"selected\":\"credentials\"}"), "{}".to_string()).unwrap();
        send_proof(handle_proof, connection_h).unwrap();
        assert_eq!(VcxStateType::VcxStateOfferSent as u32, get_state(handle_proof).unwrap());

        update_state(handle_proof, Some(String::from(ARIES_PROOF_PRESENTATION_ACK)), Some(connection_h)).unwrap();
        assert_eq!(VcxStateType::VcxStateAccepted as u32, get_state(handle_proof).unwrap());
    }

    #[test]
    #[cfg(feature = "general_test")]
    fn test_proof_update_state_v2() {
        let _setup = SetupStrictAriesMocks::init();

        let connection_handle = connection::tests::build_test_connection_inviter_requested();

        AgencyMockDecrypted::set_next_decrypted_response(GET_MESSAGES_DECRYPTED_RESPONSE);
        AgencyMockDecrypted::set_next_decrypted_message(mockdata_proof::ARIES_PRESENTATION_REQUEST);

        let request = _get_proof_request_messages(connection_handle);

        let handle = create_proof("TEST_CREDENTIAL", &request).unwrap();
        assert_eq!(VcxStateType::VcxStateRequestReceived as u32, get_state(handle).unwrap());

        generate_proof(handle, ARIES_PROVER_CREDENTIALS.to_string(), ARIES_PROVER_SELF_ATTESTED_ATTRS.to_string());
        assert_eq!(VcxStateType::VcxStateRequestReceived as u32, get_state(handle).unwrap());

        send_proof(handle, connection_handle).unwrap();
        assert_eq!(VcxStateType::VcxStateOfferSent as u32, get_state(handle).unwrap());

        ::connection::release(connection_handle);
        let connection_handle = connection::tests::build_test_connection_inviter_requested();

        AgencyMockDecrypted::set_next_decrypted_response(GET_MESSAGES_DECRYPTED_RESPONSE);
        AgencyMockDecrypted::set_next_decrypted_message(mockdata_proof::ARIES_PROOF_PRESENTATION_ACK);

        update_state(handle, None, Some(connection_handle));
        assert_eq!(VcxStateType::VcxStateAccepted as u32, get_state(handle).unwrap());
    }

    #[test]
    #[cfg(feature = "general_test")]
    fn test_proof_reject_cycle() {
        let _setup = SetupStrictAriesMocks::init();

        let connection_h = connection::tests::build_test_connection_inviter_requested();

        AgencyMockDecrypted::set_next_decrypted_response(GET_MESSAGES_DECRYPTED_RESPONSE);
        AgencyMockDecrypted::set_next_decrypted_message(ARIES_PROOF_REQUEST_PRESENTATION);

        let request = _get_proof_request_messages(connection_h);

        let handle = create_proof("TEST_CREDENTIAL", &request).unwrap();
        assert_eq!(VcxStateType::VcxStateRequestReceived as u32, get_state(handle).unwrap());

        reject_proof(handle, connection_h).unwrap();
        assert_eq!(VcxStateType::VcxStateNone as u32, get_state(handle).unwrap());
    }

    #[test]
    #[cfg(feature = "general_test")]
    fn get_state_test() {
        let _setup = SetupStrictAriesMocks::init();

        let handle = create_proof("id", ARIES_PROOF_REQUEST_PRESENTATION).unwrap();
        assert_eq!(VcxStateType::VcxStateRequestReceived as u32, get_state(handle).unwrap())
    }

    #[test]
    #[cfg(feature = "general_test")]
    fn to_string_test() {
        let _setup = SetupStrictAriesMocks::init();

        let handle = create_proof("id", ARIES_PROOF_REQUEST_PRESENTATION).unwrap();

        let serialized = to_string(handle).unwrap();
        let j: Value = serde_json::from_str(&serialized).unwrap();
        assert_eq!(j["version"], ::utils::constants::V3_OBJECT_SERIALIZE_VERSION);

        let handle_2 = from_string(&serialized).unwrap();
        assert_ne!(handle, handle_2);
    }

    #[test]
    #[cfg(feature = "general_test")]
    fn test_deserialize_fails() {
        let _setup = SetupDefaults::init();

        assert_eq!(from_string("{}").unwrap_err().kind(), VcxErrorKind::InvalidJson);
    }

    #[test]
    #[cfg(feature = "general_test")]
    fn test_get_proof_request() {
        let _setup = SetupStrictAriesMocks::init();

        let connection_h = connection::tests::build_test_connection_inviter_invited();

        let request = get_proof_request(connection_h, "123").unwrap();
        let _request: PresentationRequest = serde_json::from_str(&request).unwrap();
    }

    #[test]
    #[cfg(feature = "general_test")]
    fn test_deserialize_succeeds_with_self_attest_allowed() {
        let _setup = SetupDefaults::init();

        let handle = create_proof("id", ARIES_PROOF_REQUEST_PRESENTATION).unwrap();

        let serialized = to_string(handle).unwrap();
        from_string(&serialized).unwrap();
    }

    #[test]
    #[cfg(feature = "general_test")]
    fn test_get_proof_request_attachment() {
        let _setup = SetupStrictAriesMocks::init();

        let connection_h = connection::tests::build_test_connection_inviter_requested();

        AgencyMockDecrypted::set_next_decrypted_response(GET_MESSAGES_DECRYPTED_RESPONSE);
        AgencyMockDecrypted::set_next_decrypted_message(ARIES_PROOF_REQUEST_PRESENTATION);

        let request = _get_proof_request_messages(connection_h);

        let handle = create_proof("TEST_CREDENTIAL", &request).unwrap();
        assert_eq!(VcxStateType::VcxStateRequestReceived as u32, get_state(handle).unwrap());

        let attrs = get_proof_request_attachment(handle).unwrap();
        let attrs: PresentationRequestData = serde_json::from_str(&attrs).unwrap();
    }
}
