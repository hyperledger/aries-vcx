#![allow(clippy::unwrap_used)]

use aries_vcx_core::anoncreds::base_anoncreds::BaseAnonCreds;
use aries_vcx_core::ledger::base_ledger::{AnoncredsLedgerRead, AnoncredsLedgerWrite};
use aries_vcx_core::ledger::indy::pool::test_utils::{get_temp_dir_path, get_temp_file_path};
use std::sync::Arc;
use std::thread;
use std::time::Duration;

use aries_vcx_core::wallet::base_wallet::BaseWallet;

use crate::common::credentials::encoding::encode_attributes;
use crate::common::primitives::credential_definition::CredentialDef;
use crate::common::primitives::credential_definition::CredentialDefConfigBuilder;
use crate::common::primitives::revocation_registry::RevocationRegistry;
use crate::global::settings;
use crate::utils::constants::{DEFAULT_SCHEMA_ATTRS, TEST_TAILS_URL, TRUSTEE_SEED};

pub async fn create_schema(
    anoncreds: &Arc<dyn BaseAnonCreds>,
    attr_list: &str,
    submitter_did: &str,
) -> (String, String) {
    let data = attr_list.to_string();
    let schema_name: String = crate::utils::random::generate_random_schema_name();
    let schema_version: String = crate::utils::random::generate_random_schema_version();

    anoncreds
        .issuer_create_schema(&submitter_did, &schema_name, &schema_version, &data)
        .await
        .unwrap()
}

pub async fn create_and_write_test_schema(
    anoncreds: &Arc<dyn BaseAnonCreds>,
    ledger_write: &Arc<dyn AnoncredsLedgerWrite>,
    submitter_did: &str,
    attr_list: &str,
) -> (String, String) {
    let (schema_id, schema_json) = create_schema(anoncreds, attr_list, submitter_did).await;
    let _response = ledger_write
        .publish_schema(&schema_json, submitter_did, None)
        .await
        .unwrap();
    tokio::time::sleep(Duration::from_millis(1000)).await;
    (schema_id, schema_json)
}

pub async fn create_and_store_nonrevocable_credential_def(
    anoncreds: &Arc<dyn BaseAnonCreds>,
    ledger_read: &Arc<dyn AnoncredsLedgerRead>,
    ledger_write: &Arc<dyn AnoncredsLedgerWrite>,
    issuer_did: &str,
    attr_list: &str,
) -> (String, String, String, String, CredentialDef) {
    let (schema_id, schema_json) = create_and_write_test_schema(anoncreds, ledger_write, issuer_did, attr_list).await;
    let config = CredentialDefConfigBuilder::default()
        .issuer_did(issuer_did)
        .schema_id(&schema_id)
        .tag("1")
        .build()
        .unwrap();
    let cred_def = CredentialDef::create(ledger_read, anoncreds, "1".to_string(), config, false)
        .await
        .unwrap()
        .publish_cred_def(ledger_read, ledger_write)
        .await
        .unwrap();
    tokio::time::sleep(Duration::from_millis(1000)).await;
    let cred_def_id = cred_def.get_cred_def_id();
    tokio::time::sleep(Duration::from_millis(1000)).await;

    let cred_def_json = ledger_read.get_cred_def(&cred_def_id, None).await.unwrap();
    (schema_id, schema_json, cred_def_id, cred_def_json, cred_def)
}

pub async fn create_and_store_credential_def_and_rev_reg(
    anoncreds: &Arc<dyn BaseAnonCreds>,
    ledger_read: &Arc<dyn AnoncredsLedgerRead>,
    ledger_write: &Arc<dyn AnoncredsLedgerWrite>,
    issuer_did: &str,
    attr_list: &str,
) -> (
    String,
    String,
    String,
    String,
    String,
    String,
    CredentialDef,
    RevocationRegistry,
) {
    let (schema_id, schema_json) = create_and_write_test_schema(anoncreds, ledger_write, issuer_did, attr_list).await;
    thread::sleep(Duration::from_millis(500));
    let config = CredentialDefConfigBuilder::default()
        .issuer_did(issuer_did)
        .schema_id(&schema_id)
        .tag("1")
        .build()
        .unwrap();
    let cred_def = CredentialDef::create(ledger_read, anoncreds, "1".to_string(), config, true)
        .await
        .unwrap()
        .publish_cred_def(ledger_read, ledger_write)
        .await
        .unwrap();

    let tails_dir = String::from(get_temp_dir_path().as_path().to_str().unwrap());

    let mut rev_reg = RevocationRegistry::create(anoncreds, issuer_did, &cred_def.get_cred_def_id(), &tails_dir, 10, 1)
        .await
        .unwrap();
    rev_reg
        .publish_revocation_primitives(ledger_write, TEST_TAILS_URL)
        .await
        .unwrap();

    tokio::time::sleep(Duration::from_millis(1000)).await;
    let cred_def_id = cred_def.get_cred_def_id();
    tokio::time::sleep(Duration::from_millis(1000)).await;
    let cred_def_json = ledger_read.get_cred_def(&cred_def_id, None).await.unwrap();
    (
        schema_id,
        schema_json,
        cred_def_id,
        cred_def_json,
        rev_reg.get_rev_reg_id(),
        tails_dir,
        cred_def,
        rev_reg,
    )
}

pub async fn create_credential_req(
    anoncreds_issuer: &Arc<dyn BaseAnonCreds>,
    anoncreds_holder: &Arc<dyn BaseAnonCreds>,
    did: &str,
    cred_def_id: &str,
    cred_def_json: &str,
) -> (String, String, String) {
    let offer = anoncreds_issuer
        .issuer_create_credential_offer(cred_def_id)
        .await
        .unwrap();
    let master_secret_name = settings::DEFAULT_LINK_SECRET_ALIAS;
    let (req, req_meta) = anoncreds_holder
        .prover_create_credential_req(&did, &offer, cred_def_json, master_secret_name)
        .await
        .unwrap();
    (offer, req, req_meta)
}

// todo: extract create_and_store_credential_def into caller functions
pub async fn create_and_store_credential(
    anoncreds_issuer: &Arc<dyn BaseAnonCreds>,
    anoncreds_holder: &Arc<dyn BaseAnonCreds>,
    ledger_read: &Arc<dyn AnoncredsLedgerRead>,
    ledger_write: &Arc<dyn AnoncredsLedgerWrite>,
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
    RevocationRegistry,
) {
    let (schema_id, schema_json, cred_def_id, cred_def_json, rev_reg_id, tails_dir, _, rev_reg) =
        create_and_store_credential_def_and_rev_reg(
            anoncreds_issuer,
            ledger_read,
            ledger_write,
            institution_did,
            attr_list,
        )
        .await;

    let (offer, req, req_meta) = create_credential_req(
        anoncreds_issuer,
        anoncreds_holder,
        institution_did,
        &cred_def_id,
        &cred_def_json,
    )
    .await;

    /* create cred */
    let credential_data = r#"{"address1": ["123 Main St"], "address2": ["Suite 3"], "city": ["Draper"], "state": ["UT"], "zip": ["84000"]}"#;
    let encoded_attributes = encode_attributes(&credential_data).unwrap();
    let rev_def_json = ledger_read.get_rev_reg_def_json(&rev_reg_id).await.unwrap();

    let (cred, cred_rev_id, _) = anoncreds_issuer
        .issuer_create_credential(
            &offer,
            &req,
            &encoded_attributes,
            Some(rev_reg_id.clone()),
            Some(tails_dir.clone()),
        )
        .await
        .unwrap();
    /* store cred */
    let cred_id = anoncreds_holder
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
        tails_dir,
        rev_reg,
    )
}

// todo: extract create_and_store_nonrevocable_credential_def into caller functions
pub async fn create_and_store_nonrevocable_credential(
    anoncreds_issuer: &Arc<dyn BaseAnonCreds>,
    anoncreds_holder: &Arc<dyn BaseAnonCreds>,
    ledger_read: &Arc<dyn AnoncredsLedgerRead>,
    ledger_write: &Arc<dyn AnoncredsLedgerWrite>,
    issuer_did: &str,
    attr_list: &str,
) -> (String, String, String, String, String, String, String, String) {
    let (schema_id, schema_json, cred_def_id, cred_def_json, _) = create_and_store_nonrevocable_credential_def(
        anoncreds_issuer,
        ledger_read,
        ledger_write,
        issuer_did,
        attr_list,
    )
    .await;

    let (offer, req, req_meta) = create_credential_req(
        anoncreds_issuer,
        anoncreds_holder,
        issuer_did,
        &cred_def_id,
        &cred_def_json,
    )
    .await;

    /* create cred */
    let credential_data = r#"{"address1": ["123 Main St"], "address2": ["Suite 3"], "city": ["Draper"], "state": ["UT"], "zip": ["84000"]}"#;
    let encoded_attributes = encode_attributes(&credential_data).unwrap();

    let (cred, _, _) = anoncreds_issuer
        .issuer_create_credential(&offer, &req, &encoded_attributes, None, None)
        .await
        .unwrap();
    /* store cred */
    let cred_id = anoncreds_holder
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
    anoncreds_issuer: &Arc<dyn BaseAnonCreds>,
    anoncreds_holder: &Arc<dyn BaseAnonCreds>,
    ledger_read: &Arc<dyn AnoncredsLedgerRead>,
    ledger_write: &Arc<dyn AnoncredsLedgerWrite>,
    did: &str,
) -> (String, String, String, String) {
    let (schema_id, schema_json, cred_def_id, cred_def_json, _offer, _req, _req_meta, cred_id) =
        create_and_store_nonrevocable_credential(
            anoncreds_issuer,
            anoncreds_holder,
            ledger_read,
            ledger_write,
            &did,
            DEFAULT_SCHEMA_ATTRS,
        )
        .await;
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

    anoncreds_holder
        .prover_get_credentials_for_proof_req(&proof_req)
        .await
        .unwrap();

    let proof = anoncreds_holder
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
    anoncreds_issuer: &Arc<dyn BaseAnonCreds>,
    anoncreds_holder: &Arc<dyn BaseAnonCreds>,
    ledger_read: &Arc<dyn AnoncredsLedgerRead>,
    ledger_write: &Arc<dyn AnoncredsLedgerWrite>,
    did: &str,
    include_predicate_cred: bool,
) -> (String, String, String, String) {
    let (schema_id, schema_json, cred_def_id, cred_def_json, _offer, _req, _req_meta, cred_id) =
        create_and_store_nonrevocable_credential(
            anoncreds_issuer,
            anoncreds_holder,
            ledger_read,
            ledger_write,
            &did,
            DEFAULT_SCHEMA_ATTRS,
        )
        .await;

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

    anoncreds_holder
        .prover_get_credentials_for_proof_req(&proof_req)
        .await
        .unwrap();

    let proof = anoncreds_holder
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

pub async fn create_trustee_key(wallet: &Arc<dyn BaseWallet>) -> String {
    wallet
        .create_and_store_my_did(Some(TRUSTEE_SEED), None)
        .await
        .unwrap()
        .1
}

// TODO - FUTURE - should be a standalone method within wallet - not depending on create did
pub async fn create_key(wallet: &Arc<dyn BaseWallet>) -> String {
    let seed: String = crate::utils::random::generate_random_seed();
    wallet.create_and_store_my_did(Some(&seed), None).await.unwrap().1
}
