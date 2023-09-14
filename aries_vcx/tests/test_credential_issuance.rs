#[macro_use]
extern crate log;
#[macro_use]
extern crate serde_json;

pub mod utils;

use aries_vcx::protocols::issuance::issuer::state_machine::IssuerState;
use aries_vcx::protocols::proof_presentation::verifier::verification_status::PresentationVerificationStatus;
use aries_vcx::utils::devsetup::*;

#[cfg(feature = "migration")]
use crate::utils::migration::Migratable;
use crate::utils::scenarios::{
    accept_credential_proposal, accept_offer, attr_names, create_address_schema_creddef_revreg,
    create_credential_proposal, create_holder_from_proposal, create_issuer_from_proposal, decline_offer,
    exchange_credential, exchange_credential_with_proposal, exchange_proof, send_credential,
};
use crate::utils::test_agent::{create_test_agent, create_test_agent_trustee};

#[tokio::test]
#[ignore]
async fn test_agency_pool_double_issuance_issuer_is_verifier() {
    SetupPoolDirectory::run(|setup| async move {
        let mut institution = create_test_agent_trustee(setup.genesis_file_path.clone()).await;
        let mut consumer = create_test_agent(setup.genesis_file_path.clone()).await;

        let (schema_id, _schema_json, cred_def_id, _cred_def_json, cred_def, rev_reg, _rev_reg_id) =
            create_address_schema_creddef_revreg(&institution.profile, &institution.institution_did).await;
        let (address1, address, city, state, zip) = attr_names();
        let credential_data = json!({address1.clone(): "5th Avenue", address.clone(): "Suite 1234", city.clone(): "NYC", state.clone(): "NYS", zip.clone(): "84712"}).to_string();
        let _credential_handle = exchange_credential(
            &mut consumer,
            &mut institution,
            credential_data,
            &cred_def,
            &rev_reg,
            None,
        )
            .await;

        // NOTE: Credx-anoncreds-implementation-generated presentation is not compatible with vdrtools anoncreds implementation
        // as the presentation fails to deserialize
        // #[cfg(feature = "migration")]
        // consumer.migrate().await;

        let verifier_handler =
            exchange_proof(&mut institution, &mut consumer, &schema_id, &cred_def_id, Some("request1")).await;
        assert_eq!(
            verifier_handler.get_verification_status(),
            PresentationVerificationStatus::Valid
        );

        #[cfg(feature = "migration")]
        institution.migrate().await;

        let verifier_handler =
            exchange_proof(&mut institution, &mut consumer, &schema_id, &cred_def_id, Some("request2")).await;
        assert_eq!(
            verifier_handler.get_verification_status(),
            PresentationVerificationStatus::Valid
        );

    }).await;
}

#[tokio::test]
#[ignore]
async fn test_agency_pool_two_creds_one_rev_reg() {
    SetupPoolDirectory::run(|setup| async move {
        let mut issuer = create_test_agent_trustee(setup.genesis_file_path.clone()).await;
        let mut verifier = create_test_agent_trustee(setup.genesis_file_path.clone()).await;
        let mut consumer = create_test_agent(setup.genesis_file_path).await;

        let (schema_id, _schema_json, cred_def_id, _cred_def_json, cred_def, rev_reg, _rev_reg_id) =
            create_address_schema_creddef_revreg(&issuer.profile, &issuer.institution_did).await;
        let (address1, address2, city, state, zip) = attr_names();
        let (req1, req2) = (Some("request1"), Some("request2"));
        let credential_data1 = json!({address1.clone(): "123 Main St", address2.clone(): "Suite 3", city.clone(): "Draper", state.clone(): "UT", zip.clone(): "84000"}).to_string();
        let _credential_handle1 = exchange_credential(
            &mut consumer,
            &mut issuer,
            credential_data1.clone(),
            &cred_def,
            &rev_reg,
            req1,
        )
            .await;

        #[cfg(feature = "migration")]
        issuer.migrate().await;

        let credential_data2 = json!({address1.clone(): "101 Tela Lane", address2.clone(): "Suite 1", city.clone(): "SLC", state.clone(): "WA", zip.clone(): "8721"}).to_string();
        let _credential_handle2 = exchange_credential(
            &mut consumer,
            &mut issuer,
            credential_data2.clone(),
            &cred_def,
            &rev_reg,
            req2,
        )
            .await;

        #[cfg(feature = "migration")]
        verifier.migrate().await;

        let verifier_handler =
            exchange_proof(&mut verifier, &mut consumer, &schema_id, &cred_def_id, req1).await;
        assert_eq!(
            verifier_handler.get_verification_status(),
            PresentationVerificationStatus::Valid
        );

        #[cfg(feature = "migration")]
        consumer.migrate().await;

        let verifier_handler =
            exchange_proof(&mut verifier, &mut consumer, &schema_id, &cred_def_id, req2).await;
        assert_eq!(
            verifier_handler.get_verification_status(),
            PresentationVerificationStatus::Valid
        );

    }).await;
}

#[tokio::test]
#[ignore]
async fn test_agency_pool_credential_exchange_via_proposal() {
    SetupPoolDirectory::run(|setup| async move {
        let mut institution = create_test_agent_trustee(setup.genesis_file_path.clone()).await;
        let mut consumer = create_test_agent(setup.genesis_file_path).await;

        let (schema_id, _schema_json, cred_def_id, _cred_def_json, _cred_def, rev_reg, rev_reg_id) =
            create_address_schema_creddef_revreg(&institution.profile, &institution.institution_did).await;
        let tails_dir = rev_reg.get_tails_dir();

        #[cfg(feature = "migration")]
        institution.migrate().await;

        exchange_credential_with_proposal(
            &mut consumer,
            &mut institution,
            &schema_id,
            &cred_def_id,
            rev_reg_id,
            Some(tails_dir),
            "comment",
        )
        .await;
    })
    .await;
}

#[tokio::test]
#[ignore]
async fn test_agency_pool_credential_exchange_via_proposal_failed() {
    SetupPoolDirectory::run(|setup| async move {
        let mut institution = create_test_agent_trustee(setup.genesis_file_path.clone()).await;
        let mut consumer = create_test_agent(setup.genesis_file_path.clone()).await;

        let (schema_id, _schema_json, cred_def_id, _cred_def_json, _cred_def, rev_reg, rev_reg_id) =
            create_address_schema_creddef_revreg(&institution.profile, &institution.institution_did).await;
        let tails_dir = rev_reg.get_tails_dir();

        let cred_proposal = create_credential_proposal(&schema_id, &cred_def_id, "comment").await;
        let mut holder = create_holder_from_proposal(cred_proposal.clone());
        let mut issuer = create_issuer_from_proposal(cred_proposal.clone());

        #[cfg(feature = "migration")]
        institution.migrate().await;

        let cred_offer = accept_credential_proposal(
            &mut institution,
            &mut issuer,
            cred_proposal,
            rev_reg_id,
            Some(tails_dir),
        )
        .await;
        let problem_report = decline_offer(&mut consumer, cred_offer, &mut holder).await;
        assert_eq!(IssuerState::OfferSet, issuer.get_state());
        issuer.process_aries_msg(problem_report).await.unwrap();
        assert_eq!(IssuerState::Failed, issuer.get_state());
    })
    .await;
}

#[tokio::test]
#[ignore]
async fn test_agency_pool_credential_exchange_via_proposal_with_negotiation() {
    SetupPoolDirectory::run(|setup| async move {
        let mut institution = create_test_agent_trustee(setup.genesis_file_path.clone()).await;
        let mut consumer = create_test_agent(setup.genesis_file_path.clone()).await;

        let (schema_id, _schema_json, cred_def_id, _cred_def_json, _cred_def, rev_reg, rev_reg_id) =
            create_address_schema_creddef_revreg(&institution.profile, &institution.institution_did).await;
        let tails_dir = rev_reg.get_tails_dir();

        #[cfg(feature = "migration")]
        institution.migrate().await;

        let cred_proposal = create_credential_proposal(&schema_id, &cred_def_id, "comment").await;
        let mut holder = create_holder_from_proposal(cred_proposal.clone());
        let mut issuer = create_issuer_from_proposal(cred_proposal.clone());

        #[cfg(feature = "migration")]
        consumer.migrate().await;

        let cred_proposal_1 = create_credential_proposal(&schema_id, &cred_def_id, "comment").await;
        let cred_offer_1 = accept_credential_proposal(
            &mut institution,
            &mut issuer,
            cred_proposal_1,
            rev_reg_id.clone(),
            Some(tails_dir.clone()),
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
