use std::{sync::Arc, thread, time::Duration};

use vdrtools::{PoolHandle, WalletHandle};

use crate::{
    common::{
        credentials::encoding::encode_attributes,
        primitives::{
            credential_definition::{CredentialDef, CredentialDefConfigBuilder},
            revocation_registry::RevocationRegistry,
        },
    },
    core::profile::{indy_profile::IndySdkProfile, profile::Profile},
    global::settings,
    utils::{
        constants::{DEFAULT_SCHEMA_ATTRS, TAILS_DIR, TEST_TAILS_URL, TRUSTEE_SEED},
        get_temp_dir_path,
        mockdata::profile::mock_profile::MockProfile,
    },
};

pub async fn create_schema(profile: &Arc<dyn Profile>, attr_list: &str, submitter_did: &str) -> (String, String) {
    let data = attr_list.to_string();
    let schema_name: String = crate::utils::random::generate_random_schema_name();
    let schema_version: String = crate::utils::random::generate_random_schema_version();

    let anoncreds = Arc::clone(profile).inject_anoncreds();
    anoncreds
        .issuer_create_schema(&submitter_did, &schema_name, &schema_version, &data)
        .await
        .unwrap()
}

pub async fn create_and_write_test_schema(
    profile: &Arc<dyn Profile>,
    submitter_did: &str,
    attr_list: &str,
) -> (String, String) {
    let (schema_id, schema_json) = create_schema(profile, attr_list, submitter_did).await;
    let ledger = Arc::clone(profile).inject_ledger();
    let _response = ledger.publish_schema(&schema_json, submitter_did, None).await.unwrap();
    thread::sleep(Duration::from_millis(1000));
    (schema_id, schema_json)
}

pub async fn create_and_store_nonrevocable_credential_def(
    profile: &Arc<dyn Profile>,
    issuer_did: &str,
    attr_list: &str,
) -> (String, String, String, String, CredentialDef) {
    let (schema_id, schema_json) = create_and_write_test_schema(profile, issuer_did, attr_list).await;
    let config = CredentialDefConfigBuilder::default()
        .issuer_did(issuer_did)
        .schema_id(&schema_id)
        .tag("1")
        .build()
        .unwrap();
    let cred_def = CredentialDef::create(profile, "1".to_string(), config, false)
        .await
        .unwrap()
        .publish_cred_def(profile)
        .await
        .unwrap();
    thread::sleep(Duration::from_millis(1000));
    let cred_def_id = cred_def.get_cred_def_id();
    thread::sleep(Duration::from_millis(1000));

    let ledger = Arc::clone(profile).inject_ledger();
    let cred_def_json = ledger.get_cred_def(&cred_def_id, None).await.unwrap();
    (schema_id, schema_json, cred_def_id, cred_def_json, cred_def)
}

pub async fn create_and_store_credential_def(
    profile: &Arc<dyn Profile>,
    issuer_did: &str,
    attr_list: &str,
) -> (
    String,
    String,
    String,
    String,
    String,
    CredentialDef,
    RevocationRegistry,
) {
    let (schema_id, schema_json) = create_and_write_test_schema(profile, issuer_did, attr_list).await;
    thread::sleep(Duration::from_millis(500));
    let config = CredentialDefConfigBuilder::default()
        .issuer_did(issuer_did)
        .schema_id(&schema_id)
        .tag("1")
        .build()
        .unwrap();
    let cred_def = CredentialDef::create(profile, "1".to_string(), config, true)
        .await
        .unwrap()
        .publish_cred_def(profile)
        .await
        .unwrap();
    let mut rev_reg = RevocationRegistry::create(
        profile,
        issuer_did,
        &cred_def.get_cred_def_id(),
        get_temp_dir_path(TAILS_DIR).to_str().unwrap(),
        10,
        1,
    )
    .await
    .unwrap();
    rev_reg
        .publish_revocation_primitives(profile, TEST_TAILS_URL)
        .await
        .unwrap();

    thread::sleep(Duration::from_millis(1000));
    let cred_def_id = cred_def.get_cred_def_id();
    thread::sleep(Duration::from_millis(1000));
    let ledger = Arc::clone(profile).inject_ledger();
    let cred_def_json = ledger.get_cred_def(&cred_def_id, None).await.unwrap();
    (
        schema_id,
        schema_json,
        cred_def_id,
        cred_def_json,
        rev_reg.get_rev_reg_id(),
        cred_def,
        rev_reg,
    )
}

pub async fn create_credential_req(
    issuer: &Arc<dyn Profile>, /* FUTURE - issuer and holder seperation only needed whilst modular deps not fully
                                * implemented */
    holder: &Arc<dyn Profile>,
    did: &str,
    cred_def_id: &str,
    cred_def_json: &str,
) -> (String, String, String) {
    let offer = Arc::clone(issuer)
        .inject_anoncreds()
        .issuer_create_credential_offer(cred_def_id)
        .await
        .unwrap();
    let master_secret_name = settings::DEFAULT_LINK_SECRET_ALIAS;
    let (req, req_meta) = Arc::clone(holder)
        .inject_anoncreds()
        .prover_create_credential_req(&did, &offer, cred_def_json, master_secret_name)
        .await
        .unwrap();
    (offer, req, req_meta)
}

// todo: extract create_and_store_credential_def into caller functions
pub async fn create_and_store_credential(
    issuer: &Arc<dyn Profile>, /* FUTURE - issuer and holder seperation only needed whilst modular deps not fully
                                * implemented */
    holder: &Arc<dyn Profile>,
    institution_did: &str,
    attr_list: &str,
) -> (
    String,
    String,
    String,
    String,
    String,
    String,
    String,
    String,
    String,
    String,
    String,
) {
    let (schema_id, schema_json, cred_def_id, cred_def_json, rev_reg_id, _, _) =
        create_and_store_credential_def(issuer, institution_did, attr_list).await;

    let (offer, req, req_meta) =
        create_credential_req(issuer, holder, institution_did, &cred_def_id, &cred_def_json).await;

    /* create cred */
    let credential_data = r#"{"address1": ["123 Main St"], "address2": ["Suite 3"], "city": ["Draper"], "state": ["UT"], "zip": ["84000"]}"#;
    let encoded_attributes = encode_attributes(&credential_data).unwrap();
    let ledger = Arc::clone(issuer).inject_ledger();
    let rev_def_json = ledger.get_rev_reg_def_json(&rev_reg_id).await.unwrap();
    let tails_file = get_temp_dir_path(TAILS_DIR).to_str().unwrap().to_string();

    let (cred, cred_rev_id, _) = Arc::clone(issuer)
        .inject_anoncreds()
        .issuer_create_credential(
            &offer,
            &req,
            &encoded_attributes,
            Some(rev_reg_id.clone()),
            Some(tails_file.clone()),
        )
        .await
        .unwrap();
    /* store cred */
    let cred_id = Arc::clone(holder)
        .inject_anoncreds()
        .prover_store_credential(None, &req_meta, &cred, &cred_def_json, Some(&rev_def_json))
        .await
        .unwrap();
    (
        schema_id,
        schema_json,
        cred_def_id,
        cred_def_json,
        offer,
        req,
        req_meta,
        cred_id,
        rev_reg_id,
        cred_rev_id.unwrap(),
        tails_file,
    )
}

// todo: extract create_and_store_nonrevocable_credential_def into caller functions
pub async fn create_and_store_nonrevocable_credential(
    issuer: &Arc<dyn Profile>, /* FUTURE - issuer and holder seperation only needed whilst modular deps not fully
                                * implemented */
    holder: &Arc<dyn Profile>,
    issuer_did: &str,
    attr_list: &str,
) -> (String, String, String, String, String, String, String, String) {
    let (schema_id, schema_json, cred_def_id, cred_def_json, _) =
        create_and_store_nonrevocable_credential_def(issuer, issuer_did, attr_list).await;

    let (offer, req, req_meta) = create_credential_req(issuer, holder, issuer_did, &cred_def_id, &cred_def_json).await;

    /* create cred */
    let credential_data = r#"{"address1": ["123 Main St"], "address2": ["Suite 3"], "city": ["Draper"], "state": ["UT"], "zip": ["84000"]}"#;
    let encoded_attributes = encode_attributes(&credential_data).unwrap();

    let (cred, _, _) = Arc::clone(issuer)
        .inject_anoncreds()
        .issuer_create_credential(&offer, &req, &encoded_attributes, None, None)
        .await
        .unwrap();
    /* store cred */
    let cred_id = Arc::clone(holder)
        .inject_anoncreds()
        .prover_store_credential(None, &req_meta, &cred, &cred_def_json, None)
        .await
        .unwrap();
    (
        schema_id,
        schema_json,
        cred_def_id,
        cred_def_json,
        offer,
        req,
        req_meta,
        cred_id,
    )
}

// FUTURE - issuer and holder seperation only needed whilst modular deps not fully implemented
pub async fn create_indy_proof(
    issuer: &Arc<dyn Profile>,
    prover: &Arc<dyn Profile>,
    did: &str,
) -> (String, String, String, String) {
    let (schema_id, schema_json, cred_def_id, cred_def_json, _offer, _req, _req_meta, cred_id) =
        create_and_store_nonrevocable_credential(issuer, prover, &did, DEFAULT_SCHEMA_ATTRS).await;
    let proof_req = json!({
       "nonce":"123432421212",
       "name":"proof_req_1",
       "version":"0.1",
       "requested_attributes": json!({
           "address1_1": json!({
               "name":"address1",
               "restrictions": [json!({ "issuer_did": did })]
           }),
           "zip_2": json!({
               "name":"zip",
               "restrictions": [json!({ "issuer_did": did })]
           }),
           "self_attest_3": json!({
               "name":"self_attest",
           }),
       }),
       "requested_predicates": json!({}),
    })
    .to_string();
    let requested_credentials_json = json!({
          "self_attested_attributes":{
             "self_attest_3": "my_self_attested_val"
          },
          "requested_attributes":{
             "address1_1": {"cred_id": cred_id, "revealed": true},
             "zip_2": {"cred_id": cred_id, "revealed": true}
            },
          "requested_predicates":{}
    })
    .to_string();

    let schema_json: serde_json::Value = serde_json::from_str(&schema_json).unwrap();
    let schemas = json!({
        schema_id: schema_json,
    })
    .to_string();

    let cred_def_json: serde_json::Value = serde_json::from_str(&cred_def_json).unwrap();
    let cred_defs = json!({
        cred_def_id: cred_def_json,
    })
    .to_string();

    let anoncreds = Arc::clone(prover).inject_anoncreds();

    anoncreds
        .prover_get_credentials_for_proof_req(&proof_req)
        .await
        .unwrap();

    let proof = anoncreds
        .prover_create_proof(
            &proof_req,
            &requested_credentials_json,
            "main",
            &schemas,
            &cred_defs,
            None,
        )
        .await
        .unwrap();
    (schemas, cred_defs, proof_req, proof)
}

pub async fn create_proof_with_predicate(
    issuer: &Arc<dyn Profile>, /* FUTURE - issuer and holder seperation only needed whilst modular deps not fully
                                * implemented */
    prover: &Arc<dyn Profile>,
    did: &str,
    include_predicate_cred: bool,
) -> (String, String, String, String) {
    let (schema_id, schema_json, cred_def_id, cred_def_json, _offer, _req, _req_meta, cred_id) =
        create_and_store_nonrevocable_credential(issuer, prover, &did, DEFAULT_SCHEMA_ATTRS).await;

    let proof_req = json!({
       "nonce":"123432421212",
       "name":"proof_req_1",
       "version":"0.1",
       "requested_attributes": json!({
           "address1_1": json!({
               "name":"address1",
               "restrictions": [json!({ "issuer_did": did })]
           }),
           "self_attest_3": json!({
               "name":"self_attest",
           }),
       }),
       "requested_predicates": json!({
           "zip_3": {"name":"zip", "p_type":">=", "p_value":18}
       }),
    })
    .to_string();

    let requested_credentials_json;
    if include_predicate_cred {
        requested_credentials_json = json!({
          "self_attested_attributes":{
             "self_attest_3": "my_self_attested_val"
          },
          "requested_attributes":{
             "address1_1": {"cred_id": cred_id, "revealed": true}
            },
          "requested_predicates":{
              "zip_3": {"cred_id": cred_id}
          }
        })
        .to_string();
    } else {
        requested_credentials_json = json!({
          "self_attested_attributes":{
             "self_attest_3": "my_self_attested_val"
          },
          "requested_attributes":{
             "address1_1": {"cred_id": cred_id, "revealed": true}
            },
          "requested_predicates":{
          }
        })
        .to_string();
    }

    let schema_json: serde_json::Value = serde_json::from_str(&schema_json).unwrap();
    let schemas = json!({
        schema_id: schema_json,
    })
    .to_string();

    let cred_def_json: serde_json::Value = serde_json::from_str(&cred_def_json).unwrap();
    let cred_defs = json!({
        cred_def_id: cred_def_json,
    })
    .to_string();

    let anoncreds = Arc::clone(prover).inject_anoncreds();

    anoncreds
        .prover_get_credentials_for_proof_req(&proof_req)
        .await
        .unwrap();

    let proof = anoncreds
        .prover_create_proof(
            &proof_req,
            &requested_credentials_json,
            "main",
            &schemas,
            &cred_defs,
            None,
        )
        .await
        .unwrap();
    (schemas, cred_defs, proof_req, proof)
}

pub async fn create_trustee_key(profile: &Arc<dyn Profile>) -> String {
    Arc::clone(profile)
        .inject_wallet()
        .create_and_store_my_did(Some(TRUSTEE_SEED), None)
        .await
        .unwrap()
        .1
}

// TODO - FUTURE - should be a standalone method within wallet - not depending on create did
pub async fn create_key(profile: &Arc<dyn Profile>) -> String {
    let seed: String = crate::utils::random::generate_random_seed();
    Arc::clone(profile)
        .inject_wallet()
        .create_and_store_my_did(Some(&seed), None)
        .await
        .unwrap()
        .1
}

// used for mocking profile
pub fn mock_profile() -> Arc<dyn Profile> {
    Arc::new(MockProfile {})
}

// TODO - FUTURE - should only be used for quick mock setups, should be removable after full
// detachment from vdrtools dep
pub fn indy_handles_to_profile(wallet_handle: WalletHandle, pool_handle: PoolHandle) -> Arc<dyn Profile> {
    Arc::new(IndySdkProfile::new(wallet_handle, pool_handle))
}
