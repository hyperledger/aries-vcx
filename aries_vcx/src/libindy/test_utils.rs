use std::thread;
use std::time::Duration;
use vdrtools_sys::{PoolHandle, WalletHandle};

use crate::libindy;
use crate::libindy::primitives::credential_definition::CredentialDefConfigBuilder;
use crate::libindy::primitives::revocation_registry::RevocationRegistry;
use crate::libindy::credentials::encoding::encode_attributes;
use crate::libindy::primitives::credential_definition::CredentialDef;
use crate::libindy::{credentials, proofs};
use crate::libindy::ledger::transactions::get_cred_def_json;
use crate::libindy::proofs::prover::prover::libindy_prover_get_credentials_for_proof_req;
use crate::libindy::ledger::transactions::{append_txn_author_agreement_to_request, check_response, get_rev_reg_def_json, libindy_build_schema_request, sign_and_submit_to_ledger};
use crate::libindy::primitives::credential_schema::libindy_issuer_create_schema;
use crate::libindy::proofs::prover::prover::libindy_prover_create_proof;
use crate::utils::constants::{DEFAULT_SCHEMA_ATTRS, TAILS_DIR, TEST_TAILS_URL};
use crate::utils::get_temp_dir_path;

use super::*;

extern crate serde_json;

pub async fn create_schema(attr_list: &str, submitter_did: &str) -> (String, String) {
    let data = attr_list.to_string();
    let schema_name: String = crate::utils::random::generate_random_schema_name();
    let schema_version: String = crate::utils::random::generate_random_schema_version();

    libindy_issuer_create_schema(&submitter_did, &schema_name, &schema_version, &data)
        .await
        .unwrap()
}

pub async fn create_schema_req(schema_json: &str, submitter_did: &str) -> String {
    let request = libindy_build_schema_request(submitter_did, schema_json)
        .await
        .unwrap();
    append_txn_author_agreement_to_request(&request).await.unwrap()
}

pub async fn create_and_write_test_schema(wallet_handle: WalletHandle, pool_handle: PoolHandle, submitter_did: &str, attr_list: &str) -> (String, String) {
    let (schema_id, schema_json) = create_schema(attr_list, submitter_did).await;
    let req = create_schema_req(&schema_json, submitter_did).await;
    let response = sign_and_submit_to_ledger(wallet_handle, pool_handle, submitter_did, &req).await.unwrap();
    check_response(&response).unwrap();
    thread::sleep(Duration::from_millis(1000));
    (schema_id, schema_json)
}

pub async fn create_and_store_nonrevocable_credential_def(
    wallet_handle: WalletHandle,
    pool_handle: PoolHandle,
    issuer_did: &str,
    attr_list: &str,
) -> (String, String, String, String, CredentialDef) {
    let (schema_id, schema_json) = create_and_write_test_schema(wallet_handle, pool_handle, issuer_did, attr_list).await;
    let config = CredentialDefConfigBuilder::default()
        .issuer_did(issuer_did)
        .schema_id(&schema_id)
        .tag("1")
        .build()
        .unwrap();
    let cred_def = CredentialDef::create(wallet_handle, pool_handle, "1".to_string(), config, false)
        .await
        .unwrap()
        .publish_cred_def(wallet_handle, pool_handle)
        .await
        .unwrap();
    thread::sleep(Duration::from_millis(1000));
    let cred_def_id = cred_def.get_cred_def_id();
    thread::sleep(Duration::from_millis(1000));
    let (_, cred_def_json) = get_cred_def_json(wallet_handle, pool_handle, &cred_def_id).await.unwrap();
    (schema_id, schema_json, cred_def_id, cred_def_json, cred_def)
}

pub async fn create_and_store_credential_def(
    wallet_handle: WalletHandle,
    pool_handle: PoolHandle,
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
    let (schema_id, schema_json) = create_and_write_test_schema(wallet_handle, pool_handle, issuer_did, attr_list).await;
    thread::sleep(Duration::from_millis(500));
    let config = CredentialDefConfigBuilder::default()
        .issuer_did(issuer_did)
        .schema_id(&schema_id)
        .tag("1")
        .build()
        .unwrap();
    let cred_def = CredentialDef::create(wallet_handle, pool_handle, "1".to_string(), config, true)
        .await
        .unwrap()
        .publish_cred_def(wallet_handle, pool_handle)
        .await
        .unwrap();
    let mut rev_reg = RevocationRegistry::create(
        wallet_handle,
        issuer_did,
        &cred_def.cred_def_id,
        get_temp_dir_path(TAILS_DIR).to_str().unwrap(),
        10,
        1,
    )
        .await
        .unwrap();
    rev_reg
        .publish_revocation_primitives(wallet_handle, pool_handle, TEST_TAILS_URL)
        .await
        .unwrap();

    thread::sleep(Duration::from_millis(1000));
    let cred_def_id = cred_def.get_cred_def_id();
    thread::sleep(Duration::from_millis(1000));
    let (_, cred_def_json) = get_cred_def_json(wallet_handle, pool_handle, &cred_def_id).await.unwrap();
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
    wallet_handle: WalletHandle,
    did: &str,
    cred_def_id: &str,
    cred_def_json: &str,
) -> (String, String, String) {
    let offer = credentials::issuer::libindy_issuer_create_credential_offer(wallet_handle, cred_def_id)
        .await
        .unwrap();
    let (req, req_meta) = credentials::holder::libindy_prover_create_credential_req(
        wallet_handle,
        &did,
        &offer,
        cred_def_json,
    )
        .await
        .unwrap();
    (offer, req, req_meta)
}

// todo: extract create_and_store_credential_def into caller functions
pub async fn create_and_store_credential(
    wallet_handle: WalletHandle,
    pool_handle: PoolHandle,
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
) {
    let (schema_id, schema_json, cred_def_id, cred_def_json, rev_reg_id, _, _) =
        create_and_store_credential_def(wallet_handle, pool_handle, institution_did, attr_list).await;

    let (offer, req, req_meta) = create_credential_req(wallet_handle, institution_did, &cred_def_id, &cred_def_json).await;

    /* create cred */
    let credential_data = r#"{"address1": ["123 Main St"], "address2": ["Suite 3"], "city": ["Draper"], "state": ["UT"], "zip": ["84000"]}"#;
    let encoded_attributes = encode_attributes(&credential_data).unwrap();
    let (_id, rev_def_json) = get_rev_reg_def_json(pool_handle, &rev_reg_id).await.unwrap();
    let tails_file = get_temp_dir_path(TAILS_DIR).to_str().unwrap().to_string();

    let (cred, cred_rev_id, _) = credentials::issuer::libindy_issuer_create_credential(
        wallet_handle,
        &offer,
        &req,
        &encoded_attributes,
        Some(rev_reg_id.clone()),
        Some(tails_file),
    )
        .await
        .unwrap();
    /* store cred */
    let cred_id = credentials::holder::libindy_prover_store_credential(
        wallet_handle,
        None,
        &req_meta,
        &cred,
        &cred_def_json,
        Some(&rev_def_json),
    )
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
    )
}

// todo: extract create_and_store_nonrevocable_credential_def into caller functions
pub async fn create_and_store_nonrevocable_credential(
    wallet_handle: WalletHandle,
    pool_handle: PoolHandle,
    issuer_did: &str,
    attr_list: &str,
) -> (String, String, String, String, String, String, String, String) {
    let (schema_id, schema_json, cred_def_id, cred_def_json, _) =
        create_and_store_nonrevocable_credential_def(wallet_handle, pool_handle, issuer_did, attr_list).await;

    let (offer, req, req_meta) = create_credential_req(wallet_handle, issuer_did, &cred_def_id, &cred_def_json).await;

    /* create cred */
    let credential_data = r#"{"address1": ["123 Main St"], "address2": ["Suite 3"], "city": ["Draper"], "state": ["UT"], "zip": ["84000"]}"#;
    let encoded_attributes = encode_attributes(&credential_data).unwrap();

    let (cred, _, _) = credentials::issuer::libindy_issuer_create_credential(
        wallet_handle,
        &offer,
        &req,
        &encoded_attributes,
        None,
        None,
    )
        .await
        .unwrap();
    /* store cred */
    let cred_id = credentials::holder::libindy_prover_store_credential(
        wallet_handle,
        None,
        &req_meta,
        &cred,
        &cred_def_json,
        None,
    )
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

pub async fn create_indy_proof(wallet_handle: WalletHandle, pool_handle: PoolHandle, did: &str) -> (String, String, String, String) {
    let (schema_id, schema_json, cred_def_id, cred_def_json, _offer, _req, _req_meta, cred_id) =
        create_and_store_nonrevocable_credential(wallet_handle, pool_handle, &did, DEFAULT_SCHEMA_ATTRS).await;
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

    libindy_prover_get_credentials_for_proof_req(wallet_handle, &proof_req)
        .await
        .unwrap();

    let proof = libindy_prover_create_proof(
        wallet_handle,
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
    wallet_handle: WalletHandle,
    pool_handle: PoolHandle,
    did: &str,
    include_predicate_cred: bool,
) -> (String, String, String, String) {
    let (schema_id, schema_json, cred_def_id, cred_def_json, _offer, _req, _req_meta, cred_id) =
        create_and_store_nonrevocable_credential(wallet_handle, pool_handle, &did, DEFAULT_SCHEMA_ATTRS).await;

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

    libindy_prover_get_credentials_for_proof_req(wallet_handle, &proof_req)
        .await
        .unwrap();

    let proof = libindy_prover_create_proof(
        wallet_handle,
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
