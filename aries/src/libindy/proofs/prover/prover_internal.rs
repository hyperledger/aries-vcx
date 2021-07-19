use std::collections::HashMap;

use serde_json::Value;

use crate::error::prelude::*;
use crate::libindy::proofs::proof_request::ProofRequestData;
use crate::libindy::proofs::proof_request_internal::NonRevokedInterval;
use crate::libindy::utils::anoncreds;
use crate::libindy::utils::anoncreds::{get_rev_reg_def_json, get_rev_reg_delta_json};

#[derive(Debug, Deserialize, Serialize, PartialEq)]
pub struct CredInfoProver {
    pub requested_attr: String,
    pub referent: String,
    pub schema_id: String,
    pub cred_def_id: String,
    pub rev_reg_id: Option<String>,
    pub cred_rev_id: Option<String>,
    pub revocation_interval: Option<NonRevokedInterval>,
    pub tails_file: Option<String>,
    pub timestamp: Option<u64>,
}

pub fn build_schemas_json_prover(credentials_identifiers: &Vec<CredInfoProver>) -> VcxResult<String> {
    trace!("build_schemas_json_prover >>> credentials_identifiers: {:?}", credentials_identifiers);
    let mut rtn: Value = json!({});

    for ref cred_info in credentials_identifiers {
        if rtn.get(&cred_info.schema_id).is_none() {
            let (_, schema_json) = anoncreds::get_schema_json(&cred_info.schema_id)
                .map_err(|err| err.map(VcxErrorKind::InvalidSchema, "Cannot get schema"))?;

            let schema_json = serde_json::from_str(&schema_json)
                .map_err(|err| VcxError::from_msg(VcxErrorKind::InvalidSchema, format!("Cannot deserialize schema: {}", err)))?;

            rtn[cred_info.schema_id.to_owned()] = schema_json;
        }
    }
    Ok(rtn.to_string())
}

pub fn build_cred_defs_json_prover(credentials_identifiers: &Vec<CredInfoProver>) -> VcxResult<String> {
    trace!("build_cred_defs_json_prover >>> credentials_identifiers: {:?}", credentials_identifiers);
    let mut rtn: Value = json!({});

    for ref cred_info in credentials_identifiers {
        if rtn.get(&cred_info.cred_def_id).is_none() {
            let (_, credential_def) = anoncreds::get_cred_def_json(&cred_info.cred_def_id)
                .map_err(|err| err.map(VcxErrorKind::InvalidProofCredentialData, "Cannot get credential definition"))?;

            let credential_def = serde_json::from_str(&credential_def)
                .map_err(|err| VcxError::from_msg(VcxErrorKind::InvalidProofCredentialData, format!("Cannot deserialize credential definition: {}", err)))?;

            rtn[cred_info.cred_def_id.to_owned()] = credential_def;
        }
    }
    Ok(rtn.to_string())
}

pub fn credential_def_identifiers(credentials: &str, proof_req: &ProofRequestData) -> VcxResult<Vec<CredInfoProver>> {
    trace!("credential_def_identifiers >>> credentials: {:?}, proof_req: {:?}", credentials, proof_req);
    let mut rtn = Vec::new();

    let credentials: Value = serde_json::from_str(credentials)
        .map_err(|err| VcxError::from_msg(VcxErrorKind::InvalidJson, format!("Cannot deserialize credentials: {}", err)))?;

    if let Value::Object(ref attrs) = credentials["attrs"] {
        for (requested_attr, value) in attrs {
            if let (Some(referent), Some(schema_id), Some(cred_def_id)) =
            (value["credential"]["cred_info"]["referent"].as_str(),
             value["credential"]["cred_info"]["schema_id"].as_str(),
             value["credential"]["cred_info"]["cred_def_id"].as_str()) {
                let rev_reg_id = value["credential"]["cred_info"]["rev_reg_id"]
                    .as_str()
                    .map(|x| x.to_string());

                let cred_rev_id = value["credential"]["cred_info"]["cred_rev_id"]
                    .as_str()
                    .map(|x| x.to_string());

                let tails_file = value["tails_file"]
                    .as_str()
                    .map(|x| x.to_string());

                rtn.push(
                    CredInfoProver {
                        requested_attr: requested_attr.to_string(),
                        referent: referent.to_string(),
                        schema_id: schema_id.to_string(),
                        cred_def_id: cred_def_id.to_string(),
                        revocation_interval: _get_revocation_interval(&requested_attr, &proof_req)?,
                        timestamp: None,
                        rev_reg_id,
                        cred_rev_id,
                        tails_file,
                    }
                );
            } else { return Err(VcxError::from_msg(VcxErrorKind::InvalidProofCredentialData, "Cannot get identifiers")); }
        }
    }

    Ok(rtn)
}

fn _get_revocation_interval(attr_name: &str, proof_req: &ProofRequestData) -> VcxResult<Option<NonRevokedInterval>> {
    if let Some(attr) = proof_req.requested_attributes.get(attr_name) {
        Ok(attr.non_revoked.clone().or(proof_req.non_revoked.clone().or(None)))
    } else if let Some(attr) = proof_req.requested_predicates.get(attr_name) {
        // Handle case for predicates
        Ok(attr.non_revoked.clone().or(proof_req.non_revoked.clone().or(None)))
    } else {
        Err(VcxError::from_msg(VcxErrorKind::InvalidProofCredentialData, format!("Attribute not found for: {}", attr_name)))
    }
}

pub fn build_rev_states_json(credentials_identifiers: &mut Vec<CredInfoProver>) -> VcxResult<String> {
    trace!("build_rev_states_json >> credentials_identifiers: {:?}", credentials_identifiers);
    let mut rtn: Value = json!({});
    let mut timestamps: HashMap<String, u64> = HashMap::new();

    for cred_info in credentials_identifiers.iter_mut() {
        if let (Some(rev_reg_id), Some(cred_rev_id), Some(tails_file)) =
        (&cred_info.rev_reg_id, &cred_info.cred_rev_id, &cred_info.tails_file) {
            if rtn.get(&rev_reg_id).is_none() { // Does this make sense in case cred_info's for same rev_reg_ids have different revocation intervals
                let (from, to) = if let Some(ref interval) = cred_info.revocation_interval
                { (interval.from, interval.to) } else { (None, None) };

                let (_, rev_reg_def_json) = get_rev_reg_def_json(&rev_reg_id)?;

                let (rev_reg_id, rev_reg_delta_json, timestamp) = get_rev_reg_delta_json(
                    &rev_reg_id,
                    from,
                    to,
                )?;

                let rev_state_json = anoncreds::libindy_prover_create_revocation_state(
                    &rev_reg_def_json,
                    &rev_reg_delta_json,
                    &cred_rev_id,
                    &tails_file,
                )?;

                let rev_state_json: Value = serde_json::from_str(&rev_state_json)
                    .map_err(|err| VcxError::from_msg(VcxErrorKind::InvalidJson, format!("Cannot deserialize RevocationState: {}", err)))?;

                // TODO: proover should be able to create multiple states of same revocation policy for different timestamps
                // see ticket IS-1108
                rtn[rev_reg_id.to_string()] = json!({timestamp.to_string(): rev_state_json});
                cred_info.timestamp = Some(timestamp);

                // Cache timestamp for future attributes that have the same rev_reg_id
                timestamps.insert(rev_reg_id.to_string(), timestamp);
            }

            // If the rev_reg_id is already in the map, timestamp may not be updated on cred_info
            // All further credential info gets the same timestamp as the first one
            if cred_info.timestamp.is_none() {
                cred_info.timestamp = timestamps.get(rev_reg_id).cloned();
            }
        }
    }

    Ok(rtn.to_string())
}

pub fn build_requested_credentials_json(credentials_identifiers: &Vec<CredInfoProver>,
                                        self_attested_attrs: &str,
                                        proof_req: &ProofRequestData) -> VcxResult<String> {
    trace!("build_requested_credentials_json >> credentials_identifiers: {:?}, self_attested_attrs: {:?}, proof_req: {:?}", credentials_identifiers, self_attested_attrs, proof_req);
    let mut rtn: Value = json!({
          "self_attested_attributes":{},
          "requested_attributes":{},
          "requested_predicates":{}
    });
    // do same for predicates and self_attested
    if let Value::Object(ref mut map) = rtn["requested_attributes"] {
        for ref cred_info in credentials_identifiers {
            if let Some(_) = proof_req.requested_attributes.get(&cred_info.requested_attr) {
                let insert_val = json!({"cred_id": cred_info.referent, "revealed": true, "timestamp": cred_info.timestamp});
                map.insert(cred_info.requested_attr.to_owned(), insert_val);
            }
        }
    }

    if let Value::Object(ref mut map) = rtn["requested_predicates"] {
        for ref cred_info in credentials_identifiers {
            if let Some(_) = proof_req.requested_predicates.get(&cred_info.requested_attr) {
                let insert_val = json!({"cred_id": cred_info.referent, "timestamp": cred_info.timestamp});
                map.insert(cred_info.requested_attr.to_owned(), insert_val);
            }
        }
    }

    // handle if the attribute is not revealed
    let self_attested_attrs: Value = serde_json::from_str(self_attested_attrs)
        .map_err(|err| VcxError::from_msg(VcxErrorKind::InvalidJson, format!("Cannot deserialize self attested attributes: {}", err)))?;
    rtn["self_attested_attributes"] = self_attested_attrs;

    Ok(rtn.to_string())
}


#[cfg(test)]
pub mod tests {
    use crate::libindy::proofs::proof_request_internal::NonRevokedInterval;
    use crate::libindy::proofs::prover::prover_internal::CredInfoProver;
    use crate::utils::{
        constants::{ADDRESS_CRED_DEF_ID, ADDRESS_CRED_ID, ADDRESS_CRED_REV_ID,
                    ADDRESS_REV_REG_ID, ADDRESS_SCHEMA_ID,
                    CRED_DEF_ID, CRED_REV_ID, LICENCE_CRED_ID, REV_REG_ID,
                    REV_STATE_JSON, SCHEMA_ID, TEST_TAILS_FILE},
        get_temp_dir_path,
    };
    use crate::utils::devsetup::*;

    use super::*;

    fn proof_req_no_interval() -> ProofRequestData {
        let proof_req = json!({
            "nonce": "123432421212",
            "name": "proof_req_1",
            "version": "0.1",
            "requested_attributes": {
                "address1_1": { "name": "address1" },
                "zip_2": { "name": "zip" },
                "height_1": { "name": "height" }
            },
            "requested_predicates": {},
        }).to_string();

        serde_json::from_str(&proof_req).unwrap()
    }

    #[test]
    #[cfg(feature = "general_test")]
    fn test_find_credential_def() {
        let _setup = SetupMocks::init();

        let cred1 = CredInfoProver {
            requested_attr: "height_1".to_string(),
            referent: LICENCE_CRED_ID.to_string(),
            schema_id: SCHEMA_ID.to_string(),
            cred_def_id: CRED_DEF_ID.to_string(),
            rev_reg_id: Some(REV_REG_ID.to_string()),
            cred_rev_id: Some(CRED_REV_ID.to_string()),
            revocation_interval: None,
            tails_file: None,
            timestamp: None,
        };
        let cred2 = CredInfoProver {
            requested_attr: "zip_2".to_string(),
            referent: ADDRESS_CRED_ID.to_string(),
            schema_id: ADDRESS_SCHEMA_ID.to_string(),
            cred_def_id: ADDRESS_CRED_DEF_ID.to_string(),
            rev_reg_id: Some(ADDRESS_REV_REG_ID.to_string()),
            cred_rev_id: Some(ADDRESS_CRED_REV_ID.to_string()),
            revocation_interval: None,
            tails_file: None,
            timestamp: None,
        };
        let creds = vec![cred1, cred2];

        let credential_def = build_cred_defs_json_prover(&creds).unwrap();
        assert!(credential_def.len() > 0);
        assert!(credential_def.contains(r#""id":"V4SGRU86Z58d6TV7PBUe6f:3:CL:47:tag1","schemaId":"47""#));
    }

    #[test]
    #[cfg(feature = "general_test")]
    fn test_find_credential_def_fails() {
        let _setup = SetupLibraryWallet::init();

        let credential_ids = vec![CredInfoProver {
            requested_attr: "1".to_string(),
            referent: "2".to_string(),
            schema_id: "3".to_string(),
            cred_def_id: "3".to_string(),
            rev_reg_id: Some("4".to_string()),
            cred_rev_id: Some("5".to_string()),
            revocation_interval: None,
            tails_file: None,
            timestamp: None,
        }];
        assert_eq!(build_cred_defs_json_prover(&credential_ids).unwrap_err().kind(), VcxErrorKind::InvalidProofCredentialData);
    }

    #[test]
    #[cfg(feature = "general_test")]
    fn test_find_schemas_fails() {
        let _setup = SetupLibraryWallet::init();

        let credential_ids = vec![CredInfoProver {
            requested_attr: "1".to_string(),
            referent: "2".to_string(),
            schema_id: "3".to_string(),
            cred_def_id: "3".to_string(),
            rev_reg_id: Some("4".to_string()),
            cred_rev_id: Some("5".to_string()),
            revocation_interval: None,
            tails_file: None,
            timestamp: None,
        }];
        assert_eq!(build_schemas_json_prover(&credential_ids).unwrap_err().kind(), VcxErrorKind::InvalidSchema);
    }

    #[test]
    #[cfg(feature = "general_test")]
    fn test_find_schemas() {
        let _setup = SetupMocks::init();

        assert_eq!(build_schemas_json_prover(&Vec::new()).unwrap(), "{}".to_string());

        let cred1 = CredInfoProver {
            requested_attr: "height_1".to_string(),
            referent: LICENCE_CRED_ID.to_string(),
            schema_id: SCHEMA_ID.to_string(),
            cred_def_id: CRED_DEF_ID.to_string(),
            rev_reg_id: Some(REV_REG_ID.to_string()),
            cred_rev_id: Some(CRED_REV_ID.to_string()),
            revocation_interval: None,
            tails_file: None,
            timestamp: None,
        };
        let cred2 = CredInfoProver {
            requested_attr: "zip_2".to_string(),
            referent: ADDRESS_CRED_ID.to_string(),
            schema_id: ADDRESS_SCHEMA_ID.to_string(),
            cred_def_id: ADDRESS_CRED_DEF_ID.to_string(),
            rev_reg_id: Some(ADDRESS_REV_REG_ID.to_string()),
            cred_rev_id: Some(ADDRESS_CRED_REV_ID.to_string()),
            revocation_interval: None,
            tails_file: None,
            timestamp: None,
        };
        let creds = vec![cred1, cred2];

        let schemas = build_schemas_json_prover(&creds).unwrap();
        assert!(schemas.len() > 0);
        assert!(schemas.contains(r#""id":"2hoqvcwupRTUNkXn6ArYzs:2:test-licence:4.4.4","name":"test-licence""#));
    }

    #[test]
    #[cfg(feature = "general_test")]
    fn test_credential_def_identifiers() {
        let _setup = SetupDefaults::init();

        let cred1 = CredInfoProver {
            requested_attr: "height_1".to_string(),
            referent: LICENCE_CRED_ID.to_string(),
            schema_id: SCHEMA_ID.to_string(),
            cred_def_id: CRED_DEF_ID.to_string(),
            rev_reg_id: Some(REV_REG_ID.to_string()),
            cred_rev_id: Some(CRED_REV_ID.to_string()),
            revocation_interval: Some(NonRevokedInterval { from: Some(123), to: Some(456) }),
            tails_file: Some(get_temp_dir_path(TEST_TAILS_FILE).to_str().unwrap().to_string()),
            timestamp: None,
        };
        let cred2 = CredInfoProver {
            requested_attr: "zip_2".to_string(),
            referent: ADDRESS_CRED_ID.to_string(),
            schema_id: ADDRESS_SCHEMA_ID.to_string(),
            cred_def_id: ADDRESS_CRED_DEF_ID.to_string(),
            rev_reg_id: Some(ADDRESS_REV_REG_ID.to_string()),
            cred_rev_id: Some(ADDRESS_CRED_REV_ID.to_string()),
            revocation_interval: Some(NonRevokedInterval { from: None, to: Some(987) }),
            tails_file: None,
            timestamp: None,
        };
        let selected_credentials: Value = json!({
           "attrs":{
              "height_1":{
                "credential": {
                    "cred_info":{
                       "referent":LICENCE_CRED_ID,
                       "attrs":{
                          "sex":"male",
                          "age":"111",
                          "name":"Bob",
                          "height":"4'11"
                       },
                       "schema_id": SCHEMA_ID,
                       "cred_def_id": CRED_DEF_ID,
                       "rev_reg_id":REV_REG_ID,
                       "cred_rev_id":CRED_REV_ID
                    },
                    "interval":null
                },
                "tails_file": get_temp_dir_path(TEST_TAILS_FILE).to_str().unwrap().to_string(),
              },
              "zip_2":{
                "credential": {
                    "cred_info":{
                       "referent":ADDRESS_CRED_ID,
                       "attrs":{
                          "address1":"101 Tela Lane",
                          "address2":"101 Wilson Lane",
                          "zip":"87121",
                          "state":"UT",
                          "city":"SLC"
                       },
                       "schema_id":ADDRESS_SCHEMA_ID,
                       "cred_def_id":ADDRESS_CRED_DEF_ID,
                       "rev_reg_id":ADDRESS_REV_REG_ID,
                       "cred_rev_id":ADDRESS_CRED_REV_ID
                    },
                    "interval":null
                },
             }
           },
           "predicates":{ }
        });
        let proof_req = json!({
            "nonce": "123432421212",
            "name": "proof_req_1",
            "version": "0.1",
            "requested_attributes": {
                "zip_2": { "name": "zip" },
                "height_1": { "name": "height", "non_revoked": {"from": 123, "to": 456} }
            },
            "requested_predicates": {},
            "non_revoked": {"to": 987}
        }).to_string();

        let creds = credential_def_identifiers(&selected_credentials.to_string(), &serde_json::from_str(&proof_req).unwrap()).unwrap();
        assert_eq!(creds, vec![cred1, cred2]);
    }

    #[test]
    #[cfg(feature = "general_test")]
    fn test_credential_def_identifiers_failure() {
        let _setup = SetupDefaults::init();

        // selected credentials has incorrect json
        assert_eq!(credential_def_identifiers("", &proof_req_no_interval()).unwrap_err().kind(), VcxErrorKind::InvalidJson);


        // No Creds
        assert_eq!(credential_def_identifiers("{}", &proof_req_no_interval()).unwrap(), Vec::new());
        assert_eq!(credential_def_identifiers(r#"{"attrs":{}}"#, &proof_req_no_interval()).unwrap(), Vec::new());

        // missing cred info
        let selected_credentials: Value = json!({
           "attrs":{
              "height_1":{ "interval":null }
           },
           "predicates":{

           }
        });
        assert_eq!(credential_def_identifiers(&selected_credentials.to_string(), &proof_req_no_interval()).unwrap_err().kind(), VcxErrorKind::InvalidProofCredentialData);

        // Optional Revocation
        let mut selected_credentials: Value = json!({
           "attrs":{
              "height_1":{
                "credential": {
                    "cred_info":{
                       "referent":LICENCE_CRED_ID,
                       "attrs":{
                          "sex":"male",
                          "age":"111",
                          "name":"Bob",
                          "height":"4'11"
                       },
                       "schema_id": SCHEMA_ID,
                       "cred_def_id": CRED_DEF_ID,
                       "cred_rev_id":CRED_REV_ID
                    },
                    "interval":null
                },
                "tails_file": get_temp_dir_path(TEST_TAILS_FILE).to_str().unwrap().to_string(),
              },
           },
           "predicates":{ }
        });
        let creds = vec![CredInfoProver {
            requested_attr: "height_1".to_string(),
            referent: LICENCE_CRED_ID.to_string(),
            schema_id: SCHEMA_ID.to_string(),
            cred_def_id: CRED_DEF_ID.to_string(),
            rev_reg_id: None,
            cred_rev_id: Some(CRED_REV_ID.to_string()),
            revocation_interval: None,
            tails_file: Some(get_temp_dir_path(TEST_TAILS_FILE).to_str().unwrap().to_string()),
            timestamp: None,
        }];
        assert_eq!(&credential_def_identifiers(&selected_credentials.to_string(), &proof_req_no_interval()).unwrap(), &creds);

        // rev_reg_id is null
        selected_credentials["attrs"]["height_1"]["cred_info"]["rev_reg_id"] = serde_json::Value::Null;
        assert_eq!(&credential_def_identifiers(&selected_credentials.to_string(), &proof_req_no_interval()).unwrap(), &creds);

        // Missing schema ID
        let mut selected_credentials: Value = json!({
           "attrs":{
              "height_1":{
                "credential": {
                    "cred_info":{
                       "referent":LICENCE_CRED_ID,
                       "attrs":{
                          "sex":"male",
                          "age":"111",
                          "name":"Bob",
                          "height":"4'11"
                       },
                       "cred_def_id": CRED_DEF_ID,
                       "rev_reg_id":REV_REG_ID,
                       "cred_rev_id":CRED_REV_ID
                    },
                    "interval":null
                },
                "tails_file": get_temp_dir_path(TEST_TAILS_FILE).to_str().unwrap().to_string()
              },
           },
           "predicates":{ }
        });
        assert_eq!(credential_def_identifiers(&selected_credentials.to_string(), &proof_req_no_interval()).unwrap_err().kind(), VcxErrorKind::InvalidProofCredentialData);

        // Schema Id is null
        selected_credentials["attrs"]["height_1"]["cred_info"]["schema_id"] = serde_json::Value::Null;
        assert_eq!(credential_def_identifiers(&selected_credentials.to_string(), &proof_req_no_interval()).unwrap_err().kind(), VcxErrorKind::InvalidProofCredentialData);
    }

    #[test]
    #[cfg(feature = "general_test")]
    fn test_build_requested_credentials() {
        let _setup = SetupMocks::init();

        let cred1 = CredInfoProver {
            requested_attr: "height_1".to_string(),
            referent: LICENCE_CRED_ID.to_string(),
            schema_id: SCHEMA_ID.to_string(),
            cred_def_id: CRED_DEF_ID.to_string(),
            rev_reg_id: Some(REV_REG_ID.to_string()),
            cred_rev_id: Some(CRED_REV_ID.to_string()),
            revocation_interval: None,
            tails_file: None,
            timestamp: Some(800),
        };
        let cred2 = CredInfoProver {
            requested_attr: "zip_2".to_string(),
            referent: ADDRESS_CRED_ID.to_string(),
            schema_id: ADDRESS_SCHEMA_ID.to_string(),
            cred_def_id: ADDRESS_CRED_DEF_ID.to_string(),
            rev_reg_id: Some(ADDRESS_REV_REG_ID.to_string()),
            cred_rev_id: Some(ADDRESS_CRED_REV_ID.to_string()),
            revocation_interval: None,
            tails_file: None,
            timestamp: Some(800),
        };
        let creds = vec![cred1, cred2];
        let self_attested_attrs = json!({
            "self_attested_attr_3": "my self attested 1",
            "self_attested_attr_4": "my self attested 2",
        }).to_string();

        let test: Value = json!({
              "self_attested_attributes":{
                  "self_attested_attr_3": "my self attested 1",
                  "self_attested_attr_4": "my self attested 2",
              },
              "requested_attributes":{
                  "height_1": {"cred_id": LICENCE_CRED_ID, "revealed": true, "timestamp": 800},
                  "zip_2": {"cred_id": ADDRESS_CRED_ID, "revealed": true, "timestamp": 800},
              },
              "requested_predicates":{}
        });

        let proof_req = json!({
            "nonce": "123432421212",
            "name": "proof_req_1",
            "version": "0.1",
            "requested_attributes": {
                "height_1": {
                    "name": "height_1",
                    "non_revoked":  {"from": 123, "to": 456}
                },
                "zip_2": { "name": "zip_2" }
            },
            "requested_predicates": {},
            "non_revoked": {"from": 098, "to": 123}
        });
        let proof_req: ProofRequestData = serde_json::from_value(proof_req).unwrap();
        let requested_credential = build_requested_credentials_json(&creds, &self_attested_attrs, &proof_req).unwrap();
        assert_eq!(test.to_string(), requested_credential);
    }

    #[test]
    #[cfg(feature = "general_test")]
    fn test_build_rev_states_json() {
        let _setup = SetupMocks::init();

        let cred1 = CredInfoProver {
            requested_attr: "height".to_string(),
            referent: "abc".to_string(),
            schema_id: SCHEMA_ID.to_string(),
            cred_def_id: CRED_DEF_ID.to_string(),
            rev_reg_id: Some(REV_REG_ID.to_string()),
            cred_rev_id: Some(CRED_REV_ID.to_string()),
            tails_file: Some(get_temp_dir_path(TEST_TAILS_FILE).to_str().unwrap().to_string()),
            revocation_interval: None,
            timestamp: None,
        };
        let mut cred_info = vec![cred1];
        let states = build_rev_states_json(cred_info.as_mut()).unwrap();
        let rev_state_json: Value = serde_json::from_str(REV_STATE_JSON).unwrap();
        let expected = json!({REV_REG_ID: {"1": rev_state_json}}).to_string();
        assert_eq!(states, expected);
        assert!(cred_info[0].timestamp.is_some());
    }

    #[cfg(feature = "pool_tests")]
    #[test]
    fn test_build_rev_states_json_empty() {
        let _setup = SetupLibraryWalletPoolZeroFees::init();

        // empty vector
        assert_eq!(build_rev_states_json(Vec::new().as_mut()).unwrap(), "{}".to_string());

        // no rev_reg_id
        let cred1 = CredInfoProver {
            requested_attr: "height_1".to_string(),
            referent: LICENCE_CRED_ID.to_string(),
            schema_id: SCHEMA_ID.to_string(),
            cred_def_id: CRED_DEF_ID.to_string(),
            rev_reg_id: None,
            cred_rev_id: Some(CRED_REV_ID.to_string()),
            tails_file: Some(get_temp_dir_path(TEST_TAILS_FILE).to_str().unwrap().to_string()),
            revocation_interval: None,
            timestamp: None,
        };
        assert_eq!(build_rev_states_json(vec![cred1].as_mut()).unwrap(), "{}".to_string());
    }

    #[test]
    #[cfg(feature = "general_test")]
    fn test_get_credential_intervals_from_proof_req() {
        let _setup = SetupDefaults::init();

        let proof_req = json!({
            "nonce": "123432421212",
            "name": "proof_req_1",
            "version": "0.1",
            "requested_attributes": {
                "address1_1": {
                    "name": "address1",
                    "non_revoked":  {"from": 123, "to": 456}
                },
                "zip_2": { "name": "zip" }
            },
            "requested_predicates": {},
            "non_revoked": {"from": 098, "to": 123}
        });
        let proof_req: ProofRequestData = serde_json::from_value(proof_req).unwrap();

        // Attribute not found in proof req
        assert_eq!(_get_revocation_interval("not here", &proof_req).unwrap_err().kind(), VcxErrorKind::InvalidProofCredentialData);

        // attribute interval overrides proof request interval
        let interval = Some(NonRevokedInterval { from: Some(123), to: Some(456) });
        assert_eq!(_get_revocation_interval("address1_1", &proof_req).unwrap(), interval);

        // when attribute interval is None, defaults to proof req interval
        let interval = Some(NonRevokedInterval { from: Some(098), to: Some(123) });
        assert_eq!(_get_revocation_interval("zip_2", &proof_req).unwrap(), interval);

        // No interval provided for attribute or proof req
        assert_eq!(_get_revocation_interval("address1_1", &proof_req_no_interval()).unwrap(), None);
    }
}
