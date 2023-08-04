use std::{collections::HashMap, sync::Arc};

use aries_vcx_core::anoncreds::base_anoncreds::BaseAnonCreds;
use aries_vcx_core::errors::error::{AriesVcxCoreError, AriesVcxCoreErrorKind};
use aries_vcx_core::ledger::base_ledger::AnoncredsLedgerRead;
use serde_json::Value;

use crate::common::proofs::{proof_request::ProofRequestData, proof_request_internal::NonRevokedInterval};
use crate::errors::error::prelude::*;
use crate::handlers::proof_presentation::types::SelectedCredentials;

#[derive(Debug, Deserialize, Serialize, PartialEq, Eq)]
pub struct CredInfoProver {
    pub referent: String,
    pub credential_referent: String,
    pub schema_id: String,
    pub cred_def_id: String,
    pub rev_reg_id: Option<String>,
    pub cred_rev_id: Option<String>,
    pub revocation_interval: Option<NonRevokedInterval>,
    pub tails_dir: Option<String>,
    pub timestamp: Option<u64>,
    pub revealed: Option<bool>,
}

pub async fn build_schemas_json_prover(
    ledger: &Arc<dyn AnoncredsLedgerRead>,
    credentials_identifiers: &Vec<CredInfoProver>,
) -> VcxResult<String> {
    trace!(
        "build_schemas_json_prover >>> credentials_identifiers: {:?}",
        credentials_identifiers
    );
    let mut rtn: Value = json!({});

    for cred_info in credentials_identifiers {
        if rtn.get(&cred_info.schema_id).is_none() {
            let schema_json = ledger
                .get_schema(&cred_info.schema_id, None)
                .await
                .map_err(|err| err.map(AriesVcxCoreErrorKind::InvalidSchema, "Cannot get schema"))?;

            let schema_json = serde_json::from_str(&schema_json).map_err(|err| {
                AriesVcxCoreError::from_msg(
                    AriesVcxCoreErrorKind::InvalidSchema,
                    format!("Cannot deserialize schema: {}", err),
                )
            })?;

            rtn[cred_info.schema_id.to_owned()] = schema_json;
        }
    }
    Ok(rtn.to_string())
}

pub async fn build_cred_defs_json_prover(
    ledger: &Arc<dyn AnoncredsLedgerRead>,
    credentials_identifiers: &Vec<CredInfoProver>,
) -> VcxResult<String> {
    trace!(
        "build_cred_defs_json_prover >>> credentials_identifiers: {:?}",
        credentials_identifiers
    );
    let mut rtn: Value = json!({});

    for cred_info in credentials_identifiers {
        if rtn.get(&cred_info.cred_def_id).is_none() {
            let credential_def = ledger.get_cred_def(&cred_info.cred_def_id, None).await.map_err(|err| {
                err.map(
                    AriesVcxCoreErrorKind::InvalidProofCredentialData,
                    "Cannot get credential definition",
                )
            })?;

            let credential_def = serde_json::from_str(&credential_def).map_err(|err| {
                AriesVcxError::from_msg(
                    AriesVcxErrorKind::InvalidProofCredentialData,
                    format!("Cannot deserialize credential definition: {}", err),
                )
            })?;

            rtn[cred_info.cred_def_id.to_owned()] = credential_def;
        }
    }
    Ok(rtn.to_string())
}

// TODO - new name for method
/// Given the `credentials` selected for a given `proof_req`, construct `CredInfoProver` structures
/// which represent the details of how we should present a referent, including details of:
/// * which credential to use (identified by credential_referent),
/// * revocation interval to present,
/// * revocation timestamp to use in NRP of this referent,
/// * where the referent should be revealed (only relevant if attr presentation),
/// * other credential details
pub fn credential_def_identifiers(
    credentials: &SelectedCredentials,
    proof_req: &ProofRequestData,
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
            revocation_interval: find_revocation_interval(&referent, proof_req)?,
            timestamp: None, // populated later if required
            rev_reg_id: cred_info.rev_reg_id.clone(),
            cred_rev_id: cred_info.cred_rev_id.clone(),
            tails_dir: selected_cred.tails_dir.clone(),
            revealed: cred_info.revealed,
        });
    }

    Ok(rtn)
}

fn find_revocation_interval(referent: &str, proof_req: &ProofRequestData) -> VcxResult<Option<NonRevokedInterval>> {
    if let Some(attr_info) = proof_req.requested_attributes.get(referent) {
        Ok(attr_info.non_revoked.clone().or(proof_req.non_revoked.clone().or(None)))
    } else if let Some(pred_info) = proof_req.requested_predicates.get(referent) {
        // Handle case for predicates
        Ok(pred_info.non_revoked.clone().or(proof_req.non_revoked.clone().or(None)))
    } else {
        Err(AriesVcxError::from_msg(
            AriesVcxErrorKind::InvalidProofCredentialData,
            format!("Details of referent not found: {}", referent),
        ))
    }
}

pub async fn build_rev_states_json(
    ledger_read: &Arc<dyn AnoncredsLedgerRead>,
    anoncreds: &Arc<dyn BaseAnonCreds>,
    credentials_identifiers: &mut Vec<CredInfoProver>,
) -> VcxResult<HashMap<String, HashMap<String, Value>>> {
    trace!(
        "build_rev_states_json >> credentials_identifiers: {:?}",
        credentials_identifiers
    );
    // TODO - confirm that timestamp is a string rather than num - not sure what consumers expect..
    // TODO - ACCORDING TO INDY IMPL, IT ALSO UNDERSTANDS IT IF WE PROVIDE rev_states_by_timestamp_by_CRED_ID
    // can we use this? - we would use this if it is the case that we cannot use the same rev_state for 2 diff creds from the same rev-reg-id
    let mut rev_states_by_timestamp_by_rev_reg_id: HashMap<String, HashMap<String, Value>> = HashMap::new();

    for cred_info in credentials_identifiers.iter_mut() {
        let interval = if let Some(inner) = &cred_info.revocation_interval {
            inner
        } else {
            continue;
        };

        let (rev_reg_id, cred_rev_id, tails_dir) =
            match (&cred_info.rev_reg_id, &cred_info.cred_rev_id, &cred_info.tails_dir) {
                (Some(rev_reg_id), Some(cred_rev_id), Some(tails_dir)) => (rev_reg_id, cred_rev_id, tails_dir),
                (None, None, _) => {
                    // interval requested, but choosen credential is non-revocable. Verifier should accept this cred without a NRP
                    continue;
                }
                (Some(_), _, _) => {
                    // TODO - warning or error? interval requested AND credential is revocable. I believe the verifier will fail in this case if no NRP is given
                    todo!()
                }
                // TODO - other permutations, warning or error
                _ => todo!(),
            };

        let rev_reg_def_json = ledger_read.get_rev_reg_def_json(rev_reg_id).await?;

        // TODO - justify None for from
        let (rev_reg_id, rev_reg_delta_json, timestamp) = ledger_read
            .get_rev_reg_delta_json(rev_reg_id, None, interval.to)
            .await?;

        let rev_state_json = anoncreds
            .create_revocation_state(
                tails_dir,
                &rev_reg_def_json,
                &rev_reg_delta_json,
                timestamp,
                cred_rev_id,
            )
            .await?;

        let rev_state_json: Value = serde_json::from_str(&rev_state_json).map_err(|err| {
            AriesVcxError::from_msg(
                AriesVcxErrorKind::InvalidJson,
                format!("Cannot deserialize RevocationState: {}", err),
            )
        })?;

        if let Some(rev_states_by_timestamp) = rev_states_by_timestamp_by_rev_reg_id.get_mut(&rev_reg_id) {
            rev_states_by_timestamp.insert(timestamp.to_string(), rev_state_json);
        } else {
            let init_entry = (timestamp.to_string(), rev_state_json);
            rev_states_by_timestamp_by_rev_reg_id.insert(rev_reg_id, HashMap::from([init_entry]));
        }
    }

    Ok(rev_states_by_timestamp_by_rev_reg_id)
}

pub fn build_requested_credentials_json(
    credentials_identifiers: &Vec<CredInfoProver>,
    self_attested_attrs: &HashMap<String, String>,
    proof_req: &ProofRequestData,
) -> VcxResult<String> {
    trace!(
        "build_requested_credentials_json >> credentials_identifiers: {:?}, self_attested_attrs: {:?}, proof_req: {:?}",
        credentials_identifiers,
        self_attested_attrs,
        proof_req
    );
    let mut rtn: Value = json!({
          "self_attested_attributes":{},
          "requested_attributes":{},
          "requested_predicates":{}
    });
    // do same for predicates and self_attested
    if let Value::Object(ref mut map) = rtn["requested_attributes"] {
        for cred_info in credentials_identifiers {
            if proof_req.requested_attributes.get(&cred_info.referent).is_some() {
                let insert_val = json!({"cred_id": cred_info.credential_referent, "revealed": cred_info.revealed.unwrap_or(true), "timestamp": cred_info.timestamp});
                map.insert(cred_info.referent.to_owned(), insert_val);
            }
        }
    }

    if let Value::Object(ref mut map) = rtn["requested_predicates"] {
        for cred_info in credentials_identifiers {
            if proof_req.requested_predicates.get(&cred_info.referent).is_some() {
                let insert_val = json!({"cred_id": cred_info.credential_referent, "timestamp": cred_info.timestamp});
                map.insert(cred_info.referent.to_owned(), insert_val);
            }
        }
    }

    // handle if the attribute is not revealed
    let self_attested_attrs: Value = serde_json::to_value(self_attested_attrs).map_err(|err| {
        AriesVcxError::from_msg(
            AriesVcxErrorKind::InvalidJson,
            format!("Cannot deserialize self attested attributes: {}", err),
        )
    })?;
    rtn["self_attested_attributes"] = self_attested_attrs;

    Ok(rtn.to_string())
}

#[cfg(test)]
#[allow(clippy::unwrap_used)]
pub mod pool_tests {
    use std::collections::HashMap;

    use crate::common::proofs::prover::prover_internal::{build_rev_states_json, CredInfoProver};
    use crate::utils::constants::{CRED_DEF_ID, CRED_REV_ID, LICENCE_CRED_ID, SCHEMA_ID};
    use crate::utils::devsetup::SetupProfile;
    use aries_vcx_core::ledger::indy::pool::test_utils::get_temp_dir_path;

    #[tokio::test]
    #[ignore]
    async fn test_pool_build_rev_states_json_empty() {
        SetupProfile::run(|setup| async move {
            // empty vector
            assert_eq!(
                build_rev_states_json(
                    &setup.profile.inject_anoncreds_ledger_read(),
                    &setup.profile.inject_anoncreds(),
                    Vec::new().as_mut()
                )
                .await
                .unwrap(),
                HashMap::new()
            );

            // no rev_reg_id
            let cred1 = CredInfoProver {
                referent: "height_1".to_string(),
                credential_referent: LICENCE_CRED_ID.to_string(),
                schema_id: SCHEMA_ID.to_string(),
                cred_def_id: CRED_DEF_ID.to_string(),
                rev_reg_id: None,
                cred_rev_id: Some(CRED_REV_ID.to_string()),
                tails_dir: Some(get_temp_dir_path().to_str().unwrap().to_string()),
                revocation_interval: None,
                timestamp: None,
                revealed: None,
            };
            assert_eq!(
                build_rev_states_json(
                    &setup.profile.inject_anoncreds_ledger_read(),
                    &setup.profile.inject_anoncreds(),
                    vec![cred1].as_mut()
                )
                .await
                .unwrap(),
                HashMap::new()
            );
        })
        .await;
    }
}

#[cfg(test)]
#[allow(clippy::unwrap_used)]
pub mod unit_tests {
    use crate::utils::constants::{
        ADDRESS_CRED_DEF_ID, ADDRESS_CRED_ID, ADDRESS_CRED_REV_ID, ADDRESS_REV_REG_ID, ADDRESS_SCHEMA_ID, CRED_DEF_ID,
        CRED_REV_ID, LICENCE_CRED_ID, REV_REG_ID, REV_STATE_JSON, SCHEMA_ID,
    };
    use crate::utils::devsetup::*;
    use crate::utils::mockdata::profile::mock_anoncreds::MockAnoncreds;
    use crate::utils::mockdata::profile::mock_ledger::MockLedger;
    use aries_vcx_core::ledger::indy::pool::test_utils::get_temp_dir_path;

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
            schema_id: SCHEMA_ID.to_string(),
            cred_def_id: CRED_DEF_ID.to_string(),
            rev_reg_id: Some(REV_REG_ID.to_string()),
            cred_rev_id: Some(CRED_REV_ID.to_string()),
            revocation_interval: None,
            tails_dir: None,
            timestamp: None,
            revealed: None,
        };
        let cred2 = CredInfoProver {
            referent: "zip_2".to_string(),
            credential_referent: ADDRESS_CRED_ID.to_string(),
            schema_id: ADDRESS_SCHEMA_ID.to_string(),
            cred_def_id: ADDRESS_CRED_DEF_ID.to_string(),
            rev_reg_id: Some(ADDRESS_REV_REG_ID.to_string()),
            cred_rev_id: Some(ADDRESS_CRED_REV_ID.to_string()),
            revocation_interval: None,
            tails_dir: None,
            timestamp: None,
            revealed: None,
        };
        let creds = vec![cred1, cred2];

        let ledger_read: Arc<dyn AnoncredsLedgerRead> = Arc::new(MockLedger {});
        let credential_def = build_cred_defs_json_prover(&ledger_read, &creds).await.unwrap();
        assert!(credential_def.len() > 0);
        assert!(credential_def.contains(r#""id":"V4SGRU86Z58d6TV7PBUe6f:3:CL:47:tag1","schemaId":"47""#));
    }

    #[tokio::test]
    async fn test_find_schemas() {
        let _setup = SetupMocks::init();

        let ledger_read: Arc<dyn AnoncredsLedgerRead> = Arc::new(MockLedger {});
        assert_eq!(
            build_schemas_json_prover(&ledger_read, &Vec::new()).await.unwrap(),
            "{}".to_string()
        );

        let cred1 = CredInfoProver {
            referent: "height_1".to_string(),
            credential_referent: LICENCE_CRED_ID.to_string(),
            schema_id: SCHEMA_ID.to_string(),
            cred_def_id: CRED_DEF_ID.to_string(),
            rev_reg_id: Some(REV_REG_ID.to_string()),
            cred_rev_id: Some(CRED_REV_ID.to_string()),
            revocation_interval: None,
            tails_dir: None,
            timestamp: None,
            revealed: None,
        };
        let cred2 = CredInfoProver {
            referent: "zip_2".to_string(),
            credential_referent: ADDRESS_CRED_ID.to_string(),
            schema_id: ADDRESS_SCHEMA_ID.to_string(),
            cred_def_id: ADDRESS_CRED_DEF_ID.to_string(),
            rev_reg_id: Some(ADDRESS_REV_REG_ID.to_string()),
            cred_rev_id: Some(ADDRESS_CRED_REV_ID.to_string()),
            revocation_interval: None,
            tails_dir: None,
            timestamp: None,
            revealed: None,
        };
        let creds = vec![cred1, cred2];

        let ledger_read: Arc<dyn AnoncredsLedgerRead> = Arc::new(MockLedger {});
        let schemas = build_schemas_json_prover(&ledger_read, &creds).await.unwrap();
        assert!(schemas.len() > 0);
        assert!(schemas.contains(r#""id":"2hoqvcwupRTUNkXn6ArYzs:2:test-licence:4.4.4","name":"test-licence""#));
    }

    #[test]
    fn test_credential_def_identifiers() {
        let _setup = SetupDefaults::init();

        let cred1 = CredInfoProver {
            referent: "height_1".to_string(),
            credential_referent: LICENCE_CRED_ID.to_string(),
            schema_id: SCHEMA_ID.to_string(),
            cred_def_id: CRED_DEF_ID.to_string(),
            rev_reg_id: Some(REV_REG_ID.to_string()),
            cred_rev_id: Some(CRED_REV_ID.to_string()),
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
            schema_id: ADDRESS_SCHEMA_ID.to_string(),
            cred_def_id: ADDRESS_CRED_DEF_ID.to_string(),
            rev_reg_id: Some(ADDRESS_REV_REG_ID.to_string()),
            cred_rev_id: Some(ADDRESS_CRED_REV_ID.to_string()),
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
                       "cred_rev_id":ADDRESS_CRED_REV_ID
                    },
                    "interval":null
                },
             }
           }
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
        let _setup = SetupDefaults::init();

        // No Creds
        assert_eq!(
            credential_def_identifiers(&serde_json::from_str("{}").unwrap(), &proof_req_no_interval()).unwrap(),
            Vec::new()
        );
        assert_eq!(
            credential_def_identifiers(
                &serde_json::from_str(r#"{"attrs":{}}"#).unwrap(),
                &proof_req_no_interval()
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
            schema_id: SCHEMA_ID.to_string(),
            cred_def_id: CRED_DEF_ID.to_string(),
            rev_reg_id: None,
            cred_rev_id: Some(CRED_REV_ID.to_string()),
            revocation_interval: None,
            tails_dir: Some(get_temp_dir_path().to_str().unwrap().to_string()),
            timestamp: None,
            revealed: None,
        }];
        assert_eq!(
            &credential_def_identifiers(
                &serde_json::from_value(selected_credentials.clone()).unwrap(),
                &proof_req_no_interval()
            )
            .unwrap(),
            &creds
        );

        // rev_reg_id is null
        selected_credentials["attrs"]["height_1"]["cred_info"]["rev_reg_id"] = serde_json::Value::Null;
        assert_eq!(
            &credential_def_identifiers(
                &serde_json::from_value(selected_credentials).unwrap(),
                &proof_req_no_interval()
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
            schema_id: SCHEMA_ID.to_string(),
            cred_def_id: CRED_DEF_ID.to_string(),
            rev_reg_id: Some(REV_REG_ID.to_string()),
            cred_rev_id: Some(CRED_REV_ID.to_string()),
            revocation_interval: None,
            tails_dir: None,
            timestamp: Some(800),
            revealed: None,
        };
        let cred2 = CredInfoProver {
            referent: "zip_2".to_string(),
            credential_referent: ADDRESS_CRED_ID.to_string(),
            schema_id: ADDRESS_SCHEMA_ID.to_string(),
            cred_def_id: ADDRESS_CRED_DEF_ID.to_string(),
            rev_reg_id: Some(ADDRESS_REV_REG_ID.to_string()),
            cred_rev_id: Some(ADDRESS_CRED_REV_ID.to_string()),
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
        let requested_credential =
            build_requested_credentials_json(&creds, &serde_json::from_str(&self_attested_attrs).unwrap(), &proof_req)
                .unwrap();
        assert_eq!(test.to_string(), requested_credential);
    }

    #[tokio::test]
    async fn test_build_rev_states_json() {
        let _setup = SetupMocks::init();

        let cred1 = CredInfoProver {
            referent: "height".to_string(),
            credential_referent: "abc".to_string(),
            schema_id: SCHEMA_ID.to_string(),
            cred_def_id: CRED_DEF_ID.to_string(),
            rev_reg_id: Some(REV_REG_ID.to_string()),
            cred_rev_id: Some(CRED_REV_ID.to_string()),
            tails_dir: Some(get_temp_dir_path().to_str().unwrap().to_string()),
            revocation_interval: None,
            timestamp: None,
            revealed: None,
        };
        let mut cred_info = vec![cred1];
        let anoncreds: Arc<dyn BaseAnonCreds> = Arc::new(MockAnoncreds {});
        let ledger_read: Arc<dyn AnoncredsLedgerRead> = Arc::new(MockLedger {});
        let states = build_rev_states_json(&ledger_read, &anoncreds, cred_info.as_mut())
            .await
            .unwrap();
        let rev_state_json: Value = serde_json::from_str(REV_STATE_JSON).unwrap();
        let expected = serde_json::from_value(json!({REV_REG_ID: {"1": rev_state_json}})).unwrap();
        assert_eq!(states, expected);
        assert!(cred_info[0].timestamp.is_some());
    }

    #[test]
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
        assert_eq!(
            find_revocation_interval("not here", &proof_req).unwrap_err().kind(),
            AriesVcxErrorKind::InvalidProofCredentialData
        );

        // attribute interval overrides proof request interval
        let interval = Some(NonRevokedInterval {
            from: Some(123),
            to: Some(456),
        });
        assert_eq!(find_revocation_interval("address1_1", &proof_req).unwrap(), interval);

        // when attribute interval is None, defaults to proof req interval
        let interval = Some(NonRevokedInterval {
            from: Some(098),
            to: Some(123),
        });
        assert_eq!(find_revocation_interval("zip_2", &proof_req).unwrap(), interval);

        // No interval provided for attribute or proof req
        assert_eq!(
            find_revocation_interval("address1_1", &proof_req_no_interval()).unwrap(),
            None
        );
    }
}
