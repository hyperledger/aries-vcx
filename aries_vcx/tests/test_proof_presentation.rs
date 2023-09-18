#[macro_use]
extern crate log;
#[macro_use]
extern crate serde_json;

pub mod utils;

use aries_vcx::common::proofs::proof_request::PresentationRequestData;
use aries_vcx::common::test_utils::{
    create_and_write_credential, create_and_write_test_cred_def, create_and_write_test_rev_reg,
    create_and_write_test_schema,
};
use aries_vcx::handlers::proof_presentation::prover::Prover;
use aries_vcx::handlers::proof_presentation::verifier::Verifier;
use aries_vcx::protocols::proof_presentation::prover::state_machine::ProverState;
use aries_vcx::protocols::proof_presentation::verifier::state_machine::VerifierState;
use aries_vcx::protocols::proof_presentation::verifier::verification_status::PresentationVerificationStatus;
use aries_vcx::utils::devsetup::SetupProfile;
use aries_vcx::utils::devsetup::*;

use messages::msg_fields::protocols::present_proof::PresentProof;
use messages::AriesMessage;

#[cfg(feature = "migration")]
use crate::utils::migration::Migratable;
use crate::utils::scenarios::{
    accept_proof_proposal, create_address_schema_creddef_revreg, create_proof_proposal,
    exchange_credential_with_proposal, generate_and_send_proof, prover_select_credentials,
    receive_proof_proposal_rejection, reject_proof_proposal, verify_proof,
};
use crate::utils::test_agent::{create_test_agent, create_test_agent_trustee};

#[tokio::test]
#[ignore]
async fn test_agency_pool_generate_proof_with_predicates() {
    SetupProfile::run(|mut setup| async move {
        let schema = create_and_write_test_schema(
            &setup.profile.inject_anoncreds(),
            &setup.profile.inject_anoncreds_ledger_write(),
            &setup.institution_did,
            aries_vcx::utils::constants::DEFAULT_SCHEMA_ATTRS,
        )
        .await;
        let cred_def = create_and_write_test_cred_def(
            &setup.profile.inject_anoncreds(),
            &setup.profile.inject_anoncreds_ledger_read(),
            &setup.profile.inject_anoncreds_ledger_write(),
            &setup.institution_did,
            &schema.schema_id,
            true,
        )
        .await;
        let rev_reg = create_and_write_test_rev_reg(
            &setup.profile.inject_anoncreds(),
            &setup.profile.inject_anoncreds_ledger_write(),
            &setup.institution_did,
            &cred_def.get_cred_def_id(),
        )
        .await;
        let _cred_id = create_and_write_credential(
            &setup.profile.inject_anoncreds(),
            &setup.profile.inject_anoncreds(),
            &setup.profile.inject_anoncreds_ledger_read(),
            &setup.institution_did,
            &cred_def,
            Some(&rev_reg),
        )
        .await;

        let to = time::OffsetDateTime::now_utc().unix_timestamp() as u64;
        let indy_proof_req = json!({
            "nonce": "123432421212",
            "name": "proof_req_1",
            "version": "0.1",
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

        let pres_req_data: PresentationRequestData = serde_json::from_str(&indy_proof_req).unwrap();
        let mut verifier = Verifier::create_from_request("1".to_string(), &pres_req_data).unwrap();
        let proof_req = verifier.get_presentation_request_msg().unwrap();
        verifier.mark_presentation_request_sent().unwrap();

        let mut proof: Prover = Prover::create_from_request("1", proof_req).unwrap();

        #[cfg(feature = "migration")]
        setup.migrate().await;

        let all_creds = proof
            .retrieve_credentials(&setup.profile.inject_anoncreds())
            .await
            .unwrap();
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
                &setup.profile.inject_anoncreds_ledger_read(),
                &setup.profile.inject_anoncreds(),
                serde_json::from_value(selected_credentials).unwrap(),
                serde_json::from_value(self_attested).unwrap(),
            )
            .await
            .unwrap();
        assert!(matches!(proof.get_state(), ProverState::PresentationPrepared));

        let final_message = verifier
            .verify_presentation(
                &setup.profile.inject_anoncreds_ledger_read(),
                &setup.profile.inject_anoncreds(),
                proof.get_presentation_msg().unwrap(),
            )
            .await
            .unwrap();

        if let AriesMessage::PresentProof(PresentProof::Ack(_)) = final_message {
            assert_eq!(verifier.get_state(), VerifierState::Finished);
            assert_eq!(
                verifier.get_verification_status(),
                PresentationVerificationStatus::Valid
            );
        } else {
            panic!("Unexpected message type {:?}", final_message);
        }
    })
    .await;
}

#[tokio::test]
#[ignore]
async fn test_agency_pool_presentation_via_proposal() {
    SetupPoolDirectory::run(|setup| async move {
        let mut institution = create_test_agent_trustee(setup.genesis_file_path.clone()).await;
        let mut consumer = create_test_agent(setup.genesis_file_path.clone()).await;

        let (schema, cred_def, rev_reg) =
            create_address_schema_creddef_revreg(&institution.profile, &institution.institution_did).await;
        let tails_dir = rev_reg.get_tails_dir();

        #[cfg(feature = "migration")]
        institution.migrate().await;

        exchange_credential_with_proposal(
            &mut consumer,
            &mut institution,
            &schema.schema_id,
            &cred_def.get_cred_def_id(),
            Some(rev_reg.rev_reg_id),
            Some(tails_dir),
            "comment",
        )
        .await;
        let mut prover = Prover::create("1").unwrap();
        let mut verifier = Verifier::create("1").unwrap();
        let presentation_proposal = create_proof_proposal(&mut prover, &cred_def.get_cred_def_id()).await;
        let presentation_request = accept_proof_proposal(&mut institution, &mut verifier, presentation_proposal).await;

        #[cfg(feature = "migration")]
        consumer.migrate().await;

        let selected_credentials =
            prover_select_credentials(&mut prover, &mut consumer, presentation_request, None).await;
        let presentation = generate_and_send_proof(&mut consumer, &mut prover, selected_credentials)
            .await
            .unwrap();
        verify_proof(&mut institution, &mut verifier, presentation).await;
    })
    .await;
}

#[tokio::test]
#[ignore]
async fn test_agency_pool_presentation_via_proposal_with_rejection() {
    SetupPoolDirectory::run(|setup| async move {
        let mut institution = create_test_agent_trustee(setup.genesis_file_path.clone()).await;
        let mut consumer = create_test_agent(setup.genesis_file_path.clone()).await;

        let (schema, cred_def, rev_reg) =
            create_address_schema_creddef_revreg(&institution.profile, &institution.institution_did).await;
        let tails_dir = rev_reg.get_tails_dir();

        #[cfg(feature = "migration")]
        institution.migrate().await;

        exchange_credential_with_proposal(
            &mut consumer,
            &mut institution,
            &schema.schema_id,
            &cred_def.get_cred_def_id(),
            Some(rev_reg.rev_reg_id),
            Some(tails_dir),
            "comment",
        )
        .await;
        let mut prover = Prover::create("1").unwrap();
        let presentation_proposal = create_proof_proposal(&mut prover, &cred_def.get_cred_def_id()).await;
        let rejection = reject_proof_proposal(&presentation_proposal).await;
        receive_proof_proposal_rejection(&mut prover, rejection).await;
    })
    .await;
}

#[tokio::test]
#[ignore]
async fn test_agency_pool_presentation_via_proposal_with_negotiation() {
    SetupPoolDirectory::run(|setup| async move {
        let mut institution = create_test_agent_trustee(setup.genesis_file_path.clone()).await;
        let mut consumer = create_test_agent(setup.genesis_file_path.clone()).await;

        let (schema, cred_def, rev_reg) =
            create_address_schema_creddef_revreg(&institution.profile, &institution.institution_did).await;
        let tails_dir = rev_reg.get_tails_dir();

        #[cfg(feature = "migration")]
        institution.migrate().await;

        exchange_credential_with_proposal(
            &mut consumer,
            &mut institution,
            &schema.schema_id,
            &cred_def.get_cred_def_id(),
            Some(rev_reg.rev_reg_id),
            Some(tails_dir),
            "comment",
        )
        .await;
        let mut prover = Prover::create("1").unwrap();
        let mut verifier = Verifier::create("1").unwrap();

        #[cfg(feature = "migration")]
        consumer.migrate().await;

        let presentation_proposal = create_proof_proposal(&mut prover, &cred_def.get_cred_def_id()).await;
        let presentation_request = accept_proof_proposal(&mut institution, &mut verifier, presentation_proposal).await;
        let selected_credentials =
            prover_select_credentials(&mut prover, &mut consumer, presentation_request, None).await;
        let presentation = generate_and_send_proof(&mut consumer, &mut prover, selected_credentials)
            .await
            .unwrap();
        verify_proof(&mut institution, &mut verifier, presentation).await;
    })
    .await;
}
