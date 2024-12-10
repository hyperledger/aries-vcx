use std::{collections::HashMap, path::Path};

use anoncreds_types::data_types::{
    identifiers::{cred_def_id::CredentialDefinitionId, schema_id::SchemaId},
    messages::{
        cred_selection::SelectedCredentials,
        pres_request::{NonRevokedInterval, PresentationRequest},
        presentation::{RequestedAttribute, RequestedCredentials, RequestedPredicate},
    },
};
use aries_vcx_anoncreds::anoncreds::base_anoncreds::{
    BaseAnonCreds, CredentialDefinitionsMap, RevocationStatesMap, SchemasMap,
};
use aries_vcx_ledger::ledger::base_ledger::AnoncredsLedgerRead;
use chrono::Utc;

use crate::errors::error::prelude::*;

// TODO: Move to anoncreds_types
#[derive(Debug, Deserialize, Serialize, PartialEq, Eq)]
pub struct CredInfoProver {
    pub referent: String,
    pub credential_referent: String,
    pub schema_id: SchemaId,
    pub cred_def_id: CredentialDefinitionId,
    pub rev_reg_id: Option<String>,
    pub cred_rev_id: Option<u32>,
    pub revocation_interval: Option<NonRevokedInterval>,
    pub tails_dir: Option<String>,
    pub timestamp: Option<u64>,
    pub revealed: Option<bool>,
}

pub async fn build_schemas_json_prover(
    ledger: &impl AnoncredsLedgerRead,
    credentials_identifiers: &Vec<CredInfoProver>,
) -> VcxResult<SchemasMap> {
    trace!(
        "build_schemas_json_prover >>> credentials_identifiers: {:?}",
        credentials_identifiers
    );
    let mut rtn: SchemasMap = HashMap::new();

    for cred_info in credentials_identifiers {
        if !rtn.contains_key(&cred_info.schema_id) {
            let schema_json = ledger
                .get_schema(&cred_info.schema_id, None)
                .await
                .map_err(|err| {
                    AriesVcxError::from_msg(
                        AriesVcxErrorKind::InvalidSchema,
                        format!("Cannot get schema id {}; {}", cred_info.schema_id, err),
                    )
                })?;

            rtn.insert(cred_info.schema_id.to_owned(), schema_json);
        }
    }
    Ok(rtn)
}

pub async fn build_cred_defs_json_prover(
    ledger: &impl AnoncredsLedgerRead,
    credentials_identifiers: &Vec<CredInfoProver>,
) -> VcxResult<CredentialDefinitionsMap> {
    trace!(
        "build_cred_defs_json_prover >>> credentials_identifiers: {:?}",
        credentials_identifiers
    );
    let mut rtn: CredentialDefinitionsMap = HashMap::new();

    for cred_info in credentials_identifiers {
        if !rtn.contains_key(&cred_info.cred_def_id) {
            let credential_def = ledger
                .get_cred_def(&cred_info.cred_def_id, None)
                .await
                .map_err(|_err| {
                    AriesVcxError::from_msg(
                        AriesVcxErrorKind::InvalidProofCredentialData,
                        "Cannot get credential definition",
                    )
                })?;

            rtn.insert(cred_info.cred_def_id.to_owned(), credential_def);
        }
    }
    Ok(rtn)
}

pub fn credential_def_identifiers(
    credentials: &SelectedCredentials,
    proof_req: &PresentationRequest,
) -> VcxResult<Vec<CredInfoProver>> {
    trace!(
        "credential_def_identifiers >>> credentials: {:?}, proof_req: {:?}",
        credentials,
        proof_req
    );
    let mut rtn = Vec::new();

    for (referent, selected_cred) in credentials.credential_for_referent.iter() {
        let cred_info = &selected_cred.credential.cred_info;
        rtn.push(CredInfoProver {
            referent: referent.clone(),
            credential_referent: cred_info.referent.clone(),
            schema_id: cred_info.schema_id.clone(),
            cred_def_id: cred_info.cred_def_id.clone(),
            revocation_interval: _get_revocation_interval(referent, proof_req)?,
            timestamp: None,
            rev_reg_id: cred_info.rev_reg_id.clone(),
            cred_rev_id: cred_info.cred_rev_id,
            tails_dir: selected_cred.tails_dir.clone(),
            revealed: cred_info.revealed,
        });
    }

    Ok(rtn)
}

fn _get_revocation_interval(
    attr_name: &str,
    proof_req: &PresentationRequest,
) -> VcxResult<Option<NonRevokedInterval>> {
    if let Some(attr) = proof_req.value().requested_attributes.get(attr_name) {
        Ok(attr
            .non_revoked
            .clone()
            .or(proof_req.value().non_revoked.clone().or(None)))
    } else if let Some(attr) = proof_req.value().requested_predicates.get(attr_name) {
        // Handle case for predicates
        Ok(attr
            .non_revoked
            .clone()
            .or(proof_req.value().non_revoked.clone().or(None)))
    } else {
        Err(AriesVcxError::from_msg(
            AriesVcxErrorKind::InvalidProofCredentialData,
            format!("Attribute not found for: {}", attr_name),
        ))
    }
}

pub async fn build_rev_states_json(
    ledger_read: &impl AnoncredsLedgerRead,
    anoncreds: &impl BaseAnonCreds,
    credentials_identifiers: &mut Vec<CredInfoProver>,
) -> VcxResult<RevocationStatesMap> {
    trace!(
        "build_rev_states_json >> credentials_identifiers: {:?}",
        credentials_identifiers
    );
    let mut rtn: RevocationStatesMap = HashMap::new();
    let mut timestamps: HashMap<String, u64> = HashMap::new();

    for cred_info in credentials_identifiers.iter_mut() {
        if let (Some(rev_reg_id), Some(cred_rev_id), Some(tails_dir)) = (
            &cred_info.rev_reg_id,
            &cred_info.cred_rev_id,
            &cred_info.tails_dir,
        ) {
            if !rtn.contains_key(rev_reg_id) {
                // Does this make sense in case cred_info's for same rev_reg_ids have different
                // revocation intervals
                let (_from, to) = if let Some(ref interval) = cred_info.revocation_interval {
                    (interval.from, interval.to)
                } else {
                    (None, None)
                };

                let parsed_id = &rev_reg_id.to_owned().try_into()?;
                let (rev_reg_def_json, meta) = ledger_read.get_rev_reg_def_json(parsed_id).await?;

                let on_or_before = to.unwrap_or(Utc::now().timestamp() as u64);
                let (rev_status_list, timestamp) = ledger_read
                    .get_rev_status_list(parsed_id, on_or_before, Some(&meta))
                    .await?;

                let rev_state_json = anoncreds
                    .create_revocation_state(
                        Path::new(tails_dir),
                        rev_reg_def_json,
                        rev_status_list,
                        *cred_rev_id,
                    )
                    .await?;

                // TODO: proover should be able to create multiple states of same revocation policy
                // for different timestamps see ticket IS-1108
                rtn.insert(
                    rev_reg_id.to_string(),
                    vec![(timestamp, rev_state_json)].into_iter().collect(),
                );
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

    Ok(rtn)
}

pub fn build_requested_credentials_json(
    credentials_identifiers: &Vec<CredInfoProver>,
    self_attested_attrs: HashMap<String, String>,
    proof_req: &PresentationRequest,
) -> VcxResult<RequestedCredentials> {
    trace!(
        "build_requested_credentials_json >> credentials_identifiers: {:?}, self_attested_attrs: \
         {:?}, proof_req: {:?}",
        credentials_identifiers,
        self_attested_attrs,
        proof_req
    );

    let mut rtn = RequestedCredentials::default();

    for cred_info in credentials_identifiers {
        if proof_req
            .value()
            .requested_attributes
            .contains_key(&cred_info.referent)
        {
            rtn.requested_attributes.insert(
                cred_info.referent.to_owned(),
                RequestedAttribute {
                    cred_id: cred_info.credential_referent.to_owned(),
                    timestamp: cred_info.timestamp,
                    revealed: cred_info.revealed.unwrap_or(true),
                },
            );
        }
    }

    for cred_info in credentials_identifiers {
        if proof_req
            .value()
            .requested_predicates
            .contains_key(&cred_info.referent)
        {
            rtn.requested_predicates.insert(
                cred_info.referent.to_owned(),
                RequestedPredicate {
                    cred_id: cred_info.credential_referent.to_owned(),
                    timestamp: cred_info.timestamp,
                },
            );
        }
    }

    // handle if the attribute is not revealed
    rtn.self_attested_attributes = self_attested_attrs;

    Ok(rtn)
}

#[cfg(test)]
pub mod pool_tests {
    use std::error::Error;

    use aries_vcx_ledger::ledger::indy::pool::test_utils::get_temp_dir_path;
    use test_utils::{
        constants::{cred_def_id, schema_id, CRED_REV_ID, LICENCE_CRED_ID},
        devsetup::build_setup_profile,
    };

    use super::*;
    use crate::common::proofs::prover::prover_internal::{build_rev_states_json, CredInfoProver};

    #[tokio::test]
    #[ignore]
    async fn test_pool_build_rev_states_json_empty() -> Result<(), Box<dyn Error>> {
        let setup = build_setup_profile().await;
        let mut empty_credential_identifiers = Vec::new();
        assert_eq!(
            build_rev_states_json(
                &setup.ledger_read,
                &setup.anoncreds,
                empty_credential_identifiers.as_mut()
            )
            .await?,
            HashMap::new()
        );

        // no rev_reg_id
        let cred1 = CredInfoProver {
            referent: "height_1".to_string(),
            credential_referent: LICENCE_CRED_ID.to_string(),
            schema_id: schema_id(),
            cred_def_id: cred_def_id(),
            rev_reg_id: None,
            cred_rev_id: Some(CRED_REV_ID),
            tails_dir: Some(get_temp_dir_path().to_str().unwrap().to_string()),
            revocation_interval: None,
            timestamp: None,
            revealed: None,
        };
        assert_eq!(
            build_rev_states_json(&setup.ledger_read, &setup.anoncreds, vec![cred1].as_mut())
                .await?,
            HashMap::new()
        );
        Ok(())
    }
}

#[cfg(test)]
pub mod unit_tests {
    use aries_vcx_ledger::ledger::indy::pool::test_utils::get_temp_dir_path;
    use serde_json::Value;
    use test_utils::{
        constants::{
            address_cred_def_id, address_schema_id, cred_def_id, schema_id, ADDRESS_CRED_DEF_ID,
            ADDRESS_CRED_ID, ADDRESS_REV_REG_ID, ADDRESS_SCHEMA_ID, CRED_DEF_ID, CRED_REV_ID,
            LICENCE_CRED_ID, REV_REG_ID, REV_STATE_JSON, SCHEMA_ID,
        },
        devsetup::*,
        mockdata::{mock_anoncreds::MockAnoncreds, mock_ledger::MockLedger},
    };

    use super::*;

    fn proof_req_no_interval() -> PresentationRequest {
        let proof_req = json!({
            "nonce": "123432421212",
            "name": "proof_req_1",
            "version": "1.0",
            "requested_attributes": {
                "address1_1": { "name": "address1" },
                "zip_2": { "name": "zip" },
                "height_1": { "name": "height" }
            },
            "requested_predicates": {},
        })
        .to_string();

        serde_json::from_str(&proof_req).unwrap()
    }

    #[tokio::test]
    async fn test_find_credential_def() {
        let _setup = SetupMocks::init();

        let cred1 = CredInfoProver {
            referent: "height_1".to_string(),
            credential_referent: LICENCE_CRED_ID.to_string(),
            schema_id: schema_id(),
            cred_def_id: cred_def_id(),
            rev_reg_id: Some(REV_REG_ID.to_string()),
            cred_rev_id: Some(CRED_REV_ID),
            revocation_interval: None,
            tails_dir: None,
            timestamp: None,
            revealed: None,
        };
        let cred2 = CredInfoProver {
            referent: "zip_2".to_string(),
            credential_referent: ADDRESS_CRED_ID.to_string(),
            schema_id: address_schema_id(),
            cred_def_id: address_cred_def_id(),
            rev_reg_id: Some(ADDRESS_REV_REG_ID.to_string()),
            cred_rev_id: Some(CRED_REV_ID),
            revocation_interval: None,
            tails_dir: None,
            timestamp: None,
            revealed: None,
        };
        let creds = vec![cred1, cred2];

        let ledger_read = MockLedger;
        let credential_def = build_cred_defs_json_prover(&ledger_read, &creds)
            .await
            .unwrap();
        assert!(!credential_def.is_empty());
        assert_eq!(
            credential_def.get(&cred_def_id()).unwrap().schema_id,
            SchemaId::new_unchecked("47")
        );
        assert_eq!(
            credential_def.get(&cred_def_id()).unwrap().id,
            cred_def_id()
        );
    }

    #[tokio::test]
    async fn test_find_schemas() {
        let _setup = SetupMocks::init();

        let ledger_read = MockLedger;
        assert_eq!(
            build_schemas_json_prover(&ledger_read, &Vec::new())
                .await
                .unwrap(),
            Default::default()
        );

        let cred1 = CredInfoProver {
            referent: "height_1".to_string(),
            credential_referent: LICENCE_CRED_ID.to_string(),
            schema_id: schema_id(),
            cred_def_id: cred_def_id(),
            rev_reg_id: Some(REV_REG_ID.to_string()),
            cred_rev_id: Some(CRED_REV_ID),
            revocation_interval: None,
            tails_dir: None,
            timestamp: None,
            revealed: None,
        };
        let cred2 = CredInfoProver {
            referent: "zip_2".to_string(),
            credential_referent: ADDRESS_CRED_ID.to_string(),
            schema_id: address_schema_id(),
            cred_def_id: address_cred_def_id(),
            rev_reg_id: Some(ADDRESS_REV_REG_ID.to_string()),
            cred_rev_id: Some(CRED_REV_ID),
            revocation_interval: None,
            tails_dir: None,
            timestamp: None,
            revealed: None,
        };
        let creds = vec![cred1, cred2];

        let ledger_read = MockLedger;
        let schemas = build_schemas_json_prover(&ledger_read, &creds)
            .await
            .unwrap();
        assert!(!schemas.is_empty());
        assert_eq!(
            schemas.get(&schema_id()).unwrap().name,
            "test-licence".to_string()
        );
    }

    #[test]
    fn test_credential_def_identifiers() {
        let _setup = SetupMocks::init();

        let cred1 = CredInfoProver {
            referent: "height_1".to_string(),
            credential_referent: LICENCE_CRED_ID.to_string(),
            schema_id: schema_id(),
            cred_def_id: cred_def_id(),
            rev_reg_id: Some(REV_REG_ID.to_string()),
            cred_rev_id: Some(CRED_REV_ID),
            revocation_interval: Some(NonRevokedInterval {
                from: Some(123),
                to: Some(456),
            }),
            tails_dir: Some(get_temp_dir_path().to_str().unwrap().to_string()),
            timestamp: None,
            revealed: None,
        };
        let cred2 = CredInfoProver {
            referent: "zip_2".to_string(),
            credential_referent: ADDRESS_CRED_ID.to_string(),
            schema_id: address_schema_id(),
            cred_def_id: address_cred_def_id(),
            rev_reg_id: Some(ADDRESS_REV_REG_ID.to_string()),
            cred_rev_id: Some(CRED_REV_ID),
            revocation_interval: Some(NonRevokedInterval {
                from: None,
                to: Some(987),
            }),
            tails_dir: None,
            timestamp: None,
            revealed: None,
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
                "tails_dir": get_temp_dir_path().to_str().unwrap().to_string(),
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
                       "cred_rev_id":CRED_REV_ID
                    },
                    "interval":null
                },
             }
           }
        });
        let proof_req = json!({
            "nonce": "123432421212",
            "name": "proof_req_1",
            "version": "1.0",
            "requested_attributes": {
                "zip_2": { "name": "zip" },
                "height_1": { "name": "height", "non_revoked": {"from": 123, "to": 456} }
            },
            "requested_predicates": {},
            "non_revoked": {"to": 987}
        })
        .to_string();

        let creds = credential_def_identifiers(
            &serde_json::from_value(selected_credentials).unwrap(),
            &serde_json::from_str(&proof_req).unwrap(),
        )
        .unwrap();
        assert_eq!(creds.len(), 2);
        assert!(creds.contains(&cred1));
        assert!(creds.contains(&cred2));
    }

    #[test]
    fn test_credential_def_identifiers_failure() {
        let _setup = SetupMocks::init();

        // No Creds
        assert_eq!(
            credential_def_identifiers(
                &serde_json::from_str("{}").unwrap(),
                &proof_req_no_interval(),
            )
            .unwrap(),
            Vec::new()
        );
        assert_eq!(
            credential_def_identifiers(
                &serde_json::from_str(r#"{"attrs":{}}"#).unwrap(),
                &proof_req_no_interval(),
            )
            .unwrap(),
            Vec::new()
        );

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
                "tails_dir": get_temp_dir_path().to_str().unwrap().to_string(),
              },
           }
        });
        let creds = vec![CredInfoProver {
            referent: "height_1".to_string(),
            credential_referent: LICENCE_CRED_ID.to_string(),
            schema_id: schema_id(),
            cred_def_id: cred_def_id(),
            rev_reg_id: None,
            cred_rev_id: Some(CRED_REV_ID),
            revocation_interval: None,
            tails_dir: Some(get_temp_dir_path().to_str().unwrap().to_string()),
            timestamp: None,
            revealed: None,
        }];
        assert_eq!(
            &credential_def_identifiers(
                &serde_json::from_value(selected_credentials.clone()).unwrap(),
                &proof_req_no_interval(),
            )
            .unwrap(),
            &creds
        );

        // rev_reg_id is null
        selected_credentials["attrs"]["height_1"]["cred_info"]["rev_reg_id"] =
            serde_json::Value::Null;
        assert_eq!(
            &credential_def_identifiers(
                &serde_json::from_value(selected_credentials).unwrap(),
                &proof_req_no_interval(),
            )
            .unwrap(),
            &creds
        );
    }

    #[test]
    fn test_build_requested_credentials() {
        let _setup = SetupMocks::init();

        let cred1 = CredInfoProver {
            referent: "height_1".to_string(),
            credential_referent: LICENCE_CRED_ID.to_string(),
            schema_id: schema_id(),
            cred_def_id: cred_def_id(),
            rev_reg_id: Some(REV_REG_ID.to_string()),
            cred_rev_id: Some(CRED_REV_ID),
            revocation_interval: None,
            tails_dir: None,
            timestamp: Some(800),
            revealed: None,
        };
        let cred2 = CredInfoProver {
            referent: "zip_2".to_string(),
            credential_referent: ADDRESS_CRED_ID.to_string(),
            schema_id: address_schema_id(),
            cred_def_id: address_cred_def_id(),
            rev_reg_id: Some(ADDRESS_REV_REG_ID.to_string()),
            cred_rev_id: Some(CRED_REV_ID),
            revocation_interval: None,
            tails_dir: None,
            timestamp: Some(800),
            revealed: Some(false),
        };
        let creds = vec![cred1, cred2];
        let self_attested_attrs = json!({
            "self_attested_attr_3": "my self attested 1",
            "self_attested_attr_4": "my self attested 2",
        })
        .to_string();

        let test: Value = json!({
              "self_attested_attributes":{
                  "self_attested_attr_3": "my self attested 1",
                  "self_attested_attr_4": "my self attested 2",
              },
              "requested_attributes":{
                  "height_1": {"cred_id": LICENCE_CRED_ID, "revealed": true, "timestamp": 800},
                  "zip_2": {"cred_id": ADDRESS_CRED_ID, "revealed": false, "timestamp": 800},
              },
              "requested_predicates":{}
        });

        let proof_req = json!({
            "nonce": "123432421212",
            "name": "proof_req_1",
            "version": "1.0",
            "requested_attributes": {
                "height_1": {
                    "name": "height_1",
                    "non_revoked":  {"from": 123, "to": 456}
                },
                "zip_2": { "name": "zip_2" }
            },
            "requested_predicates": {},
            "non_revoked": {"from": 98, "to": 123}
        });
        let proof_req: PresentationRequest = serde_json::from_value(proof_req).unwrap();
        let requested_credential = build_requested_credentials_json(
            &creds,
            serde_json::from_str(&self_attested_attrs).unwrap(),
            &proof_req,
        )
        .unwrap();
        assert_eq!(test, serde_json::to_value(requested_credential).unwrap());
    }

    #[tokio::test]
    async fn test_build_rev_states_json() {
        let _setup = SetupMocks::init();

        let cred1 = CredInfoProver {
            referent: "height".to_string(),
            credential_referent: "abc".to_string(),
            schema_id: schema_id(),
            cred_def_id: cred_def_id(),
            rev_reg_id: Some(REV_REG_ID.to_string()),
            cred_rev_id: Some(CRED_REV_ID),
            tails_dir: Some(get_temp_dir_path().to_str().unwrap().to_string()),
            revocation_interval: None,
            timestamp: None,
            revealed: None,
        };
        let mut cred_info = vec![cred1];
        let anoncreds = MockAnoncreds;
        let ledger_read = MockLedger;
        let states = build_rev_states_json(&ledger_read, &anoncreds, cred_info.as_mut())
            .await
            .unwrap();
        let expected: RevocationStatesMap = vec![(
            REV_REG_ID.to_string(),
            vec![(1, serde_json::from_str(REV_STATE_JSON).unwrap())]
                .into_iter()
                .collect(),
        )]
        .into_iter()
        .collect();
        assert_eq!(states, expected);
        assert!(cred_info[0].timestamp.is_some());
    }

    #[test]
    fn test_get_credential_intervals_from_proof_req() {
        let _setup = SetupMocks::init();

        let proof_req = json!({
            "nonce": "123432421212",
            "name": "proof_req_1",
            "version": "1.0",
            "requested_attributes": {
                "address1_1": {
                    "name": "address1",
                    "non_revoked":  {"from": 123, "to": 456}
                },
                "zip_2": { "name": "zip" }
            },
            "requested_predicates": {},
            "non_revoked": {"from": 98, "to": 123}
        });
        let proof_req: PresentationRequest = serde_json::from_value(proof_req).unwrap();

        // Attribute not found in proof req
        assert_eq!(
            _get_revocation_interval("not here", &proof_req)
                .unwrap_err()
                .kind(),
            AriesVcxErrorKind::InvalidProofCredentialData
        );

        // attribute interval overrides proof request interval
        let interval = Some(NonRevokedInterval {
            from: Some(123),
            to: Some(456),
        });
        assert_eq!(
            _get_revocation_interval("address1_1", &proof_req).unwrap(),
            interval
        );

        // when attribute interval is None, defaults to proof req interval
        let interval = Some(NonRevokedInterval {
            from: Some(98),
            to: Some(123),
        });
        assert_eq!(
            _get_revocation_interval("zip_2", &proof_req).unwrap(),
            interval
        );

        // No interval provided for attribute or proof req
        assert_eq!(
            _get_revocation_interval("address1_1", &proof_req_no_interval()).unwrap(),
            None
        );
    }
}
