use serde_json;
use serde_json::Value;

use api::{ProofStateType, VcxStateType};
use error::prelude::*;
use messages::proofs::proof_message::{CredInfo, ProofMessage};
use messages::proofs::proof_message::get_credential_info;
use messages::proofs::proof_request::ProofRequestMessage;
use object_cache::ObjectCache;
use settings;
use settings::get_config_value;
use utils::constants::*;
use utils::error;
use utils::libindy::anoncreds;
use utils::openssl::encode;
use v3::handlers::proof_presentation::verifier::verifier::Verifier;

lazy_static! {
    static ref PROOF_MAP: ObjectCache<Verifier> = ObjectCache::<Verifier>::new("proofs-cache");
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(tag = "version", content = "data")]
enum Proofs {
    #[serde(rename = "2.0")]
    V3(Verifier),
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
pub struct Proof {}

impl Proof {
    fn validate_proof_revealed_attributes(proof_json: &str) -> VcxResult<()> {
        if settings::indy_mocks_enabled() { return Ok(()); }

        let proof: Value = serde_json::from_str(proof_json)
            .map_err(|err| VcxError::from_msg(VcxErrorKind::InvalidJson, format!("Cannot deserialize libndy proof: {}", err)))?;

        let revealed_attrs = match proof["requested_proof"]["revealed_attrs"].as_object() {
            Some(revealed_attrs) => revealed_attrs,
            None => return Ok(())
        };

        for (attr1_referent, info) in revealed_attrs.iter() {
            let raw = info["raw"].as_str().ok_or(VcxError::from_msg(VcxErrorKind::InvalidProof, format!("Cannot get raw value for \"{}\" attribute", attr1_referent)))?;
            let encoded_ = info["encoded"].as_str().ok_or(VcxError::from_msg(VcxErrorKind::InvalidProof, format!("Cannot get encoded value for \"{}\" attribute", attr1_referent)))?;

            let expected_encoded = encode(&raw)?;

            if expected_encoded != encoded_.to_string() {
                return Err(VcxError::from_msg(VcxErrorKind::InvalidProof, format!("Encoded values are different. Expected: {}. From Proof: {}", expected_encoded, encoded_)));
            }
        }

        Ok(())
    }

    fn build_credential_defs_json(credential_data: &Vec<CredInfo>) -> VcxResult<String> {
        debug!("building credential_def_json for proof validation");
        let mut credential_json = json!({});

        for ref cred_info in credential_data.iter() {
            if credential_json.get(&cred_info.cred_def_id).is_none() {
                let (id, credential_def) = anoncreds::get_cred_def_json(&cred_info.cred_def_id)?;

                let credential_def = serde_json::from_str(&credential_def)
                    .map_err(|err| VcxError::from_msg(VcxErrorKind::InvalidProofCredentialData, format!("Cannot deserialize credential definition: {}", err)))?;

                credential_json[id] = credential_def;
            }
        }

        Ok(credential_json.to_string())
    }

    fn build_schemas_json(credential_data: &Vec<CredInfo>) -> VcxResult<String> {
        debug!("building schemas json for proof validation");

        let mut schemas_json = json!({});

        for ref cred_info in credential_data.iter() {
            if schemas_json.get(&cred_info.schema_id).is_none() {
                let (id, schema_json) = anoncreds::get_schema_json(&cred_info.schema_id)
                    .map_err(|err| err.map(VcxErrorKind::InvalidSchema, "Cannot get schema"))?;

                let schema_val = serde_json::from_str(&schema_json)
                    .map_err(|err| VcxError::from_msg(VcxErrorKind::InvalidSchema, format!("Cannot deserialize schema: {}", err)))?;

                schemas_json[id] = schema_val;
            }
        }

        Ok(schemas_json.to_string())
    }

    fn build_rev_reg_defs_json(credential_data: &Vec<CredInfo>) -> VcxResult<String> {
        debug!("building rev_reg_def_json for proof validation");

        let mut rev_reg_defs_json = json!({});

        for ref cred_info in credential_data.iter() {
            let rev_reg_id = cred_info
                .rev_reg_id
                .as_ref()
                .ok_or(VcxError::from(VcxErrorKind::InvalidRevocationDetails))?;

            if rev_reg_defs_json.get(rev_reg_id).is_none() {
                let (id, json) = anoncreds::get_rev_reg_def_json(rev_reg_id)
                    .or(Err(VcxError::from(VcxErrorKind::InvalidRevocationDetails)))?;

                let rev_reg_def_json = serde_json::from_str(&json)
                    .or(Err(VcxError::from(VcxErrorKind::InvalidSchema)))?;

                rev_reg_defs_json[id] = rev_reg_def_json;
            }
        }

        Ok(rev_reg_defs_json.to_string())
    }

    fn build_rev_reg_json(credential_data: &Vec<CredInfo>) -> VcxResult<String> {
        debug!("building rev_reg_json for proof validation");

        let mut rev_regs_json = json!({});

        for ref cred_info in credential_data.iter() {
            let rev_reg_id = cred_info
                .rev_reg_id
                .as_ref()
                .ok_or(VcxError::from(VcxErrorKind::InvalidRevocationDetails))?;

            let timestamp = cred_info
                .timestamp
                .as_ref()
                .ok_or(VcxError::from(VcxErrorKind::InvalidRevocationTimestamp))?;

            if rev_regs_json.get(rev_reg_id).is_none() {
                let (id, json, timestamp) = anoncreds::get_rev_reg(rev_reg_id, timestamp.to_owned())
                    .or(Err(VcxError::from(VcxErrorKind::InvalidRevocationDetails)))?;

                let rev_reg_json: Value = serde_json::from_str(&json)
                    .or(Err(VcxError::from(VcxErrorKind::InvalidJson)))?;

                let rev_reg_json = json!({timestamp.to_string(): rev_reg_json});
                rev_regs_json[id] = rev_reg_json;
            }
        }

        Ok(rev_regs_json.to_string())
    }

    pub fn validate_indy_proof(proof_json: &str, proof_req_json: &str) -> VcxResult<bool> {
        if settings::indy_mocks_enabled() {
            let mock_result: bool = get_config_value(settings::MOCK_INDY_PROOF_VALIDATION).unwrap_or("true".into()).parse().unwrap();
            return Ok(mock_result);
        }

        Proof::validate_proof_revealed_attributes(&proof_json)?;

        let credential_data = get_credential_info(&proof_json)?;

        let credential_defs_json = Proof::build_credential_defs_json(&credential_data)
            .unwrap_or(json!({}).to_string());
        let schemas_json = Proof::build_schemas_json(&credential_data)
            .unwrap_or(json!({}).to_string());
        let rev_reg_defs_json = Proof::build_rev_reg_defs_json(&credential_data)
            .unwrap_or(json!({}).to_string());
        let rev_regs_json = Proof::build_rev_reg_json(&credential_data)
            .unwrap_or(json!({}).to_string());

        debug!("*******\n{}\n********", credential_defs_json);
        debug!("*******\n{}\n********", schemas_json);
        debug!("*******\n{}\n********", proof_json);
        debug!("*******\n{}\n********", proof_req_json);
        debug!("*******\n{}\n********", rev_reg_defs_json);
        debug!("*******\n{}\n********", rev_regs_json);
        anoncreds::libindy_verifier_verify_proof(proof_req_json,
                                                 proof_json,
                                                 &schemas_json,
                                                 &credential_defs_json,
                                                 &rev_reg_defs_json,
                                                 &rev_regs_json)
    }
}

pub fn create_proof(source_id: String,
                    requested_attrs: String,
                    requested_predicates: String,
                    revocation_details: String,
                    name: String) -> VcxResult<u32> {
    let verifier = Verifier::create(source_id, requested_attrs, requested_predicates, revocation_details, name)?;
    PROOF_MAP.add(verifier)
        .or(Err(VcxError::from(VcxErrorKind::CreateProof)))
}

pub fn is_valid_handle(handle: u32) -> bool {
    PROOF_MAP.has_handle(handle)
}

pub fn update_state(handle: u32, message: Option<String>, connection_handle: Option<u32>) -> VcxResult<u32> {
    PROOF_MAP.get_mut(handle, |proof| {
        proof.update_state(message.as_ref().map(String::as_str), connection_handle)?;
        Ok(proof.state())
    })
}

pub fn get_state(handle: u32) -> VcxResult<u32> {
    PROOF_MAP.get(handle, |proof| {
        Ok(proof.state())
    })
}

pub fn get_proof_state(handle: u32) -> VcxResult<u32> {
    PROOF_MAP.get(handle, |proof| {
        Ok(proof.presentation_status())
    })
}

pub fn release(handle: u32) -> VcxResult<()> {
    PROOF_MAP.release(handle).or(Err(VcxError::from(VcxErrorKind::InvalidProofHandle)))
}

pub fn release_all() {
    PROOF_MAP.drain().ok();
}

pub fn to_string(handle: u32) -> VcxResult<String> {
    PROOF_MAP.get(handle, |proof| {
        serde_json::to_string(&Proofs::V3(proof.clone()))
            .map_err(|err| VcxError::from_msg(VcxErrorKind::InvalidState, format!("cannot serialize Proof proofect: {:?}", err)))
    })
}

pub fn get_source_id(handle: u32) -> VcxResult<String> {
    PROOF_MAP.get(handle, |proof| {
        Ok(proof.get_source_id())
    })
}

pub fn from_string(proof_data: &str) -> VcxResult<u32> {
    let proof: Proofs = serde_json::from_str(proof_data)
        .map_err(|err| VcxError::from_msg(VcxErrorKind::InvalidJson, format!("cannot deserialize Proofs proofect: {:?}", err)))?;

    match proof {
        Proofs::V3(proof) => PROOF_MAP.add(proof),
        _ => Err(VcxError::from_msg(VcxErrorKind::InvalidJson, "Found proof of unsupported version"))
    } 
}

pub fn generate_proof_request_msg(handle: u32) -> VcxResult<String> {
    PROOF_MAP.get_mut(handle, |proof| {
        proof.generate_presentation_request_msg()
    })
}

pub fn send_proof_request(handle: u32, connection_handle: u32) -> VcxResult<u32> {
    PROOF_MAP.get_mut(handle, |proof| {
        proof.send_presentation_request(connection_handle)?;
        Ok(error::SUCCESS.code_num)
    })
}


fn parse_proof_payload(payload: &str) -> VcxResult<ProofMessage> {
    let my_credential_req = ProofMessage::from_str(&payload)
        .map_err(|err| VcxError::from_msg(VcxErrorKind::InvalidJson, format!("Cannot deserialize ProofMessage: {}", err)))?;
    Ok(my_credential_req)
}

pub fn get_proof(handle: u32) -> VcxResult<String> {
    PROOF_MAP.get(handle, |proof| {
        proof.get_presentation()
    })
}

#[cfg(test)]
pub mod tests {
    use super::*;
    use connection::tests::build_test_connection_inviter_requested;
    use utils::devsetup::*;
    use utils::httpclient::{HttpClientMockResponse};
    use utils::mockdata::mockdata_proof;
    use v3::handlers::proof_presentation::verifier::verifier::Verifier;
    
    fn create_default_proof() -> Verifier {
        let proof = Verifier::create("1".to_string(),
                                  REQUESTED_ATTRS.to_owned(),
                                  REQUESTED_PREDICATES.to_owned(),
                                  r#"{"support_revocation":false}"#.to_string(),
                                  "Optional".to_owned()).unwrap();
        // let handle = PROOF_MAP.add(proof).unwrap();
        return proof
    }

    fn progress_proof_to_final_state(proof: &mut Verifier, connection_handle: u32) {
        proof.send_presentation_request(connection_handle).unwrap();
        assert_eq!(proof.state(), VcxStateType::VcxStateOfferSent as u32);

        proof.update_state(Some(mockdata_proof::ARIES_PROOF_PRESENTATION), None).unwrap();
        assert_eq!(proof.state(), VcxStateType::VcxStateAccepted as u32);
    }

    #[test]
    #[cfg(feature = "general_test")]
    fn test_create_proof_succeeds() {
        let _setup = SetupStrictAriesMocks::init();

        create_proof("1".to_string(),
                     REQUESTED_ATTRS.to_owned(),
                     REQUESTED_PREDICATES.to_owned(),
                     r#"{"support_revocation":false}"#.to_string(),
                     "Optional".to_owned()).unwrap();
    }

    #[test]
    #[cfg(feature = "general_test")]
    fn test_revocation_details() {
        let _setup = SetupStrictAriesMocks::init();

        // No Revocation
        create_proof("1".to_string(),
                     REQUESTED_ATTRS.to_owned(),
                     REQUESTED_PREDICATES.to_owned(),
                     r#"{"support_revocation":false}"#.to_string(),
                     "Optional".to_owned()).unwrap();

        // Support Revocation Success
        let revocation_details = json!({
            "to": 1234,
        });
        create_proof("1".to_string(),
                     REQUESTED_ATTRS.to_owned(),
                     REQUESTED_PREDICATES.to_owned(),
                     revocation_details.to_string(),
                     "Optional".to_owned()).unwrap();
    }

    #[test]
    #[cfg(feature = "general_test")]
    fn test_to_string_succeeds() {
        let _setup = SetupStrictAriesMocks::init();

        let handle = create_proof("1".to_string(),
                                  REQUESTED_ATTRS.to_owned(),
                                  REQUESTED_PREDICATES.to_owned(),
                                  r#"{"support_revocation":false}"#.to_string(),
                                  "Optional".to_owned()).unwrap();
        let proof_string = to_string(handle).unwrap();
        let s: Value = serde_json::from_str(&proof_string).unwrap();
        assert_eq!(s["version"], V3_OBJECT_SERIALIZE_VERSION);
        assert!(s["data"]["verifier_sm"].is_object());
        assert!(!proof_string.is_empty());
    }

    #[test]
    #[cfg(feature = "general_test")]
    fn test_from_string_succeeds() {
        let _setup = SetupStrictAriesMocks::init();

        let handle = create_proof("1".to_string(),
                                  REQUESTED_ATTRS.to_owned(),
                                  REQUESTED_PREDICATES.to_owned(),
                                  r#"{"support_revocation":false}"#.to_string(),
                                  "Optional".to_owned()).unwrap();
        let proof_data = to_string(handle).unwrap();
        let _hnadle2 = from_string(&proof_data).unwrap();
        let proof_data2 = to_string(handle).unwrap();
        assert_eq!(proof_data, proof_data2);
    }

    #[test]
    #[cfg(feature = "general_test")]
    fn test_release_proof() {
        let _setup = SetupStrictAriesMocks::init();

        let handle = create_proof("1".to_string(),
                                  REQUESTED_ATTRS.to_owned(),
                                  REQUESTED_PREDICATES.to_owned(),
                                  r#"{"support_revocation":false}"#.to_string(),
                                  "Optional".to_owned()).unwrap();
        assert!(release(handle).is_ok());
        assert!(!is_valid_handle(handle));
    }

    #[test]
    #[cfg(feature = "general_test")]
    fn test_send_proof_request() {
        let _setup = SetupStrictAriesMocks::init();

        let connection_handle = build_test_connection_inviter_requested();

        let proof_handle = create_proof("1".to_string(),
                                        REQUESTED_ATTRS.to_owned(),
                                        REQUESTED_PREDICATES.to_owned(),
                                        r#"{"support_revocation":false}"#.to_string(),
                                        "Optional".to_owned()).unwrap();
        assert_eq!(send_proof_request(proof_handle, connection_handle).unwrap(), error::SUCCESS.code_num);
        assert_eq!(get_state(proof_handle).unwrap(), VcxStateType::VcxStateOfferSent as u32);
    }

    #[test]
    #[cfg(feature = "general_test")]
    fn test_get_proof_fails_with_no_proof() {
        let _setup = SetupStrictAriesMocks::init();

        let handle = create_proof("1".to_string(),
                                  REQUESTED_ATTRS.to_owned(),
                                  REQUESTED_PREDICATES.to_owned(),
                                  r#"{"support_revocation":false}"#.to_string(),
                                  "Optional".to_owned()).unwrap();
        assert!(is_valid_handle(handle));
        assert!(get_proof(handle).is_err())
    }

    #[test]
    #[cfg(feature = "general_test")]
    fn test_proof_update_state_v2() {
        let _setup = SetupStrictAriesMocks::init();

        let connection_handle = build_test_connection_inviter_requested();

        let mut proof = create_default_proof();
        proof.send_presentation_request(connection_handle).unwrap();
        assert_eq!(proof.state(), VcxStateType::VcxStateOfferSent as u32);

        ::connection::release(connection_handle);
        let connection_handle = build_test_connection_inviter_requested();

        proof.update_state(Some(mockdata_proof::ARIES_PROOF_PRESENTATION), Some(connection_handle)).unwrap();

        assert_eq!(proof.state(), VcxStateType::VcxStateAccepted as u32);
    }

    #[test]
    #[cfg(feature = "general_test")]
    fn test_update_state_with_message() {
        let _setup = SetupStrictAriesMocks::init();

        let connection_handle = build_test_connection_inviter_requested();

        let mut proof = create_default_proof();
        progress_proof_to_final_state(&mut proof, connection_handle);
    }

    #[test]
    #[cfg(feature = "general_test")]
    fn test_build_credential_defs_json_with_multiple_credentials() {
        let _setup = SetupStrictAriesMocks::init();

        let cred1 = CredInfo {
            schema_id: "schema_key1".to_string(),
            cred_def_id: "cred_def_key1".to_string(),
            rev_reg_id: None,
            timestamp: None,
        };
        let cred2 = CredInfo {
            schema_id: "schema_key2".to_string(),
            cred_def_id: "cred_def_key2".to_string(),
            rev_reg_id: None,
            timestamp: None,
        };
        let credentials = vec![cred1, cred2];
        let credential_json = Proof::build_credential_defs_json(&credentials).unwrap();

        let json: Value = serde_json::from_str(CRED_DEF_JSON).unwrap();
        let expected = json!({CRED_DEF_ID:json}).to_string();
        assert_eq!(credential_json, expected);
    }

    #[test]
    #[cfg(feature = "general_test")]
    fn test_build_schemas_json_with_multiple_schemas() {
        let _setup = SetupStrictAriesMocks::init();

        let cred1 = CredInfo {
            schema_id: "schema_key1".to_string(),
            cred_def_id: "cred_def_key1".to_string(),
            rev_reg_id: None,
            timestamp: None,
        };
        let cred2 = CredInfo {
            schema_id: "schema_key2".to_string(),
            cred_def_id: "cred_def_key2".to_string(),
            rev_reg_id: None,
            timestamp: None,
        };
        let credentials = vec![cred1, cred2];
        let schema_json = Proof::build_schemas_json(&credentials).unwrap();

        let json: Value = serde_json::from_str(SCHEMA_JSON).unwrap();
        let expected = json!({SCHEMA_ID:json}).to_string();
        assert_eq!(schema_json, expected);
    }

    #[test]
    #[cfg(feature = "general_test")]
    fn test_build_rev_reg_defs_json() {
        let _setup = SetupStrictAriesMocks::init();

        let cred1 = CredInfo {
            schema_id: "schema_key1".to_string(),
            cred_def_id: "cred_def_key1".to_string(),
            rev_reg_id: Some("id1".to_string()),
            timestamp: None,
        };
        let cred2 = CredInfo {
            schema_id: "schema_key2".to_string(),
            cred_def_id: "cred_def_key2".to_string(),
            rev_reg_id: Some("id2".to_string()),
            timestamp: None,
        };
        let credentials = vec![cred1, cred2];
        let rev_reg_defs_json = Proof::build_rev_reg_defs_json(&credentials).unwrap();

        let json: Value = serde_json::from_str(&rev_def_json()).unwrap();
        let expected = json!({REV_REG_ID:json}).to_string();
        assert_eq!(rev_reg_defs_json, expected);
    }

    #[test]
    #[cfg(feature = "general_test")]
    fn test_build_rev_reg_json() {
        let _setup = SetupStrictAriesMocks::init();

        let cred1 = CredInfo {
            schema_id: "schema_key1".to_string(),
            cred_def_id: "cred_def_key1".to_string(),
            rev_reg_id: Some("id1".to_string()),
            timestamp: Some(1),
        };
        let cred2 = CredInfo {
            schema_id: "schema_key2".to_string(),
            cred_def_id: "cred_def_key2".to_string(),
            rev_reg_id: Some("id2".to_string()),
            timestamp: Some(2),
        };
        let credentials = vec![cred1, cred2];
        let rev_reg_json = Proof::build_rev_reg_json(&credentials).unwrap();

        let json: Value = serde_json::from_str(REV_REG_JSON).unwrap();
        let expected = json!({REV_REG_ID:{"1":json}}).to_string();
        assert_eq!(rev_reg_json, expected);
    }

    #[test]
    #[cfg(feature = "general_test")]
    fn test_get_proof() {
        let _setup = SetupStrictAriesMocks::init();

        let connection_handle = build_test_connection_inviter_requested();
        let mut proof = create_default_proof();
        progress_proof_to_final_state(&mut proof, connection_handle);

        let handle = PROOF_MAP.add(proof).unwrap();
        let proof_str = get_proof(handle).unwrap();
        // TODO: Why don't these equal? Parse compare values?
        // assert_eq!(&proof_str, mockdata_proof::ARIES_PROOF_PRESENTATION);
    }

    #[test]
    #[cfg(feature = "general_test")]
    fn test_release_all() {
        let _setup = SetupStrictAriesMocks::init();

        let h1 = create_proof("1".to_string(), REQUESTED_ATTRS.to_owned(), REQUESTED_PREDICATES.to_owned(), r#"{"support_revocation":false}"#.to_string(), "Optional".to_owned()).unwrap();
        let h2 = create_proof("1".to_string(), REQUESTED_ATTRS.to_owned(), REQUESTED_PREDICATES.to_owned(), r#"{"support_revocation":false}"#.to_string(), "Optional".to_owned()).unwrap();
        let h3 = create_proof("1".to_string(), REQUESTED_ATTRS.to_owned(), REQUESTED_PREDICATES.to_owned(), r#"{"support_revocation":false}"#.to_string(), "Optional".to_owned()).unwrap();
        let h4 = create_proof("1".to_string(), REQUESTED_ATTRS.to_owned(), REQUESTED_PREDICATES.to_owned(), r#"{"support_revocation":false}"#.to_string(), "Optional".to_owned()).unwrap();
        let h5 = create_proof("1".to_string(), REQUESTED_ATTRS.to_owned(), REQUESTED_PREDICATES.to_owned(), r#"{"support_revocation":false}"#.to_string(), "Optional".to_owned()).unwrap();
        release_all();
        assert_eq!(release(h1).unwrap_err().kind(), VcxErrorKind::InvalidProofHandle);
        assert_eq!(release(h2).unwrap_err().kind(), VcxErrorKind::InvalidProofHandle);
        assert_eq!(release(h3).unwrap_err().kind(), VcxErrorKind::InvalidProofHandle);
        assert_eq!(release(h4).unwrap_err().kind(), VcxErrorKind::InvalidProofHandle);
        assert_eq!(release(h5).unwrap_err().kind(), VcxErrorKind::InvalidProofHandle);
    }

    #[test]
    #[cfg(feature = "general_test")]
    fn test_send_proof_request_can_be_retried() {
        let _setup = SetupStrictAriesMocks::init();
        settings::set_config_value(settings::CONFIG_PROTOCOL_TYPE, "4.0");

        let handle_conn = build_test_connection_inviter_requested();

        let handle_proof = create_proof("1".to_string(),
                                        REQUESTED_ATTRS.to_owned(),
                                        REQUESTED_PREDICATES.to_owned(),
                                        r#"{"support_revocation":false}"#.to_string(),
                                        "Optional".to_owned()).unwrap();
        let _request = generate_proof_request_msg(handle_proof).unwrap();
        assert_eq!(get_state(handle_proof).unwrap(), VcxStateType::VcxStateInitialized as u32);

        HttpClientMockResponse::set_next_response(VcxResult::Err(VcxError::from_msg(VcxErrorKind::IOError, "Sending message timeout.")));
        assert_eq!(send_proof_request(handle_proof, handle_conn).unwrap_err().kind(), VcxErrorKind::IOError);
        assert_eq!(get_state(handle_proof).unwrap(), VcxStateType::VcxStateInitialized as u32);

        // Retry sending proof request
        assert_eq!(send_proof_request(handle_proof, handle_conn).unwrap(), 0);
        assert_eq!(get_state(handle_proof).unwrap(), VcxStateType::VcxStateOfferSent as u32);
    }

    #[test]
    #[cfg(feature = "general_test")]
    fn test_proof_accepted() {
        let _setup = SetupStrictAriesMocks::init();

        let handle_conn = build_test_connection_inviter_requested();

        let handle_proof = create_proof("1".to_string(),
                                        REQUESTED_ATTRS.to_owned(),
                                        REQUESTED_PREDICATES.to_owned(),
                                        r#"{"support_revocation":false}"#.to_string(),
                                        "Optional".to_owned()).unwrap();
        let _request = generate_proof_request_msg(handle_proof).unwrap();
        send_proof_request(handle_proof, handle_conn).unwrap();
        update_state(handle_proof, Some(mockdata_proof::ARIES_PROOF_PRESENTATION.to_string()), Some(handle_conn)).unwrap();
        assert_eq!(::proof::get_state(handle_proof).unwrap(), VcxStateType::VcxStateAccepted as u32);
    }

    #[test]
    #[cfg(feature = "general_test")]
    fn test_proof_errors() {
        SetupLibraryWallet::init();
        let _setup = SetupStrictAriesMocks::init();

        let connection_handle = build_test_connection_inviter_requested();

        let proof = create_default_proof();
        let proof_handle = PROOF_MAP.add(proof).unwrap();

        let bad_handle = 100000;
        let empty = r#""#;

        assert_eq!(send_proof_request(bad_handle, connection_handle).unwrap_err().kind(), VcxErrorKind::InvalidHandle);
        assert_eq!(get_proof_state(proof_handle).unwrap(), 0);
        assert_eq!(create_proof("my source id".to_string(),
                                empty.to_string(),
                                "{}".to_string(),
                                r#"{"support_revocation":false}"#.to_string(),
                                "my name".to_string()).unwrap_err().kind(), VcxErrorKind::InvalidJson);
        assert_eq!(to_string(bad_handle).unwrap_err().kind(), VcxErrorKind::InvalidHandle);
        assert_eq!(get_source_id(bad_handle).unwrap_err().kind(), VcxErrorKind::InvalidHandle);
        assert_eq!(from_string(empty).unwrap_err().kind(), VcxErrorKind::InvalidJson);
    }

    #[cfg(feature = "pool_tests")]
    #[cfg(feature = "to_restore")]
    #[test]
    fn test_proof_validate_attribute() {
        let _setup = SetupLibraryWalletPoolZeroFees::init();

        let (_, _, proof_req, proof_json) = ::utils::libindy::anoncreds::tests::create_proof();

        let mut proof_req_obj = ProofRequestMessage::create();

        proof_req_obj.proof_request_data = serde_json::from_str(&proof_req).unwrap();

        let mut proof_msg = ProofMessage::new();
        let mut proof = create_boxed_proof(None, None, None);
        proof.proof_request = Some(proof_req_obj);

        // valid proof_obj
        {
            proof_msg.libindy_proof = proof_json.clone();
            proof.proof = Some(proof_msg);

            let _rc = proof.proof_validation().unwrap();
            assert_eq!(proof.proof_state, ProofStateType::ProofValidated);
        }

        let mut proof_obj: serde_json::Value = serde_json::from_str(&proof_json).unwrap();

        // change Raw value
        {
            let mut proof_msg = ProofMessage::new();
            proof_obj["requested_proof"]["revealed_attrs"]["address1_1"]["raw"] = json!("Other Value");
            let proof_json = serde_json::to_string(&proof_obj).unwrap();

            proof_msg.libindy_proof = proof_json;
            proof.proof = Some(proof_msg);

            let rc = proof.proof_validation();
            rc.unwrap_err();
            assert_eq!(proof.get_proof_state(), ProofStateType::ProofInvalid as u32);
        }

        // change Encoded value
        {
            let mut proof_msg = ProofMessage::new();
            proof_obj["requested_proof"]["revealed_attrs"]["address1_1"]["encoded"] = json!("1111111111111111111111111111111111111111111111111111111111");
            let proof_json = serde_json::to_string(&proof_obj).unwrap();

            proof_msg.libindy_proof = proof_json;
            proof.proof = Some(proof_msg);

            let rc = proof.proof_validation();
            rc.unwrap_err(); //FIXME check error code also
            assert_eq!(proof.get_proof_state(), ProofStateType::ProofInvalid as u32);
        }
    }
}
