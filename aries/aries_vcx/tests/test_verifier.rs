use std::{collections::HashMap, error::Error, time::Duration};

use anoncreds_types::data_types::{
    identifiers::{cred_def_id::CredentialDefinitionId, schema_id::SchemaId},
    ledger::{cred_def::CredentialDefinition, schema::Schema},
};
use aries_vcx::{
    common::{
        primitives::{
            credential_definition::CredentialDef, credential_schema::Schema as SchemaPrimitive,
        },
        proofs::{proof_request::ProofRequestData, verifier::validate_indy_proof},
    },
    errors::error::AriesVcxErrorKind,
};
use aries_vcx_core::{
    anoncreds::base_anoncreds::BaseAnonCreds,
    ledger::base_ledger::{AnoncredsLedgerRead, AnoncredsLedgerWrite},
    wallet::base_wallet::BaseWallet,
};
use did_parser::Did;
use serde_json::json;
use test_utils::{constants::DEFAULT_SCHEMA_ATTRS, devsetup::build_setup_profile};

use crate::utils::{
    create_and_write_credential, create_and_write_test_cred_def, create_and_write_test_schema,
};

pub mod utils;

// FUTURE - issuer and holder seperation only needed whilst modular deps not fully implemented
async fn create_indy_proof(
    wallet_issuer: &impl BaseWallet,
    wallet_holder: &impl BaseWallet,
    anoncreds_issuer: &impl BaseAnonCreds,
    anoncreds_holder: &impl BaseAnonCreds,
    ledger_read: &impl AnoncredsLedgerRead,
    ledger_write: &impl AnoncredsLedgerWrite,
    did: &Did,
) -> Result<
    (
        HashMap<SchemaId, Schema>,
        HashMap<CredentialDefinitionId, CredentialDefinition>,
        String,
        String,
    ),
    Box<dyn Error>,
> {
    let (schema, cred_def, cred_id) = create_and_store_nonrevocable_credential(
        wallet_issuer,
        wallet_holder,
        anoncreds_issuer,
        anoncreds_holder,
        ledger_read,
        ledger_write,
        did,
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

    let schema_id = schema.schema_id.clone();
    let schemas = json!({
        schema_id: schema.schema_json,
    })
    .to_string();

    let cred_def_json = serde_json::to_value(cred_def.get_cred_def_json())?;
    let cred_defs = json!({
        cred_def.get_cred_def_id().to_string(): cred_def_json,
    })
    .to_string();

    anoncreds_holder
        .prover_get_credentials_for_proof_req(wallet_holder, &proof_req)
        .await?;

    let proof = anoncreds_holder
        .prover_create_proof(
            wallet_holder,
            serde_json::from_str(&proof_req)?,
            &requested_credentials_json,
            "main",
            serde_json::from_str(&schemas)?,
            serde_json::from_str(&cred_defs)?,
            None,
        )
        .await?;
    Ok((
        serde_json::from_str(&schemas).unwrap(),
        serde_json::from_str(&cred_defs).unwrap(),
        proof_req,
        proof,
    ))
}

#[allow(clippy::too_many_arguments)]
async fn create_proof_with_predicate(
    wallet_issuer: &impl BaseWallet,
    wallet_holder: &impl BaseWallet,
    anoncreds_issuer: &impl BaseAnonCreds,
    anoncreds_holder: &impl BaseAnonCreds,
    ledger_read: &impl AnoncredsLedgerRead,
    ledger_write: &impl AnoncredsLedgerWrite,
    did: &Did,
    include_predicate_cred: bool,
) -> Result<
    (
        HashMap<SchemaId, Schema>,
        HashMap<CredentialDefinitionId, CredentialDefinition>,
        String,
        String,
    ),
    Box<dyn Error>,
> {
    let (schema, cred_def, cred_id) = create_and_store_nonrevocable_credential(
        wallet_issuer,
        wallet_holder,
        anoncreds_issuer,
        anoncreds_holder,
        ledger_read,
        ledger_write,
        did,
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

    let requested_credentials_json = if include_predicate_cred {
        json!({
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
        .to_string()
    } else {
        json!({
          "self_attested_attributes":{
             "self_attest_3": "my_self_attested_val"
          },
          "requested_attributes":{
             "address1_1": {"cred_id": cred_id, "revealed": true}
            },
          "requested_predicates":{
          }
        })
        .to_string()
    };

    let schemas = json!({
        schema.schema_id: schema.schema_json,
    })
    .to_string();

    let cred_def_json = serde_json::to_value(cred_def.get_cred_def_json())?;
    let cred_defs = json!({
        cred_def.get_cred_def_id().to_string(): cred_def_json,
    })
    .to_string();

    anoncreds_holder
        .prover_get_credentials_for_proof_req(wallet_holder, &proof_req)
        .await?;

    let proof = anoncreds_holder
        .prover_create_proof(
            wallet_holder,
            serde_json::from_str(&proof_req)?,
            &requested_credentials_json,
            "main",
            serde_json::from_str(&schemas)?,
            serde_json::from_str(&cred_defs)?,
            None,
        )
        .await?;
    Ok((
        serde_json::from_str(&schemas).unwrap(),
        serde_json::from_str(&cred_defs).unwrap(),
        proof_req,
        proof,
    ))
}

#[allow(clippy::too_many_arguments)]
async fn create_and_store_nonrevocable_credential(
    wallet_issuer: &impl BaseWallet,
    wallet_holder: &impl BaseWallet,
    anoncreds_issuer: &impl BaseAnonCreds,
    anoncreds_holder: &impl BaseAnonCreds,
    ledger_read: &impl AnoncredsLedgerRead,
    ledger_write: &impl AnoncredsLedgerWrite,
    issuer_did: &Did,
    attr_list: &str,
) -> (SchemaPrimitive, CredentialDef, String) {
    let schema = create_and_write_test_schema(
        wallet_issuer,
        anoncreds_issuer,
        ledger_write,
        issuer_did,
        attr_list,
    )
    .await;

    let cred_def = create_and_write_test_cred_def(
        wallet_issuer,
        anoncreds_issuer,
        ledger_read,
        ledger_write,
        issuer_did,
        &schema.schema_id,
        false,
    )
    .await;
    tokio::time::sleep(Duration::from_millis(500)).await;

    let cred_id = create_and_write_credential(
        wallet_issuer,
        wallet_holder,
        anoncreds_issuer,
        anoncreds_holder,
        issuer_did,
        &cred_def,
        None,
    )
    .await;
    (schema, cred_def, cred_id)
}

#[tokio::test]
#[ignore]
async fn test_pool_proof_self_attested_proof_validation() -> Result<(), Box<dyn Error>> {
    let setup = build_setup_profile().await;
    let requested_attrs = json!([
        json!({
            "name":"address1",
            "self_attest_allowed": true,
        }),
        json!({
            "name":"zip",
            "self_attest_allowed": true,
        }),
    ])
    .to_string();
    let requested_predicates = json!([]).to_string();
    let revocation_details = r#"{"support_revocation":false}"#.to_string();
    let name = "Optional".to_owned();

    let proof_req_json = ProofRequestData::create(&setup.anoncreds, &name)
        .await?
        .set_requested_attributes_as_string(requested_attrs)?
        .set_requested_predicates_as_string(requested_predicates)?
        .set_not_revoked_interval(revocation_details)?;

    let proof_req_json = serde_json::to_string(&proof_req_json)?;

    let anoncreds = &setup.anoncreds;
    let prover_proof_json = anoncreds
        .prover_create_proof(
            &setup.wallet,
            serde_json::from_str(&proof_req_json)?,
            &json!({
              "self_attested_attributes":{
                 "attribute_0": "my_self_attested_address",
                 "attribute_1": "my_self_attested_zip"
              },
              "requested_attributes":{},
              "requested_predicates":{}
            })
            .to_string(),
            "main",
            Default::default(),
            Default::default(),
            None,
        )
        .await?;

    assert!(
        validate_indy_proof(
            &setup.ledger_read,
            &setup.anoncreds,
            &prover_proof_json,
            &proof_req_json,
        )
        .await?
    );
    Ok(())
}

#[tokio::test]
#[ignore]
async fn test_pool_proof_restrictions() -> Result<(), Box<dyn Error>> {
    let setup = build_setup_profile().await;
    let requested_attrs = json!([
        json!({
            "name":"address1",
            "restrictions": [{ "issuer_did": "Not Here" }],
        }),
        json!({
            "name":"zip",
        }),
        json!({
            "name":"self_attest",
            "self_attest_allowed": true,
        }),
    ])
    .to_string();
    let requested_predicates = json!([]).to_string();
    let revocation_details = r#"{"support_revocation":true}"#.to_string();
    let name = "Optional".to_owned();

    let proof_req_json = ProofRequestData::create(&setup.anoncreds, &name)
        .await?
        .set_requested_attributes_as_string(requested_attrs)?
        .set_requested_predicates_as_string(requested_predicates)?
        .set_not_revoked_interval(revocation_details)?;

    let proof_req_json = serde_json::to_string(&proof_req_json)?;

    let (schema, cred_def, cred_id) = create_and_store_nonrevocable_credential(
        &setup.wallet,
        &setup.wallet,
        &setup.anoncreds,
        &setup.anoncreds,
        &setup.ledger_read,
        &setup.ledger_write,
        &setup.institution_did,
        DEFAULT_SCHEMA_ATTRS,
    )
    .await;
    let cred_def_json = serde_json::to_value(cred_def.get_cred_def_json())?;

    let anoncreds = &setup.anoncreds;
    let prover_proof_json = anoncreds
        .prover_create_proof(
            &setup.wallet,
            serde_json::from_str(&proof_req_json)?,
            &json!({
                "self_attested_attributes":{
                   "attribute_2": "my_self_attested_val"
                },
                "requested_attributes":{
                   "attribute_0": {"cred_id": cred_id, "revealed": true},
                   "attribute_1": {"cred_id": cred_id, "revealed": true}
                },
                "requested_predicates":{}
            })
            .to_string(),
            "main",
            serde_json::from_str(&json!({ schema.schema_id: schema.schema_json }).to_string())?,
            serde_json::from_str(
                &json!({ cred_def.get_cred_def_id().to_string(): cred_def_json }).to_string(),
            )?,
            None,
        )
        .await?;
    assert_eq!(
        validate_indy_proof(
            &setup.ledger_read,
            &setup.anoncreds,
            &prover_proof_json,
            &proof_req_json,
        )
        .await
        .unwrap_err()
        .kind(),
        AriesVcxErrorKind::ProofRejected
    );

    let mut proof_req_json: serde_json::Value = serde_json::from_str(&proof_req_json)?;
    proof_req_json["requested_attributes"]["attribute_0"]["restrictions"] = json!({});
    assert!(
        validate_indy_proof(
            &setup.ledger_read,
            &setup.anoncreds,
            &prover_proof_json,
            &proof_req_json.to_string(),
        )
        .await?
    );
    Ok(())
}

#[tokio::test]
#[ignore]
async fn test_pool_proof_validate_attribute() -> Result<(), Box<dyn Error>> {
    let setup = build_setup_profile().await;
    let requested_attrs = json!([
        json!({
            "name":"address1",
            "restrictions": [json!({ "issuer_did": setup.institution_did })]
        }),
        json!({
            "name":"zip",
            "restrictions": [json!({ "issuer_did": setup.institution_did })]
        }),
        json!({
            "name":"self_attest",
            "self_attest_allowed": true,
        }),
    ])
    .to_string();
    let requested_predicates = json!([]).to_string();
    let revocation_details = r#"{"support_revocation":true}"#.to_string();
    let name = "Optional".to_owned();

    let proof_req_json = ProofRequestData::create(&setup.anoncreds, &name)
        .await?
        .set_requested_attributes_as_string(requested_attrs)?
        .set_requested_predicates_as_string(requested_predicates)?
        .set_not_revoked_interval(revocation_details)?;

    let proof_req_json = serde_json::to_string(&proof_req_json)?;

    let (schema, cred_def, cred_id) = create_and_store_nonrevocable_credential(
        &setup.wallet,
        &setup.wallet,
        &setup.anoncreds,
        &setup.anoncreds,
        &setup.ledger_read,
        &setup.ledger_write,
        &setup.institution_did,
        DEFAULT_SCHEMA_ATTRS,
    )
    .await;
    let cred_def_json = serde_json::to_value(cred_def.get_cred_def_json())?;

    let anoncreds = &setup.anoncreds;
    let prover_proof_json = anoncreds
        .prover_create_proof(
            &setup.wallet,
            serde_json::from_str(&proof_req_json)?,
            &json!({
                "self_attested_attributes":{
                   "attribute_2": "my_self_attested_val"
                },
                "requested_attributes":{
                   "attribute_0": {"cred_id": cred_id, "revealed": true},
                   "attribute_1": {"cred_id": cred_id, "revealed": true}
                },
                "requested_predicates":{}
            })
            .to_string(),
            "main",
            serde_json::from_str(&json!({ schema.schema_id: schema.schema_json }).to_string())?,
            serde_json::from_str(
                &json!({ cred_def.get_cred_def_id().to_string(): cred_def_json }).to_string(),
            )?,
            None,
        )
        .await?;
    assert!(
        validate_indy_proof(
            &setup.ledger_read,
            &setup.anoncreds,
            &prover_proof_json,
            &proof_req_json,
        )
        .await?
    );

    let mut proof_obj: serde_json::Value = serde_json::from_str(&prover_proof_json)?;
    {
        proof_obj["requested_proof"]["revealed_attrs"]["address1_1"]["raw"] = json!("Other Value");
        let prover_proof_json = serde_json::to_string(&proof_obj)?;

        assert_eq!(
            validate_indy_proof(
                &setup.ledger_read,
                &setup.anoncreds,
                &prover_proof_json,
                &proof_req_json,
            )
            .await
            .unwrap_err()
            .kind(),
            AriesVcxErrorKind::InvalidProof
        );
    }
    {
        proof_obj["requested_proof"]["revealed_attrs"]["address1_1"]["encoded"] =
            json!("1111111111111111111111111111111111111111111111111111111111");
        let prover_proof_json = serde_json::to_string(&proof_obj).unwrap();

        assert_eq!(
            validate_indy_proof(
                &setup.ledger_read,
                &setup.anoncreds,
                &prover_proof_json,
                &proof_req_json,
            )
            .await
            .unwrap_err()
            .kind(),
            AriesVcxErrorKind::InvalidProof
        );
    }
    Ok(())
}

#[tokio::test]
#[ignore]
async fn test_pool_prover_verify_proof() -> Result<(), Box<dyn Error>> {
    let setup = build_setup_profile().await;
    let (schemas, cred_defs, proof_req, proof) = create_indy_proof(
        &setup.wallet,
        &setup.wallet,
        &setup.anoncreds,
        &setup.anoncreds,
        &setup.ledger_read,
        &setup.ledger_write,
        &setup.institution_did,
    )
    .await?;

    let anoncreds = &setup.anoncreds;
    let proof_validation = anoncreds
        .verifier_verify_proof(&proof_req, &proof, schemas, cred_defs, None, None)
        .await?;

    assert!(proof_validation);
    Ok(())
}

#[tokio::test]
#[ignore]
async fn test_pool_prover_verify_proof_with_predicate_success_case() -> Result<(), Box<dyn Error>> {
    let setup = build_setup_profile().await;
    let (schemas, cred_defs, proof_req, proof) = create_proof_with_predicate(
        &setup.wallet,
        &setup.wallet,
        &setup.anoncreds,
        &setup.anoncreds,
        &setup.ledger_read,
        &setup.ledger_write,
        &setup.institution_did,
        true,
    )
    .await?;

    let anoncreds = &setup.anoncreds;
    let proof_validation = anoncreds
        .verifier_verify_proof(&proof_req, &proof, schemas, cred_defs, None, None)
        .await?;

    assert!(proof_validation);
    Ok(())
}

#[tokio::test]
#[ignore]
async fn test_pool_prover_verify_proof_with_predicate_fail_case() -> Result<(), Box<dyn Error>> {
    let setup = build_setup_profile().await;
    let (schemas, cred_defs, proof_req, proof) = create_proof_with_predicate(
        &setup.wallet,
        &setup.wallet,
        &setup.anoncreds,
        &setup.anoncreds,
        &setup.ledger_read,
        &setup.ledger_write,
        &setup.institution_did,
        false,
    )
    .await?;

    let anoncreds = &setup.anoncreds;
    anoncreds
        .verifier_verify_proof(&proof_req, &proof, schemas, cred_defs, None, None)
        .await
        .unwrap_err();
    Ok(())
}
