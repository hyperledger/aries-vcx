use std::{error::Error, thread, time::Duration};

use anoncreds_types::data_types::messages::pres_request::NonRevokedInterval;
use aries_vcx::protocols::proof_presentation::verifier::{
    state_machine::VerifierState, verification_status::PresentationVerificationStatus,
};
use test_utils::devsetup::*;

use crate::utils::{
    scenarios::{
        create_address_schema_creddef_revreg, create_proof_request_data,
        create_verifier_from_request_data, credential_data_address_1, credential_data_address_2,
        credential_data_address_3, exchange_credential, exchange_proof, issue_address_credential,
        prover_select_credentials_and_send_proof, publish_revocation, requested_attrs_address,
        revoke_credential_and_publish_accumulator, revoke_credential_local, rotate_rev_reg,
        verifier_create_proof_and_send_request,
    },
    test_agent::{create_test_agent, create_test_agent_trustee},
};

pub mod utils;

#[tokio::test]
#[ignore]
async fn test_agency_pool_basic_revocation() -> Result<(), Box<dyn Error>> {
    let setup = SetupPoolDirectory::init().await;
    let mut institution = create_test_agent_trustee(setup.genesis_file_path.clone()).await;
    let mut consumer = create_test_agent(setup.genesis_file_path).await;

    let (schema, cred_def, rev_reg, issuer) =
        issue_address_credential(&mut consumer, &mut institution).await;

    assert!(!issuer.is_revoked(&institution.ledger_read).await?);

    let time_before_revocation = time::OffsetDateTime::now_utc().unix_timestamp() as u64;
    revoke_credential_and_publish_accumulator(&mut institution, &issuer, &rev_reg).await;

    tokio::time::sleep(Duration::from_millis(1000)).await;
    let time_after_revocation = time::OffsetDateTime::now_utc().unix_timestamp() as u64;

    assert!(issuer.is_revoked(&institution.ledger_read).await?);

    let requested_attrs = requested_attrs_address(
        &institution.institution_did,
        &schema.schema_id,
        cred_def.get_cred_def_id(),
        None,
        Some(time_after_revocation),
    );

    let presentation_request_data = create_proof_request_data(
        &mut institution,
        requested_attrs,
        Default::default(),
        NonRevokedInterval::new(
            Some(time_before_revocation - 100),
            Some(time_after_revocation),
        ),
        None,
    )
    .await;

    let mut verifier = create_verifier_from_request_data(presentation_request_data).await;
    let presentation = prover_select_credentials_and_send_proof(
        &mut consumer,
        verifier.get_presentation_request_msg()?,
        None,
    )
    .await;

    verifier
        .verify_presentation(
            &institution.ledger_read,
            &institution.anoncreds,
            presentation,
        )
        .await?;
    assert_eq!(verifier.get_state(), VerifierState::Finished);
    assert_eq!(
        verifier.get_verification_status(),
        PresentationVerificationStatus::Invalid
    );
    Ok(())
}

#[tokio::test]
#[ignore]
async fn test_agency_pool_revoked_credential_might_still_work() -> Result<(), Box<dyn Error>> {
    let setup = SetupPoolDirectory::init().await;
    let mut institution = create_test_agent_trustee(setup.genesis_file_path.clone()).await;
    let mut consumer = create_test_agent(setup.genesis_file_path).await;

    let (schema, cred_def, rev_reg, issuer) =
        issue_address_credential(&mut consumer, &mut institution).await;

    assert!(!issuer.is_revoked(&institution.ledger_read).await?);

    tokio::time::sleep(Duration::from_millis(1000)).await;
    let time_before_revocation = time::OffsetDateTime::now_utc().unix_timestamp() as u64;
    tokio::time::sleep(Duration::from_millis(1000)).await;

    revoke_credential_and_publish_accumulator(&mut institution, &issuer, &rev_reg).await;
    tokio::time::sleep(Duration::from_millis(1000)).await;

    let from = time_before_revocation - 100;
    let to = time_before_revocation;
    let requested_attrs = requested_attrs_address(
        &institution.institution_did,
        &schema.schema_id,
        cred_def.get_cred_def_id(),
        Some(from),
        Some(to),
    );

    let presentation_request_data = create_proof_request_data(
        &mut institution,
        requested_attrs,
        Default::default(),
        NonRevokedInterval::new(Some(from), Some(to)),
        None,
    )
    .await;

    let mut verifier = create_verifier_from_request_data(presentation_request_data).await;
    let presentation = prover_select_credentials_and_send_proof(
        &mut consumer,
        verifier.get_presentation_request_msg()?,
        None,
    )
    .await;

    verifier
        .verify_presentation(
            &institution.ledger_read,
            &institution.anoncreds,
            presentation,
        )
        .await?;
    assert_eq!(verifier.get_state(), VerifierState::Finished);
    assert_eq!(
        verifier.get_verification_status(),
        PresentationVerificationStatus::Valid
    );
    Ok(())
}

#[tokio::test]
#[ignore]
async fn test_agency_pool_local_revocation() -> Result<(), Box<dyn Error>> {
    let setup = SetupPoolDirectory::init().await;
    let mut institution = create_test_agent_trustee(setup.genesis_file_path.clone()).await;
    let mut consumer = create_test_agent(setup.genesis_file_path).await;

    let (schema, cred_def, rev_reg, issuer) =
        issue_address_credential(&mut consumer, &mut institution).await;

    revoke_credential_local(&mut institution, &issuer, &rev_reg.rev_reg_id).await;
    assert!(!issuer.is_revoked(&institution.ledger_read).await?);

    let verifier_handler = exchange_proof(
        &mut institution,
        &mut consumer,
        &schema.schema_id,
        cred_def.get_cred_def_id(),
        Some("request1"),
    )
    .await;
    assert_eq!(
        verifier_handler.get_verification_status(),
        PresentationVerificationStatus::Valid
    );

    assert!(!issuer.is_revoked(&institution.ledger_read).await?);

    publish_revocation(&mut institution, &rev_reg).await;

    let verifier_handler = exchange_proof(
        &mut institution,
        &mut consumer,
        &schema.schema_id,
        cred_def.get_cred_def_id(),
        Some("request2"),
    )
    .await;
    assert_eq!(
        verifier_handler.get_verification_status(),
        PresentationVerificationStatus::Invalid
    );

    assert!(issuer.is_revoked(&institution.ledger_read).await?);
    Ok(())
}

// TODO - re-enable after https://github.com/hyperledger/aries-vcx/issues/1309
// #[tokio::test]
// #[ignore]
#[allow(unused)]
async fn test_agency_batch_revocation() -> Result<(), Box<dyn Error>> {
    let setup = SetupPoolDirectory::init().await;
    let mut institution = create_test_agent_trustee(setup.genesis_file_path.clone()).await;
    let mut consumer1 = create_test_agent(setup.genesis_file_path.clone()).await;
    let mut consumer2 = create_test_agent(setup.genesis_file_path.clone()).await;
    let mut consumer3 = create_test_agent(setup.genesis_file_path).await;

    // Issue and send three credentials of the same schema
    let (schema, cred_def, rev_reg) = create_address_schema_creddef_revreg(
        &institution.wallet,
        &institution.ledger_read,
        &institution.ledger_write,
        &institution.anoncreds,
        &institution.institution_did,
    )
    .await;

    let issuer_credential1 = exchange_credential(
        &mut consumer1,
        &mut institution,
        credential_data_address_1().to_string(),
        &cred_def,
        &rev_reg,
        None,
    )
    .await;

    let issuer_credential2 = exchange_credential(
        &mut consumer2,
        &mut institution,
        credential_data_address_2().to_string(),
        &cred_def,
        &rev_reg,
        None,
    )
    .await;

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
    assert!(
        !issuer_credential1
            .is_revoked(&institution.ledger_read)
            .await?
    );
    assert!(
        !issuer_credential2
            .is_revoked(&institution.ledger_read)
            .await?
    );
    assert!(
        !issuer_credential3
            .is_revoked(&institution.ledger_read)
            .await?
    );

    // Revoke two locally and verify their are all still valid
    let verifier_handler = exchange_proof(
        &mut institution,
        &mut consumer1,
        &schema.schema_id,
        cred_def.get_cred_def_id(),
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
        &schema.schema_id,
        cred_def.get_cred_def_id(),
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
        &schema.schema_id,
        cred_def.get_cred_def_id(),
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

    assert!(
        issuer_credential1
            .is_revoked(&institution.ledger_read)
            .await?
    );
    assert!(
        issuer_credential2
            .is_revoked(&institution.ledger_read)
            .await?
    );
    assert!(
        !issuer_credential3
            .is_revoked(&institution.ledger_read)
            .await?
    );

    let verifier_handler = exchange_proof(
        &mut institution,
        &mut consumer1,
        &schema.schema_id,
        cred_def.get_cred_def_id(),
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
        &schema.schema_id,
        cred_def.get_cred_def_id(),
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
        &schema.schema_id,
        cred_def.get_cred_def_id(),
        Some("request3"),
    )
    .await;
    assert_eq!(
        verifier_handler.get_verification_status(),
        PresentationVerificationStatus::Valid
    );
    Ok(())
}

#[tokio::test]
#[ignore]
#[allow(unused_mut)]
async fn test_agency_pool_two_creds_one_rev_reg_revoke_first() -> Result<(), Box<dyn Error>> {
    let setup = SetupPoolDirectory::init().await;
    let mut issuer = create_test_agent_trustee(setup.genesis_file_path.clone()).await;
    let mut verifier = create_test_agent_trustee(setup.genesis_file_path.clone()).await;
    let mut consumer = create_test_agent(setup.genesis_file_path).await;

    let (schema, cred_def, rev_reg) = create_address_schema_creddef_revreg(
        &issuer.wallet,
        &issuer.ledger_read,
        &issuer.ledger_write,
        &issuer.anoncreds,
        &issuer.institution_did,
    )
    .await;
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

    assert!(!issuer_credential1.is_revoked(&issuer.ledger_read).await?);
    assert!(!issuer_credential2.is_revoked(&issuer.ledger_read).await?);

    revoke_credential_and_publish_accumulator(&mut issuer, &issuer_credential1, &rev_reg).await;

    let mut proof_verifier = verifier_create_proof_and_send_request(
        &mut verifier,
        &schema.schema_id,
        cred_def.get_cred_def_id(),
        Some("request1"),
    )
    .await;
    let presentation_request = proof_verifier.get_presentation_request_msg()?;
    let presentation = prover_select_credentials_and_send_proof(
        &mut consumer,
        presentation_request,
        Some(&credential_data1),
    )
    .await;
    proof_verifier
        .verify_presentation(&verifier.ledger_read, &verifier.anoncreds, presentation)
        .await?;
    assert_eq!(proof_verifier.get_state(), VerifierState::Finished);
    assert_eq!(
        proof_verifier.get_verification_status(),
        PresentationVerificationStatus::Invalid
    );

    let mut proof_verifier = verifier_create_proof_and_send_request(
        &mut verifier,
        &schema.schema_id,
        cred_def.get_cred_def_id(),
        Some("request2"),
    )
    .await;
    let presentation_request = proof_verifier.get_presentation_request_msg()?;
    let presentation = prover_select_credentials_and_send_proof(
        &mut consumer,
        presentation_request,
        Some(&credential_data2),
    )
    .await;

    proof_verifier
        .verify_presentation(&verifier.ledger_read, &verifier.anoncreds, presentation)
        .await?;
    assert_eq!(
        proof_verifier.get_verification_status(),
        PresentationVerificationStatus::Valid
    );

    assert!(issuer_credential1.is_revoked(&issuer.ledger_read).await?);
    assert!(!issuer_credential2.is_revoked(&issuer.ledger_read).await?);
    Ok(())
}

#[tokio::test]
#[ignore]
#[allow(unused_mut)]
async fn test_agency_pool_two_creds_one_rev_reg_revoke_second() -> Result<(), Box<dyn Error>> {
    let setup = SetupPoolDirectory::init().await;
    let mut issuer = create_test_agent_trustee(setup.genesis_file_path.clone()).await;
    let mut verifier = create_test_agent_trustee(setup.genesis_file_path.clone()).await;
    let mut consumer = create_test_agent(setup.genesis_file_path).await;

    let (schema, cred_def, rev_reg) = create_address_schema_creddef_revreg(
        &issuer.wallet,
        &issuer.ledger_read,
        &issuer.ledger_write,
        &issuer.anoncreds,
        &issuer.institution_did,
    )
    .await;
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

    assert!(!issuer_credential1.is_revoked(&issuer.ledger_read).await?);
    assert!(!issuer_credential2.is_revoked(&issuer.ledger_read).await?);

    revoke_credential_and_publish_accumulator(&mut issuer, &issuer_credential2, &rev_reg).await;

    let mut proof_verifier = verifier_create_proof_and_send_request(
        &mut verifier,
        &schema.schema_id,
        cred_def.get_cred_def_id(),
        Some("request1"),
    )
    .await;
    let presentation = prover_select_credentials_and_send_proof(
        &mut consumer,
        proof_verifier.get_presentation_request_msg()?,
        Some(&credential_data1),
    )
    .await;
    proof_verifier
        .verify_presentation(&verifier.ledger_read, &verifier.anoncreds, presentation)
        .await?;
    assert_eq!(proof_verifier.get_state(), VerifierState::Finished);
    assert_eq!(
        proof_verifier.get_verification_status(),
        PresentationVerificationStatus::Valid
    );

    let mut proof_verifier = verifier_create_proof_and_send_request(
        &mut verifier,
        &schema.schema_id,
        cred_def.get_cred_def_id(),
        Some("request2"),
    )
    .await;
    let presentation = prover_select_credentials_and_send_proof(
        &mut consumer,
        proof_verifier.get_presentation_request_msg()?,
        Some(&credential_data2),
    )
    .await;

    proof_verifier
        .verify_presentation(&verifier.ledger_read, &verifier.anoncreds, presentation)
        .await?;
    assert_eq!(
        proof_verifier.get_verification_status(),
        PresentationVerificationStatus::Invalid
    );

    assert!(!issuer_credential1.is_revoked(&issuer.ledger_read).await?);
    assert!(issuer_credential2.is_revoked(&issuer.ledger_read).await?);
    Ok(())
}

#[tokio::test]
#[ignore]
async fn test_agency_pool_two_creds_two_rev_reg_id() -> Result<(), Box<dyn Error>> {
    let setup = SetupPoolDirectory::init().await;
    let mut issuer = create_test_agent_trustee(setup.genesis_file_path.clone()).await;
    let mut verifier = create_test_agent_trustee(setup.genesis_file_path.clone()).await;
    let mut consumer = create_test_agent(setup.genesis_file_path).await;

    let (schema, cred_def, rev_reg) = create_address_schema_creddef_revreg(
        &issuer.wallet,
        &issuer.ledger_read,
        &issuer.ledger_write,
        &issuer.anoncreds,
        &issuer.institution_did,
    )
    .await;
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

    let mut proof_verifier = verifier_create_proof_and_send_request(
        &mut verifier,
        &schema.schema_id,
        cred_def.get_cred_def_id(),
        Some("request1"),
    )
    .await;
    let presentation = prover_select_credentials_and_send_proof(
        &mut consumer,
        proof_verifier.get_presentation_request_msg()?,
        Some(&credential_data1),
    )
    .await;
    proof_verifier
        .verify_presentation(&verifier.ledger_read, &verifier.anoncreds, presentation)
        .await?;
    assert_eq!(proof_verifier.get_state(), VerifierState::Finished);
    assert_eq!(
        proof_verifier.get_verification_status(),
        PresentationVerificationStatus::Valid
    );

    let mut proof_verifier = verifier_create_proof_and_send_request(
        &mut verifier,
        &schema.schema_id,
        cred_def.get_cred_def_id(),
        Some("request2"),
    )
    .await;

    let presentation = prover_select_credentials_and_send_proof(
        &mut consumer,
        proof_verifier.get_presentation_request_msg()?,
        Some(&credential_data2),
    )
    .await;
    proof_verifier
        .verify_presentation(&verifier.ledger_read, &verifier.anoncreds, presentation)
        .await?;
    assert_eq!(proof_verifier.get_state(), VerifierState::Finished);
    assert_eq!(
        proof_verifier.get_verification_status(),
        PresentationVerificationStatus::Valid
    );

    assert!(!issuer_credential1.is_revoked(&issuer.ledger_read).await?);
    assert!(!issuer_credential2.is_revoked(&issuer.ledger_read).await?);
    Ok(())
}

#[tokio::test]
#[ignore]
#[allow(unused_mut)]
async fn test_agency_pool_two_creds_two_rev_reg_id_revoke_first() -> Result<(), Box<dyn Error>> {
    let setup = SetupPoolDirectory::init().await;
    let mut issuer = create_test_agent_trustee(setup.genesis_file_path.clone()).await;
    let mut verifier = create_test_agent_trustee(setup.genesis_file_path.clone()).await;
    let mut consumer = create_test_agent(setup.genesis_file_path).await;

    let (schema, cred_def, rev_reg) = create_address_schema_creddef_revreg(
        &issuer.wallet,
        &issuer.ledger_read,
        &issuer.ledger_write,
        &issuer.anoncreds,
        &issuer.institution_did,
    )
    .await;
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

    assert!(!issuer_credential1.is_revoked(&issuer.ledger_read).await?);
    assert!(!issuer_credential2.is_revoked(&issuer.ledger_read).await?);

    revoke_credential_and_publish_accumulator(&mut issuer, &issuer_credential1, &rev_reg).await;

    let mut proof_verifier = verifier_create_proof_and_send_request(
        &mut verifier,
        &schema.schema_id,
        cred_def.get_cred_def_id(),
        Some("request1"),
    )
    .await;
    let presentation = prover_select_credentials_and_send_proof(
        &mut consumer,
        proof_verifier.get_presentation_request_msg()?,
        Some(&credential_data1),
    )
    .await;
    proof_verifier
        .verify_presentation(&verifier.ledger_read, &verifier.anoncreds, presentation)
        .await?;
    assert_eq!(proof_verifier.get_state(), VerifierState::Finished);
    assert_eq!(
        proof_verifier.get_verification_status(),
        PresentationVerificationStatus::Invalid
    );

    let mut proof_verifier = verifier_create_proof_and_send_request(
        &mut verifier,
        &schema.schema_id,
        cred_def.get_cred_def_id(),
        Some("request2"),
    )
    .await;
    let presentation = prover_select_credentials_and_send_proof(
        &mut consumer,
        proof_verifier.get_presentation_request_msg()?,
        Some(&credential_data2),
    )
    .await;

    proof_verifier
        .verify_presentation(&verifier.ledger_read, &verifier.anoncreds, presentation)
        .await?;
    assert_eq!(
        proof_verifier.get_verification_status(),
        PresentationVerificationStatus::Valid
    );

    assert!(issuer_credential1.is_revoked(&issuer.ledger_read).await?);
    assert!(!issuer_credential2.is_revoked(&issuer.ledger_read).await?);
    Ok(())
}

#[tokio::test]
#[ignore]
async fn test_agency_pool_two_creds_two_rev_reg_id_revoke_second() -> Result<(), Box<dyn Error>> {
    let setup = SetupPoolDirectory::init().await;
    let mut issuer = create_test_agent_trustee(setup.genesis_file_path.clone()).await;
    let mut verifier = create_test_agent_trustee(setup.genesis_file_path.clone()).await;
    let mut consumer = create_test_agent(setup.genesis_file_path).await;

    let (schema, cred_def, rev_reg) = create_address_schema_creddef_revreg(
        &issuer.wallet,
        &issuer.ledger_read,
        &issuer.ledger_write,
        &issuer.anoncreds,
        &issuer.institution_did,
    )
    .await;
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

    assert!(!issuer_credential1.is_revoked(&issuer.ledger_read).await?);
    assert!(!issuer_credential2.is_revoked(&issuer.ledger_read).await?);

    revoke_credential_and_publish_accumulator(&mut issuer, &issuer_credential2, &rev_reg_2).await;

    let mut proof_verifier = verifier_create_proof_and_send_request(
        &mut verifier,
        &schema.schema_id,
        cred_def.get_cred_def_id(),
        Some("request1"),
    )
    .await;
    let presentation = prover_select_credentials_and_send_proof(
        &mut consumer,
        proof_verifier.get_presentation_request_msg()?,
        Some(&credential_data1),
    )
    .await;

    proof_verifier
        .verify_presentation(&verifier.ledger_read, &verifier.anoncreds, presentation)
        .await?;
    assert_eq!(proof_verifier.get_state(), VerifierState::Finished);
    assert_eq!(
        proof_verifier.get_verification_status(),
        PresentationVerificationStatus::Valid
    );

    let mut proof_verifier = verifier_create_proof_and_send_request(
        &mut verifier,
        &schema.schema_id,
        cred_def.get_cred_def_id(),
        Some("request2"),
    )
    .await;
    let presentation = prover_select_credentials_and_send_proof(
        &mut consumer,
        proof_verifier.get_presentation_request_msg()?,
        Some(&credential_data2),
    )
    .await;

    proof_verifier
        .verify_presentation(&verifier.ledger_read, &verifier.anoncreds, presentation)
        .await?;
    assert_eq!(proof_verifier.get_state(), VerifierState::Finished);
    assert_eq!(
        proof_verifier.get_verification_status(),
        PresentationVerificationStatus::Invalid
    );

    assert!(!issuer_credential1.is_revoked(&issuer.ledger_read).await?);
    assert!(issuer_credential2.is_revoked(&issuer.ledger_read).await?);
    Ok(())
}

#[tokio::test]
#[ignore]
async fn test_agency_pool_three_creds_one_rev_reg_revoke_all() -> Result<(), Box<dyn Error>> {
    let setup = SetupPoolDirectory::init().await;
    let mut issuer = create_test_agent_trustee(setup.genesis_file_path.clone()).await;
    let mut consumer = create_test_agent(setup.genesis_file_path.clone()).await;

    let (_schema, cred_def, rev_reg) = create_address_schema_creddef_revreg(
        &issuer.wallet,
        &issuer.ledger_read,
        &issuer.ledger_write,
        &issuer.anoncreds,
        &issuer.institution_did,
    )
    .await;

    let issuer_credential1 = exchange_credential(
        &mut consumer,
        &mut issuer,
        credential_data_address_1().to_string(),
        &cred_def,
        &rev_reg,
        Some("request1"),
    )
    .await;

    assert!(!issuer_credential1.is_revoked(&issuer.ledger_read).await?);
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

    assert!(!issuer_credential2.is_revoked(&issuer.ledger_read).await?);

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

    assert!(issuer_credential1.is_revoked(&issuer.ledger_read).await?);
    assert!(issuer_credential2.is_revoked(&issuer.ledger_read).await?);
    assert!(issuer_credential3.is_revoked(&issuer.ledger_read).await?);
    Ok(())
}
