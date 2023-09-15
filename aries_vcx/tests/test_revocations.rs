#[macro_use]
extern crate log;
#[macro_use]
extern crate serde_json;

pub mod utils;

use std::thread;
use std::time::Duration;

use aries_vcx::protocols::proof_presentation::verifier::state_machine::VerifierState;
use aries_vcx::protocols::proof_presentation::verifier::verification_status::PresentationVerificationStatus;
use aries_vcx::utils::devsetup::*;

#[cfg(feature = "migration")]
use crate::utils::migration::Migratable;
use crate::utils::scenarios::{
    create_address_schema_creddef_revreg, create_proof_request_data, create_verifier_from_request_data,
    credential_data_address_1, credential_data_address_2, credential_data_address_3, exchange_credential,
    exchange_proof, issue_address_credential, prover_select_credentials_and_send_proof, publish_revocation,
    requested_attrs_address, revoke_credential_and_publish_accumulator, revoke_credential_local, rotate_rev_reg,
    verifier_create_proof_and_send_request,
};
use crate::utils::test_agent::{create_test_agent, create_test_agent_trustee};

#[tokio::test]
#[ignore]
async fn test_agency_pool_basic_revocation() {
    SetupPoolDirectory::run(|setup| async move {
        let mut institution = create_test_agent_trustee(setup.genesis_file_path.clone()).await;
        let mut consumer = create_test_agent(setup.genesis_file_path).await;

        let (schema_id, cred_def_id, _, _cred_def, rev_reg, issuer_credential) =
            issue_address_credential(&mut consumer, &mut institution).await;

        #[cfg(feature = "migration")]
        institution.migrate().await;

        assert!(!issuer_credential
            .is_revoked(&institution.profile.inject_anoncreds_ledger_read())
            .await
            .unwrap());

        let time_before_revocation = time::OffsetDateTime::now_utc().unix_timestamp() as u64;
        revoke_credential_and_publish_accumulator(&mut institution, &issuer_credential, &rev_reg).await;

        #[cfg(feature = "migration")]
        consumer.migrate().await;

        tokio::time::sleep(Duration::from_millis(1000)).await;
        let time_after_revocation = time::OffsetDateTime::now_utc().unix_timestamp() as u64;

        assert!(issuer_credential
            .is_revoked(&institution.profile.inject_anoncreds_ledger_read())
            .await
            .unwrap());

        let requested_attrs = requested_attrs_address(
            &institution.institution_did,
            &schema_id,
            &cred_def_id,
            None,
            Some(time_after_revocation),
        );

        let presentation_request_data = create_proof_request_data(
            &mut institution,
            &requested_attrs.to_string(),
            "[]",
            &json!({"from": time_before_revocation - 100, "to": time_after_revocation}).to_string(),
            None,
        )
        .await;

        let mut verifier = create_verifier_from_request_data(presentation_request_data).await;
        let presentation = prover_select_credentials_and_send_proof(
            &mut consumer,
            verifier.get_presentation_request_msg().unwrap(),
            None,
        )
        .await;

        verifier
            .verify_presentation(
                &institution.profile.inject_anoncreds_ledger_read(),
                &institution.profile.inject_anoncreds(),
                presentation,
            )
            .await
            .unwrap();
        assert_eq!(verifier.get_state(), VerifierState::Finished);
        assert_eq!(
            verifier.get_verification_status(),
            PresentationVerificationStatus::Invalid
        );
    })
    .await;
}

#[tokio::test]
#[ignore]
async fn test_agency_pool_revoked_credential_might_still_work() {
    SetupPoolDirectory::run(|setup| async move {
        let mut institution = create_test_agent_trustee(setup.genesis_file_path.clone()).await;
        let mut consumer = create_test_agent(setup.genesis_file_path).await;

        let (schema_id, cred_def_id, _, _cred_def, rev_reg, issuer_credential) =
            issue_address_credential(&mut consumer, &mut institution).await;

        assert!(!issuer_credential
            .is_revoked(&institution.profile.inject_anoncreds_ledger_read())
            .await
            .unwrap());

        #[cfg(feature = "migration")]
        institution.migrate().await;

        tokio::time::sleep(Duration::from_millis(1000)).await;
        let time_before_revocation = time::OffsetDateTime::now_utc().unix_timestamp() as u64;
        tokio::time::sleep(Duration::from_millis(1000)).await;

        revoke_credential_and_publish_accumulator(&mut institution, &issuer_credential, &rev_reg).await;
        tokio::time::sleep(Duration::from_millis(1000)).await;

        #[cfg(feature = "migration")]
        consumer.migrate().await;

        let from = time_before_revocation - 100;
        let to = time_before_revocation;
        let requested_attrs = requested_attrs_address(
            &institution.institution_did,
            &schema_id,
            &cred_def_id,
            Some(from),
            Some(to),
        );

        let presentation_request_data = create_proof_request_data(
            &mut institution,
            &requested_attrs.to_string(),
            "[]",
            &json!({"from": from, "to": to}).to_string(),
            None,
        )
        .await;

        let mut verifier = create_verifier_from_request_data(presentation_request_data).await;
        let presentation = prover_select_credentials_and_send_proof(
            &mut consumer,
            verifier.get_presentation_request_msg().unwrap(),
            None,
        )
        .await;

        verifier
            .verify_presentation(
                &institution.profile.inject_anoncreds_ledger_read(),
                &institution.profile.inject_anoncreds(),
                presentation,
            )
            .await
            .unwrap();
        assert_eq!(verifier.get_state(), VerifierState::Finished);
        assert_eq!(
            verifier.get_verification_status(),
            PresentationVerificationStatus::Valid
        );
    })
    .await;
}

#[tokio::test]
#[ignore]
async fn test_agency_pool_local_revocation() {
    SetupPoolDirectory::run(|setup| async move {
        let mut institution = create_test_agent_trustee(setup.genesis_file_path.clone()).await;
        let mut consumer = create_test_agent(setup.genesis_file_path).await;

        let (schema_id, cred_def_id, _, _cred_def, rev_reg, issuer_credential) =
            issue_address_credential(&mut consumer, &mut institution).await;

        #[cfg(feature = "migration")]
        institution.migrate().await;

        revoke_credential_local(&mut institution, &issuer_credential, &rev_reg.rev_reg_id).await;
        assert!(!issuer_credential
            .is_revoked(&institution.profile.inject_anoncreds_ledger_read())
            .await
            .unwrap());

        let verifier_handler = exchange_proof(
            &mut institution,
            &mut consumer,
            &schema_id,
            &cred_def_id,
            Some("request1"),
        )
        .await;
        assert_eq!(
            verifier_handler.get_verification_status(),
            PresentationVerificationStatus::Valid
        );

        assert!(!issuer_credential
            .is_revoked(&institution.profile.inject_anoncreds_ledger_read())
            .await
            .unwrap());

        publish_revocation(&mut institution, &rev_reg).await;

        let verifier_handler = exchange_proof(
            &mut institution,
            &mut consumer,
            &schema_id,
            &cred_def_id,
            Some("request2"),
        )
        .await;
        assert_eq!(
            verifier_handler.get_verification_status(),
            PresentationVerificationStatus::Invalid
        );

        assert!(issuer_credential
            .is_revoked(&institution.profile.inject_anoncreds_ledger_read())
            .await
            .unwrap());
    })
    .await;
}

#[tokio::test]
#[ignore]
async fn test_agency_batch_revocation() {
    SetupPoolDirectory::run(|setup| async move {
        let mut institution = create_test_agent_trustee(setup.genesis_file_path.clone()).await;
        let mut consumer1 = create_test_agent(setup.genesis_file_path.clone()).await;
        let mut consumer2 = create_test_agent(setup.genesis_file_path.clone()).await;
        let mut consumer3 = create_test_agent(setup.genesis_file_path).await;

        // Issue and send three credentials of the same schema
        let (schema_id, _schema_json, cred_def_id, _cred_def_json, cred_def, rev_reg, _rev_reg_id) =
            create_address_schema_creddef_revreg(&institution.profile, &institution.institution_did).await;

        let issuer_credential1 = exchange_credential(
            &mut consumer1,
            &mut institution,
            credential_data_address_1().to_string(),
            &cred_def,
            &rev_reg,
            None,
        )
        .await;

        #[cfg(feature = "migration")]
        institution.migrate().await;

        let issuer_credential2 = exchange_credential(
            &mut consumer2,
            &mut institution,
            credential_data_address_2().to_string(),
            &cred_def,
            &rev_reg,
            None,
        )
        .await;

        #[cfg(feature = "migration")]
        consumer1.migrate().await;

        let issuer_credential3 = exchange_credential(
            &mut consumer3,
            &mut institution,
            credential_data_address_3().to_string(),
            &cred_def,
            &rev_reg,
            None,
        )
        .await;

        revoke_credential_local(&mut institution, &issuer_credential1, &rev_reg.rev_reg_id).await;
        revoke_credential_local(&mut institution, &issuer_credential2, &rev_reg.rev_reg_id).await;
        assert!(!issuer_credential1
            .is_revoked(&institution.profile.inject_anoncreds_ledger_read())
            .await
            .unwrap());
        assert!(!issuer_credential2
            .is_revoked(&institution.profile.inject_anoncreds_ledger_read())
            .await
            .unwrap());
        assert!(!issuer_credential3
            .is_revoked(&institution.profile.inject_anoncreds_ledger_read())
            .await
            .unwrap());

        #[cfg(feature = "migration")]
        consumer2.migrate().await;

        #[cfg(feature = "migration")]
        consumer3.migrate().await;

        // Revoke two locally and verify their are all still valid
        let verifier_handler = exchange_proof(
            &mut institution,
            &mut consumer1,
            &schema_id,
            &cred_def_id,
            Some("request1"),
        )
        .await;
        assert_eq!(
            verifier_handler.get_verification_status(),
            PresentationVerificationStatus::Valid
        );
        let verifier_handler = exchange_proof(
            &mut institution,
            &mut consumer2,
            &schema_id,
            &cred_def_id,
            Some("request2"),
        )
        .await;
        assert_eq!(
            verifier_handler.get_verification_status(),
            PresentationVerificationStatus::Valid
        );
        let verifier_handler = exchange_proof(
            &mut institution,
            &mut consumer3,
            &schema_id,
            &cred_def_id,
            Some("request3"),
        )
        .await;
        assert_eq!(
            verifier_handler.get_verification_status(),
            PresentationVerificationStatus::Valid
        );

        // Publish revocations and verify the two are invalid, third still valid
        publish_revocation(&mut institution, &rev_reg).await;
        tokio::time::sleep(Duration::from_millis(1000)).await;

        assert!(issuer_credential1
            .is_revoked(&institution.profile.inject_anoncreds_ledger_read())
            .await
            .unwrap());
        assert!(issuer_credential2
            .is_revoked(&institution.profile.inject_anoncreds_ledger_read())
            .await
            .unwrap());
        assert!(!issuer_credential3
            .is_revoked(&institution.profile.inject_anoncreds_ledger_read())
            .await
            .unwrap());

        let verifier_handler = exchange_proof(
            &mut institution,
            &mut consumer1,
            &schema_id,
            &cred_def_id,
            Some("request1"),
        )
        .await;
        assert_eq!(
            verifier_handler.get_verification_status(),
            PresentationVerificationStatus::Invalid
        );
        let verifier_handler = exchange_proof(
            &mut institution,
            &mut consumer2,
            &schema_id,
            &cred_def_id,
            Some("request2"),
        )
        .await;
        assert_eq!(
            verifier_handler.get_verification_status(),
            PresentationVerificationStatus::Invalid
        );
        let verifier_handler = exchange_proof(
            &mut institution,
            &mut consumer3,
            &schema_id,
            &cred_def_id,
            Some("request3"),
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
async fn test_agency_pool_two_creds_one_rev_reg_revoke_first() {
    SetupPoolDirectory::run(|setup| async move {
        let mut issuer = create_test_agent_trustee(setup.genesis_file_path.clone()).await;
        let mut verifier = create_test_agent_trustee(setup.genesis_file_path.clone()).await;
        let mut consumer = create_test_agent(setup.genesis_file_path).await;

        let (schema_id, _schema_json, cred_def_id, _cred_def_json, cred_def, rev_reg, _rev_reg_id) =
            create_address_schema_creddef_revreg(&issuer.profile, &issuer.institution_did).await;
        let credential_data1 = credential_data_address_1().to_string();
        let issuer_credential1 = exchange_credential(
            &mut consumer,
            &mut issuer,
            credential_data1.clone(),
            &cred_def,
            &rev_reg,
            Some("request1"),
        )
        .await;

        #[cfg(feature = "migration")]
        issuer.migrate().await;

        let credential_data2 = credential_data_address_2().to_string();
        let issuer_credential2 = exchange_credential(
            &mut consumer,
            &mut issuer,
            credential_data2.clone(),
            &cred_def,
            &rev_reg,
            Some("request2"),
        )
        .await;

        assert!(!issuer_credential1
            .is_revoked(&issuer.profile.inject_anoncreds_ledger_read())
            .await
            .unwrap());
        assert!(!issuer_credential2
            .is_revoked(&issuer.profile.inject_anoncreds_ledger_read())
            .await
            .unwrap());

        #[cfg(feature = "migration")]
        verifier.migrate().await;

        revoke_credential_and_publish_accumulator(&mut issuer, &issuer_credential1, &rev_reg).await;

        let mut proof_verifier =
            verifier_create_proof_and_send_request(&mut verifier, &schema_id, &cred_def_id, Some("request1")).await;
        let presentation_request = proof_verifier.get_presentation_request_msg().unwrap();
        let presentation =
            prover_select_credentials_and_send_proof(&mut consumer, presentation_request, Some(&credential_data1))
                .await;
        proof_verifier
            .verify_presentation(
                &verifier.profile.inject_anoncreds_ledger_read(),
                &verifier.profile.inject_anoncreds(),
                presentation,
            )
            .await
            .unwrap();
        assert_eq!(proof_verifier.get_state(), VerifierState::Finished);
        assert_eq!(
            proof_verifier.get_verification_status(),
            PresentationVerificationStatus::Invalid
        );

        let mut proof_verifier =
            verifier_create_proof_and_send_request(&mut verifier, &schema_id, &cred_def_id, Some("request2")).await;
        let presentation_request = proof_verifier.get_presentation_request_msg().unwrap();
        let presentation =
            prover_select_credentials_and_send_proof(&mut consumer, presentation_request, Some(&credential_data2))
                .await;

        #[cfg(feature = "migration")]
        consumer.migrate().await;

        proof_verifier
            .verify_presentation(
                &verifier.profile.inject_anoncreds_ledger_read(),
                &verifier.profile.inject_anoncreds(),
                presentation,
            )
            .await
            .unwrap();
        assert_eq!(
            proof_verifier.get_verification_status(),
            PresentationVerificationStatus::Valid
        );

        assert!(issuer_credential1
            .is_revoked(&issuer.profile.inject_anoncreds_ledger_read())
            .await
            .unwrap());
        assert!(!issuer_credential2
            .is_revoked(&issuer.profile.inject_anoncreds_ledger_read())
            .await
            .unwrap());
    })
    .await;
}

#[tokio::test]
#[ignore]
async fn test_agency_pool_two_creds_one_rev_reg_revoke_second() {
    SetupPoolDirectory::run(|setup| async move {
        let mut issuer = create_test_agent_trustee(setup.genesis_file_path.clone()).await;
        let mut verifier = create_test_agent_trustee(setup.genesis_file_path.clone()).await;
        let mut consumer = create_test_agent(setup.genesis_file_path).await;

        let (schema_id, _schema_json, cred_def_id, _cred_def_json, cred_def, rev_reg, _rev_reg_id) =
            create_address_schema_creddef_revreg(&issuer.profile, &issuer.institution_did).await;
        let credential_data1 = credential_data_address_1().to_string();
        let issuer_credential1 = exchange_credential(
            &mut consumer,
            &mut issuer,
            credential_data1.clone(),
            &cred_def,
            &rev_reg,
            Some("request1"),
        )
        .await;

        #[cfg(feature = "migration")]
        issuer.migrate().await;

        let credential_data2 = credential_data_address_2().to_string();
        let issuer_credential2 = exchange_credential(
            &mut consumer,
            &mut issuer,
            credential_data2.clone(),
            &cred_def,
            &rev_reg,
            Some("request2"),
        )
        .await;

        assert!(!issuer_credential1
            .is_revoked(&issuer.profile.inject_anoncreds_ledger_read())
            .await
            .unwrap());
        assert!(!issuer_credential2
            .is_revoked(&issuer.profile.inject_anoncreds_ledger_read())
            .await
            .unwrap());

        #[cfg(feature = "migration")]
        verifier.migrate().await;

        revoke_credential_and_publish_accumulator(&mut issuer, &issuer_credential2, &rev_reg).await;

        let mut proof_verifier =
            verifier_create_proof_and_send_request(&mut verifier, &schema_id, &cred_def_id, Some("request1")).await;
        let presentation = prover_select_credentials_and_send_proof(
            &mut consumer,
            proof_verifier.get_presentation_request_msg().unwrap(),
            Some(&credential_data1),
        )
        .await;
        proof_verifier
            .verify_presentation(
                &verifier.profile.inject_anoncreds_ledger_read(),
                &verifier.profile.inject_anoncreds(),
                presentation,
            )
            .await
            .unwrap();
        assert_eq!(proof_verifier.get_state(), VerifierState::Finished);
        assert_eq!(
            proof_verifier.get_verification_status(),
            PresentationVerificationStatus::Valid
        );

        let mut proof_verifier =
            verifier_create_proof_and_send_request(&mut verifier, &schema_id, &cred_def_id, Some("request2")).await;
        let presentation = prover_select_credentials_and_send_proof(
            &mut consumer,
            proof_verifier.get_presentation_request_msg().unwrap(),
            Some(&credential_data2),
        )
        .await;

        #[cfg(feature = "migration")]
        consumer.migrate().await;

        proof_verifier
            .verify_presentation(
                &verifier.profile.inject_anoncreds_ledger_read(),
                &verifier.profile.inject_anoncreds(),
                presentation,
            )
            .await
            .unwrap();
        assert_eq!(
            proof_verifier.get_verification_status(),
            PresentationVerificationStatus::Invalid
        );

        assert!(!issuer_credential1
            .is_revoked(&issuer.profile.inject_anoncreds_ledger_read())
            .await
            .unwrap());
        assert!(issuer_credential2
            .is_revoked(&issuer.profile.inject_anoncreds_ledger_read())
            .await
            .unwrap());
    })
    .await;
}

#[tokio::test]
#[ignore]
async fn test_agency_pool_two_creds_two_rev_reg_id() {
    SetupPoolDirectory::run(|setup| async move {
        let mut issuer = create_test_agent_trustee(setup.genesis_file_path.clone()).await;
        let mut verifier = create_test_agent_trustee(setup.genesis_file_path.clone()).await;
        let mut consumer = create_test_agent(setup.genesis_file_path).await;

        let (schema_id, _schema_json, cred_def_id, _cred_def_json, cred_def, rev_reg, _) =
            create_address_schema_creddef_revreg(&issuer.profile, &issuer.institution_did).await;
        let credential_data1 = credential_data_address_1().to_string();
        let issuer_credential1 = exchange_credential(
            &mut consumer,
            &mut issuer,
            credential_data1.clone(),
            &cred_def,
            &rev_reg,
            Some("request1"),
        )
        .await;

        #[cfg(feature = "migration")]
        issuer.migrate().await;

        let rev_reg_2 = rotate_rev_reg(&mut issuer, &cred_def, &rev_reg).await;
        let credential_data2 = credential_data_address_2().to_string();
        let issuer_credential2 = exchange_credential(
            &mut consumer,
            &mut issuer,
            credential_data2.clone(),
            &cred_def,
            &rev_reg_2,
            Some("request2"),
        )
        .await;

        let mut proof_verifier =
            verifier_create_proof_and_send_request(&mut verifier, &schema_id, &cred_def_id, Some("request1")).await;
        let presentation = prover_select_credentials_and_send_proof(
            &mut consumer,
            proof_verifier.get_presentation_request_msg().unwrap(),
            Some(&credential_data1),
        )
        .await;
        proof_verifier
            .verify_presentation(
                &verifier.profile.inject_anoncreds_ledger_read(),
                &verifier.profile.inject_anoncreds(),
                presentation,
            )
            .await
            .unwrap();
        assert_eq!(proof_verifier.get_state(), VerifierState::Finished);
        assert_eq!(
            proof_verifier.get_verification_status(),
            PresentationVerificationStatus::Valid
        );

        #[cfg(feature = "migration")]
        verifier.migrate().await;

        let mut proof_verifier =
            verifier_create_proof_and_send_request(&mut verifier, &schema_id, &cred_def_id, Some("request2")).await;

        #[cfg(feature = "migration")]
        consumer.migrate().await;

        let presentation = prover_select_credentials_and_send_proof(
            &mut consumer,
            proof_verifier.get_presentation_request_msg().unwrap(),
            Some(&credential_data2),
        )
        .await;
        proof_verifier
            .verify_presentation(
                &verifier.profile.inject_anoncreds_ledger_read(),
                &verifier.profile.inject_anoncreds(),
                presentation,
            )
            .await
            .unwrap();
        assert_eq!(proof_verifier.get_state(), VerifierState::Finished);
        assert_eq!(
            proof_verifier.get_verification_status(),
            PresentationVerificationStatus::Valid
        );

        assert!(!issuer_credential1
            .is_revoked(&issuer.profile.inject_anoncreds_ledger_read())
            .await
            .unwrap());
        assert!(!issuer_credential2
            .is_revoked(&issuer.profile.inject_anoncreds_ledger_read())
            .await
            .unwrap());
    })
    .await;
}

#[tokio::test]
#[ignore]
async fn test_agency_pool_two_creds_two_rev_reg_id_revoke_first() {
    SetupPoolDirectory::run(|setup| async move {
        let mut issuer = create_test_agent_trustee(setup.genesis_file_path.clone()).await;
        let mut verifier = create_test_agent_trustee(setup.genesis_file_path.clone()).await;
        let mut consumer = create_test_agent(setup.genesis_file_path).await;

        let (schema_id, _schema_json, cred_def_id, _cred_def_json, cred_def, rev_reg, _) =
            create_address_schema_creddef_revreg(&issuer.profile, &issuer.institution_did).await;
        let credential_data1 = credential_data_address_1().to_string();
        let issuer_credential1 = exchange_credential(
            &mut consumer,
            &mut issuer,
            credential_data1.clone(),
            &cred_def,
            &rev_reg,
            Some("request1"),
        )
        .await;

        #[cfg(feature = "migration")]
        issuer.migrate().await;

        let rev_reg_2 = rotate_rev_reg(&mut issuer, &cred_def, &rev_reg).await;
        let credential_data2 = credential_data_address_2().to_string();
        let issuer_credential2 = exchange_credential(
            &mut consumer,
            &mut issuer,
            credential_data2.clone(),
            &cred_def,
            &rev_reg_2,
            Some("request2"),
        )
        .await;

        assert!(!issuer_credential1
            .is_revoked(&issuer.profile.inject_anoncreds_ledger_read())
            .await
            .unwrap());
        assert!(!issuer_credential2
            .is_revoked(&issuer.profile.inject_anoncreds_ledger_read())
            .await
            .unwrap());

        revoke_credential_and_publish_accumulator(&mut issuer, &issuer_credential1, &rev_reg).await;

        #[cfg(feature = "migration")]
        verifier.migrate().await;

        let mut proof_verifier =
            verifier_create_proof_and_send_request(&mut verifier, &schema_id, &cred_def_id, Some("request1")).await;
        let presentation = prover_select_credentials_and_send_proof(
            &mut consumer,
            proof_verifier.get_presentation_request_msg().unwrap(),
            Some(&credential_data1),
        )
        .await;
        proof_verifier
            .verify_presentation(
                &verifier.profile.inject_anoncreds_ledger_read(),
                &verifier.profile.inject_anoncreds(),
                presentation,
            )
            .await
            .unwrap();
        assert_eq!(proof_verifier.get_state(), VerifierState::Finished);
        assert_eq!(
            proof_verifier.get_verification_status(),
            PresentationVerificationStatus::Invalid
        );

        let mut proof_verifier =
            verifier_create_proof_and_send_request(&mut verifier, &schema_id, &cred_def_id, Some("request2")).await;
        let presentation = prover_select_credentials_and_send_proof(
            &mut consumer,
            proof_verifier.get_presentation_request_msg().unwrap(),
            Some(&credential_data2),
        )
        .await;

        #[cfg(feature = "migration")]
        consumer.migrate().await;

        proof_verifier
            .verify_presentation(
                &verifier.profile.inject_anoncreds_ledger_read(),
                &verifier.profile.inject_anoncreds(),
                presentation,
            )
            .await
            .unwrap();
        assert_eq!(
            proof_verifier.get_verification_status(),
            PresentationVerificationStatus::Valid
        );

        assert!(issuer_credential1
            .is_revoked(&issuer.profile.inject_anoncreds_ledger_read())
            .await
            .unwrap());
        assert!(!issuer_credential2
            .is_revoked(&issuer.profile.inject_anoncreds_ledger_read())
            .await
            .unwrap());
    })
    .await;
}

#[tokio::test]
#[ignore]
async fn test_agency_pool_two_creds_two_rev_reg_id_revoke_second() {
    SetupPoolDirectory::run(|setup| async move {
        let mut issuer = create_test_agent_trustee(setup.genesis_file_path.clone()).await;
        let mut verifier = create_test_agent_trustee(setup.genesis_file_path.clone()).await;
        let mut consumer = create_test_agent(setup.genesis_file_path).await;

        let (schema_id, _schema_json, cred_def_id, _cred_def_json, cred_def, rev_reg, _) =
            create_address_schema_creddef_revreg(&issuer.profile, &issuer.institution_did).await;
        let credential_data1 = credential_data_address_1().to_string();
        let issuer_credential1 = exchange_credential(
            &mut consumer,
            &mut issuer,
            credential_data1.clone(),
            &cred_def,
            &rev_reg,
            Some("request1"),
        )
        .await;

        #[cfg(feature = "migration")]
        issuer.migrate().await;

        let rev_reg_2 = rotate_rev_reg(&mut issuer, &cred_def, &rev_reg).await;
        let credential_data2 = credential_data_address_2().to_string();
        let issuer_credential2 = exchange_credential(
            &mut consumer,
            &mut issuer,
            credential_data2.clone(),
            &cred_def,
            &rev_reg_2,
            Some("request2"),
        )
        .await;

        assert!(!issuer_credential1
            .is_revoked(&issuer.profile.inject_anoncreds_ledger_read())
            .await
            .unwrap());
        assert!(!issuer_credential2
            .is_revoked(&issuer.profile.inject_anoncreds_ledger_read())
            .await
            .unwrap());

        revoke_credential_and_publish_accumulator(&mut issuer, &issuer_credential2, &rev_reg_2).await;

        let mut proof_verifier =
            verifier_create_proof_and_send_request(&mut verifier, &schema_id, &cred_def_id, Some("request1")).await;
        let presentation = prover_select_credentials_and_send_proof(
            &mut consumer,
            proof_verifier.get_presentation_request_msg().unwrap(),
            Some(&credential_data1),
        )
        .await;

        #[cfg(feature = "migration")]
        verifier.migrate().await;

        proof_verifier
            .verify_presentation(
                &verifier.profile.inject_anoncreds_ledger_read(),
                &verifier.profile.inject_anoncreds(),
                presentation,
            )
            .await
            .unwrap();
        assert_eq!(proof_verifier.get_state(), VerifierState::Finished);
        assert_eq!(
            proof_verifier.get_verification_status(),
            PresentationVerificationStatus::Valid
        );

        let mut proof_verifier =
            verifier_create_proof_and_send_request(&mut verifier, &schema_id, &cred_def_id, Some("request2")).await;
        let presentation = prover_select_credentials_and_send_proof(
            &mut consumer,
            proof_verifier.get_presentation_request_msg().unwrap(),
            Some(&credential_data2),
        )
        .await;

        #[cfg(feature = "migration")]
        consumer.migrate().await;

        proof_verifier
            .verify_presentation(
                &verifier.profile.inject_anoncreds_ledger_read(),
                &verifier.profile.inject_anoncreds(),
                presentation,
            )
            .await
            .unwrap();
        assert_eq!(proof_verifier.get_state(), VerifierState::Finished);
        assert_eq!(
            proof_verifier.get_verification_status(),
            PresentationVerificationStatus::Invalid
        );

        assert!(!issuer_credential1
            .is_revoked(&issuer.profile.inject_anoncreds_ledger_read())
            .await
            .unwrap());
        assert!(issuer_credential2
            .is_revoked(&issuer.profile.inject_anoncreds_ledger_read())
            .await
            .unwrap());
    })
    .await;
}

#[tokio::test]
#[ignore]
async fn test_agency_pool_three_creds_one_rev_reg_revoke_all() {
    SetupPoolDirectory::run(|setup| async move {
        let mut issuer = create_test_agent_trustee(setup.genesis_file_path.clone()).await;
        let mut consumer = create_test_agent(setup.genesis_file_path.clone()).await;

        let (_schema_id, _schema_json, _cred_def_id, _cred_def_json, cred_def, rev_reg, _rev_reg_id) =
            create_address_schema_creddef_revreg(&issuer.profile, &issuer.institution_did).await;

        let issuer_credential1 = exchange_credential(
            &mut consumer,
            &mut issuer,
            credential_data_address_1().to_string(),
            &cred_def,
            &rev_reg,
            Some("request1"),
        )
        .await;

        assert!(!issuer_credential1
            .is_revoked(&issuer.profile.inject_anoncreds_ledger_read())
            .await
            .unwrap());
        revoke_credential_local(&mut issuer, &issuer_credential1, &rev_reg.rev_reg_id).await;

        let issuer_credential2 = exchange_credential(
            &mut consumer,
            &mut issuer,
            credential_data_address_2().to_string(),
            &cred_def,
            &rev_reg,
            Some("request2"),
        )
        .await;

        assert!(!issuer_credential2
            .is_revoked(&issuer.profile.inject_anoncreds_ledger_read())
            .await
            .unwrap());

        #[cfg(feature = "migration")]
        issuer.migrate().await;

        #[cfg(feature = "migration")]
        consumer.migrate().await;

        revoke_credential_local(&mut issuer, &issuer_credential2, &rev_reg.rev_reg_id).await;

        let issuer_credential3 = exchange_credential(
            &mut consumer,
            &mut issuer,
            credential_data_address_3().to_string(),
            &cred_def,
            &rev_reg,
            Some("request3"),
        )
        .await;

        revoke_credential_and_publish_accumulator(&mut issuer, &issuer_credential3, &rev_reg).await;
        thread::sleep(Duration::from_millis(100));

        assert!(issuer_credential1
            .is_revoked(&issuer.profile.inject_anoncreds_ledger_read())
            .await
            .unwrap());
        assert!(issuer_credential2
            .is_revoked(&issuer.profile.inject_anoncreds_ledger_read())
            .await
            .unwrap());
        assert!(issuer_credential3
            .is_revoked(&issuer.profile.inject_anoncreds_ledger_read())
            .await
            .unwrap());
    })
    .await;
}
