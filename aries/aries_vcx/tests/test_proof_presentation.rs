#![allow(clippy::diverging_sub_expression)]

use std::error::Error;

use anoncreds_types::data_types::messages::pres_request::PresentationRequest;
use aries_vcx::{
    handlers::proof_presentation::{prover::Prover, verifier::Verifier},
    protocols::proof_presentation::{
        prover::state_machine::ProverState,
        verifier::{
            state_machine::VerifierState, verification_status::PresentationVerificationStatus,
        },
    },
};
use messages::{
    msg_fields::protocols::present_proof::{v1::PresentProofV1, PresentProof},
    AriesMessage,
};
use serde_json::json;
use test_utils::{
    constants::DEFAULT_SCHEMA_ATTRS,
    devsetup::{build_setup_profile, SetupPoolDirectory},
};

use crate::utils::{
    create_and_publish_test_rev_reg, create_and_write_credential, create_and_write_test_cred_def,
    create_and_write_test_schema,
    scenarios::{
        accept_proof_proposal, create_address_schema_creddef_revreg, create_proof_proposal,
        exchange_credential_with_proposal, generate_and_send_proof, prover_select_credentials,
        receive_proof_proposal_rejection, reject_proof_proposal, verify_proof,
    },
    test_agent::{create_test_agent, create_test_agent_trustee},
};

pub mod utils;

#[tokio::test]
#[ignore]
async fn test_agency_pool_generate_proof_with_predicates() -> Result<(), Box<dyn Error>> {
    let setup = build_setup_profile().await;
    let schema = create_and_write_test_schema(
        &setup.wallet,
        &setup.anoncreds,
        &setup.ledger_write,
        &setup.institution_did,
        DEFAULT_SCHEMA_ATTRS,
    )
    .await;
    let cred_def = create_and_write_test_cred_def(
        &setup.wallet,
        &setup.anoncreds,
        &setup.ledger_read,
        &setup.ledger_write,
        &setup.institution_did,
        &schema.schema_id,
        true,
    )
    .await;
    let rev_reg = create_and_publish_test_rev_reg(
        &setup.wallet,
        &setup.anoncreds,
        &setup.ledger_write,
        &setup.institution_did,
        cred_def.get_cred_def_id(),
    )
    .await;
    let _cred_id = create_and_write_credential(
        &setup.wallet,
        &setup.wallet,
        &setup.anoncreds,
        &setup.anoncreds,
        &setup.institution_did,
        &schema,
        &cred_def,
        Some(&rev_reg),
    )
    .await;

    let to = time::OffsetDateTime::now_utc().unix_timestamp() as u64;
    let indy_proof_req = json!({
            "nonce": "123432421212",
            "name": "proof_req_1",
            "version": "1.0",
            "requested_attributes": {
                "address1_1": {
                    "name": "address1",
                    "restrictions": [{"issuer_did": "abcdef0000000000000000"}, {"issuer_did": setup.institution_did}],
                    "non_revoked":  {"from": 123, "to": to}
                },
                "state_2": {
                    "name": "state",
                    "restrictions": {
                        "issuer_did": setup.institution_did,
                        "schema_id": schema.schema_id,
                        "cred_def_id": cred_def.get_cred_def_id(),
                    }
                },
                "zip_self_attested_3": {
                    "name":"zip",
                }
            },
            "requested_predicates": json!({
                "zip_3": {"name":"zip", "p_type":">=", "p_value":18}
            }),
            "non_revoked": {"from": 98, "to": to}
        })
        .to_string();

    let pres_req_data: PresentationRequest = serde_json::from_str(&indy_proof_req)?;
    let mut verifier = Verifier::create_from_request("1".to_string(), &pres_req_data)?;
    let proof_req = verifier.get_presentation_request_msg()?;
    verifier.mark_presentation_request_sent()?;

    let mut proof: Prover = Prover::create_from_request("1", proof_req)?;

    let all_creds = proof
        .retrieve_credentials(&setup.wallet, &setup.anoncreds)
        .await?;
    let selected_credentials: serde_json::Value = json!({
       "attrs":{
          "address1_1": {
            "credential": all_creds.credentials_by_referent["address1_1"][0],
            "tails_dir": rev_reg.get_tails_dir()
          },
          "state_2": {
            "credential": all_creds.credentials_by_referent["state_2"][0],
            "tails_dir": rev_reg.get_tails_dir()
          },
          "zip_3": {
            "credential": all_creds.credentials_by_referent["zip_3"][0],
            "tails_dir": rev_reg.get_tails_dir()
          },
       },
    });
    let self_attested: serde_json::Value = json!({
          "zip_self_attested_3":"attested_val"
    });
    proof
        .generate_presentation(
            &setup.wallet,
            &setup.ledger_read,
            &setup.anoncreds,
            serde_json::from_value(selected_credentials)?,
            serde_json::from_value(self_attested)?,
        )
        .await?;
    assert!(matches!(
        proof.get_state(),
        ProverState::PresentationPrepared
    ));

    let final_message = verifier
        .verify_presentation(
            &setup.ledger_read,
            &setup.anoncreds,
            proof.get_presentation_msg()?,
        )
        .await?;

    if let AriesMessage::PresentProof(PresentProof::V1(PresentProofV1::Ack(_))) = final_message {
        assert_eq!(verifier.get_state(), VerifierState::Finished);
        assert_eq!(
            verifier.get_verification_status(),
            PresentationVerificationStatus::Valid
        );
    } else {
        panic!("Unexpected message type {:?}", final_message);
    }
    Ok(())
}

#[tokio::test]
#[ignore]
#[allow(unused_mut)]
async fn test_agency_pool_presentation_via_proposal() -> Result<(), Box<dyn Error>> {
    let setup = build_setup_profile().await;
    let mut institution = create_test_agent_trustee(setup.genesis_file_path.clone()).await;
    let mut consumer = create_test_agent(setup.genesis_file_path.clone()).await;

    let (schema, cred_def, rev_reg) = create_address_schema_creddef_revreg(
        &institution.wallet,
        &institution.ledger_read,
        &institution.ledger_write,
        &institution.anoncreds,
        &institution.institution_did,
    )
    .await;
    let tails_dir = rev_reg.get_tails_dir();

    exchange_credential_with_proposal(
        &mut consumer,
        &mut institution,
        &schema.schema_id,
        cred_def.get_cred_def_id(),
        Some(rev_reg.rev_reg_id),
        Some(tails_dir),
        "comment",
    )
    .await;
    let mut prover = Prover::create("1")?;
    let mut verifier = Verifier::create("1")?;
    let presentation_proposal =
        create_proof_proposal(&mut prover, cred_def.get_cred_def_id()).await;
    let presentation_request =
        accept_proof_proposal(&mut institution, &mut verifier, presentation_proposal).await;

    let selected_credentials =
        prover_select_credentials(&mut prover, &mut consumer, presentation_request, None).await;
    let presentation = generate_and_send_proof(&mut consumer, &mut prover, selected_credentials)
        .await
        .unwrap();
    verify_proof(&mut institution, &mut verifier, presentation).await;
    Ok(())
}

#[tokio::test]
#[ignore]
#[allow(unused_mut)]
async fn test_agency_pool_presentation_via_proposal_with_rejection() -> Result<(), Box<dyn Error>> {
    let setup = SetupPoolDirectory::init().await;
    let mut institution = create_test_agent_trustee(setup.genesis_file_path.clone()).await;
    let mut consumer = create_test_agent(setup.genesis_file_path.clone()).await;

    let (schema, cred_def, rev_reg) = create_address_schema_creddef_revreg(
        &institution.wallet,
        &institution.ledger_read,
        &institution.ledger_write,
        &institution.anoncreds,
        &institution.institution_did,
    )
    .await;
    let tails_dir = rev_reg.get_tails_dir();

    exchange_credential_with_proposal(
        &mut consumer,
        &mut institution,
        &schema.schema_id,
        cred_def.get_cred_def_id(),
        Some(rev_reg.rev_reg_id),
        Some(tails_dir),
        "comment",
    )
    .await;
    let mut prover = Prover::create("1")?;
    let presentation_proposal =
        create_proof_proposal(&mut prover, cred_def.get_cred_def_id()).await;
    let rejection = reject_proof_proposal(&presentation_proposal).await;
    receive_proof_proposal_rejection(&mut prover, rejection).await;
    Ok(())
}

#[tokio::test]
#[ignore]
#[allow(unused_mut)]
async fn test_agency_pool_presentation_via_proposal_with_negotiation() -> Result<(), Box<dyn Error>>
{
    let setup = SetupPoolDirectory::init().await;
    let mut institution = create_test_agent_trustee(setup.genesis_file_path.clone()).await;
    let mut consumer = create_test_agent(setup.genesis_file_path.clone()).await;

    let (schema, cred_def, rev_reg) = create_address_schema_creddef_revreg(
        &institution.wallet,
        &institution.ledger_read,
        &institution.ledger_write,
        &institution.anoncreds,
        &institution.institution_did,
    )
    .await;
    let tails_dir = rev_reg.get_tails_dir();

    exchange_credential_with_proposal(
        &mut consumer,
        &mut institution,
        &schema.schema_id,
        cred_def.get_cred_def_id(),
        Some(rev_reg.rev_reg_id),
        Some(tails_dir),
        "comment",
    )
    .await;
    let mut prover = Prover::create("1")?;
    let mut verifier = Verifier::create("1")?;

    let presentation_proposal =
        create_proof_proposal(&mut prover, &cred_def.get_cred_def_id().to_owned()).await;
    let presentation_request =
        accept_proof_proposal(&mut institution, &mut verifier, presentation_proposal).await;
    let selected_credentials =
        prover_select_credentials(&mut prover, &mut consumer, presentation_request, None).await;
    let presentation = generate_and_send_proof(&mut consumer, &mut prover, selected_credentials)
        .await
        .unwrap();
    verify_proof(&mut institution, &mut verifier, presentation).await;
    Ok(())
}
