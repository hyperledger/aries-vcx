#[macro_use]
extern crate log;

pub mod utils;

use aries_vcx::{
    protocols::{
        issuance::issuer::state_machine::IssuerState,
        proof_presentation::verifier::verification_status::PresentationVerificationStatus,
    },
    utils::devsetup::*,
};

use crate::utils::{
    scenarios::{
        accept_credential_proposal, accept_offer, create_address_schema_creddef_revreg,
        create_credential_proposal, create_holder_from_proposal, create_issuer_from_proposal,
        credential_data_address_1, decline_offer, exchange_credential,
        exchange_credential_with_proposal, exchange_proof, send_credential,
    },
    test_agent::{create_test_agent, create_test_agent_trustee},
};

#[tokio::test]
#[ignore]
async fn test_agency_pool_double_issuance_issuer_is_verifier() {
    SetupPoolDirectory::run(|setup| async move {
        let mut institution = create_test_agent_trustee(setup.genesis_file_path.clone()).await;
        let mut consumer = create_test_agent(setup.genesis_file_path.clone()).await;

        let (schema, cred_def, rev_reg) = create_address_schema_creddef_revreg(
            &institution.profile,
            &institution.institution_did,
        )
        .await;
        let _credential_handle = exchange_credential(
            &mut consumer,
            &mut institution,
            credential_data_address_1().to_string(),
            &cred_def,
            &rev_reg,
            None,
        )
        .await;

        let verifier = exchange_proof(
            &mut institution,
            &mut consumer,
            &schema.schema_id,
            &cred_def.get_cred_def_id(),
            Some("request1"),
        )
        .await;
        assert_eq!(
            verifier.get_verification_status(),
            PresentationVerificationStatus::Valid
        );

        let verifier = exchange_proof(
            &mut institution,
            &mut consumer,
            &schema.schema_id,
            &cred_def.get_cred_def_id(),
            Some("request2"),
        )
        .await;
        assert_eq!(
            verifier.get_verification_status(),
            PresentationVerificationStatus::Valid
        );
    })
    .await;
}

#[tokio::test]
#[ignore]
#[allow(unused_mut)]
async fn test_agency_pool_two_creds_one_rev_reg() {
    SetupPoolDirectory::run(|setup| async move {
        let mut issuer = create_test_agent_trustee(setup.genesis_file_path.clone()).await;
        let mut verifier = create_test_agent_trustee(setup.genesis_file_path.clone()).await;
        let mut consumer = create_test_agent(setup.genesis_file_path).await;

        let (schema, cred_def, rev_reg) =
            create_address_schema_creddef_revreg(&issuer.profile, &issuer.institution_did).await;
        let credential_data1 = credential_data_address_1().to_string();
        let _credential_handle1 = exchange_credential(
            &mut consumer,
            &mut issuer,
            credential_data1.clone(),
            &cred_def,
            &rev_reg,
            Some("request1"),
        )
        .await;

        let _credential_handle2 = exchange_credential(
            &mut consumer,
            &mut issuer,
            credential_data_address_1().to_string(),
            &cred_def,
            &rev_reg,
            Some("request2"),
        )
        .await;

        let verifier_handler = exchange_proof(
            &mut verifier,
            &mut consumer,
            &schema.schema_id,
            &cred_def.get_cred_def_id(),
            Some("request1"),
        )
        .await;
        assert_eq!(
            verifier_handler.get_verification_status(),
            PresentationVerificationStatus::Valid
        );

        let verifier_handler = exchange_proof(
            &mut verifier,
            &mut consumer,
            &schema.schema_id,
            &cred_def.get_cred_def_id(),
            Some("request2"),
        )
        .await;
        assert_eq!(
            verifier_handler.get_verification_status(),
            PresentationVerificationStatus::Valid
        );
    })
    .await;
}

#[tokio::test]
#[ignore]
#[allow(unused_mut)]
async fn test_agency_pool_credential_exchange_via_proposal() {
    SetupPoolDirectory::run(|setup| async move {
        let mut institution = create_test_agent_trustee(setup.genesis_file_path.clone()).await;
        let mut consumer = create_test_agent(setup.genesis_file_path).await;

        let (schema, cred_def, rev_reg) = create_address_schema_creddef_revreg(
            &institution.profile,
            &institution.institution_did,
        )
        .await;

        exchange_credential_with_proposal(
            &mut consumer,
            &mut institution,
            &schema.schema_id,
            &cred_def.get_cred_def_id(),
            Some(rev_reg.rev_reg_id.clone()),
            Some(rev_reg.get_tails_dir()),
            "comment",
        )
        .await;
    })
    .await;
}

#[tokio::test]
#[ignore]
#[allow(unused_mut)]
async fn test_agency_pool_credential_exchange_via_proposal_failed() {
    SetupPoolDirectory::run(|setup| async move {
        let mut institution = create_test_agent_trustee(setup.genesis_file_path.clone()).await;
        let mut consumer = create_test_agent(setup.genesis_file_path.clone()).await;

        let (schema, cred_def, rev_reg) = create_address_schema_creddef_revreg(
            &institution.profile,
            &institution.institution_did,
        )
        .await;

        let cred_proposal =
            create_credential_proposal(&schema.schema_id, &cred_def.get_cred_def_id(), "comment");
        let mut holder = create_holder_from_proposal(cred_proposal.clone());
        let mut issuer = create_issuer_from_proposal(cred_proposal.clone());

        let cred_offer = accept_credential_proposal(
            &mut institution,
            &mut issuer,
            cred_proposal,
            Some(rev_reg.rev_reg_id.clone()),
            Some(rev_reg.get_tails_dir()),
        )
        .await;
        let problem_report = decline_offer(&mut consumer, cred_offer, &mut holder).await;
        assert_eq!(IssuerState::OfferSet, issuer.get_state());
        issuer
            .process_aries_msg(problem_report.into())
            .await
            .unwrap();
        assert_eq!(IssuerState::Failed, issuer.get_state());
    })
    .await;
}

// TODO: Maybe duplicates test_agency_pool_credential_exchange_via_proposal
#[tokio::test]
#[ignore]
#[allow(unused_mut)]
async fn test_agency_pool_credential_exchange_via_proposal_with_negotiation() {
    SetupPoolDirectory::run(|setup| async move {
        let mut institution = create_test_agent_trustee(setup.genesis_file_path.clone()).await;
        let mut consumer = create_test_agent(setup.genesis_file_path.clone()).await;

        let (schema, cred_def, rev_reg) = create_address_schema_creddef_revreg(
            &institution.profile,
            &institution.institution_did,
        )
        .await;

        let cred_proposal =
            create_credential_proposal(&schema.schema_id, &cred_def.get_cred_def_id(), "comment");
        let mut holder = create_holder_from_proposal(cred_proposal.clone());
        let mut issuer = create_issuer_from_proposal(cred_proposal.clone());

        let cred_proposal_1 =
            create_credential_proposal(&schema.schema_id, &cred_def.get_cred_def_id(), "comment");
        let cred_offer_1 = accept_credential_proposal(
            &mut institution,
            &mut issuer,
            cred_proposal_1,
            Some(rev_reg.rev_reg_id.clone()),
            Some(rev_reg.get_tails_dir()),
        )
        .await;

        let cred_request = accept_offer(&mut consumer, cred_offer_1, &mut holder).await;

        send_credential(
            &mut consumer,
            &mut institution,
            &mut issuer,
            &mut holder,
            cred_request,
            true,
        )
        .await;
    })
    .await;
}
