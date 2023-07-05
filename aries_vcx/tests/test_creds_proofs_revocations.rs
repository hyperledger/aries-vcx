#[macro_use]
extern crate log;
#[macro_use]
extern crate serde_json;

pub mod utils;

mod integration_tests {
    use std::time::Duration;

    use aries_vcx::protocols::proof_presentation::prover::state_machine::ProverState;
    use aries_vcx::protocols::proof_presentation::verifier::state_machine::VerifierState;
    use aries_vcx::protocols::proof_presentation::verifier::verification_status::PresentationVerificationStatus;
    use aries_vcx::utils::devsetup::*;

    use crate::utils::devsetup_alice::create_alice;
    use crate::utils::devsetup_faber::{create_faber, Faber};
    #[cfg(feature = "migration")]
    use crate::utils::migration::Migratable;
    use crate::utils::scenarios::test_utils::{
        _create_address_schema, _exchange_credential, attr_names, create_connected_connections, create_proof,
        generate_and_send_proof, issue_address_credential, prover_select_credentials_and_send_proof,
        publish_revocation, requested_attrs, retrieved_to_selected_credentials_simple,
        revoke_credential_and_publish_accumulator, revoke_credential_local, rotate_rev_reg, send_proof_request,
        verifier_create_proof_and_send_request,
    };

    use super::*;

    #[tokio::test]
    #[ignore]
    async fn test_agency_pool_basic_revocation() {
        SetupPoolDirectory::run(|setup| async move {
            let mut institution = create_faber(setup.genesis_file_path.clone()).await;
            let mut consumer = create_alice(setup.genesis_file_path).await;

            let (consumer_to_institution, institution_to_consumer) =
                create_connected_connections(&mut consumer, &mut institution).await;
            let (schema_id, cred_def_id, _, _cred_def, rev_reg, issuer_credential) = issue_address_credential(
                &mut consumer,
                &mut institution,
                &consumer_to_institution,
                &institution_to_consumer,
            )
            .await;

            #[cfg(feature = "migration")]
            institution.migrate().await;

            assert!(!issuer_credential
                .is_revoked(&institution.profile.inject_anoncreds_ledger_read())
                .await
                .unwrap());

            let time_before_revocation = time::OffsetDateTime::now_utc().unix_timestamp() as u64;
            info!("test_basic_revocation :: verifier :: Going to revoke credential");
            revoke_credential_and_publish_accumulator(&mut institution, &issuer_credential, &rev_reg).await;

            #[cfg(feature = "migration")]
            consumer.migrate().await;

            tokio::time::sleep(Duration::from_millis(1000)).await;
            let time_after_revocation = time::OffsetDateTime::now_utc().unix_timestamp() as u64;

            assert!(issuer_credential
                .is_revoked(&institution.profile.inject_anoncreds_ledger_read())
                .await
                .unwrap());

            let _requested_attrs = requested_attrs(
                &institution.institution_did,
                &schema_id,
                &cred_def_id,
                None,
                Some(time_after_revocation),
            );
            let interval = json!({"from": time_before_revocation - 100, "to": time_after_revocation}).to_string();
            let requested_attrs_string = serde_json::to_string(&_requested_attrs).unwrap();

            info!(
                "test_basic_revocation :: Going to seng proof request with attributes {}",
                &requested_attrs_string
            );
            let mut verifier = send_proof_request(
                &mut institution,
                &institution_to_consumer,
                &requested_attrs_string,
                "[]",
                &interval,
                None,
            )
            .await;

            prover_select_credentials_and_send_proof(&mut consumer, &consumer_to_institution, None, None).await;

            info!("test_basic_revocation :: verifier :: going to verify proof");
            verifier
                .update_state(
                    &institution.profile.inject_wallet(),
                    &institution.profile.inject_anoncreds_ledger_read(),
                    &institution.profile.inject_anoncreds(),
                    &institution.agency_client,
                    &institution_to_consumer,
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
    async fn test_agency_pool_revocation_notification() {
        use messages::decorators::please_ack::AckOn;

        SetupPoolDirectory::run(|setup| async move {
            let mut institution = create_faber(setup.genesis_file_path.clone()).await;
            let mut consumer = create_alice(setup.genesis_file_path).await;

            let (consumer_to_institution, institution_to_consumer) =
                create_connected_connections(&mut consumer, &mut institution).await;
            let (_, _, _, _cred_def, rev_reg, issuer_credential) = issue_address_credential(
                &mut consumer,
                &mut institution,
                &consumer_to_institution,
                &institution_to_consumer,
            )
            .await;

            #[cfg(feature = "migration")]
            institution.migrate().await;

            assert!(!issuer_credential
                .is_revoked(&institution.profile.inject_anoncreds_ledger_read())
                .await
                .unwrap());

            info!("test_revocation_notification :: verifier :: Going to revoke credential");
            revoke_credential_and_publish_accumulator(&mut institution, &issuer_credential, &rev_reg).await;
            tokio::time::sleep(Duration::from_millis(1000)).await;

            #[cfg(feature = "migration")]
            consumer.migrate().await;

            assert!(issuer_credential
                .is_revoked(&institution.profile.inject_anoncreds_ledger_read())
                .await
                .unwrap());
            let config =
                aries_vcx::protocols::revocation_notification::sender::state_machine::SenderConfigBuilder::default()
                    .ack_on(vec![AckOn::Receipt])
                    .rev_reg_id(issuer_credential.get_rev_reg_id().unwrap())
                    .cred_rev_id(issuer_credential.get_rev_id().unwrap())
                    .comment(None)
                    .build()
                    .unwrap();
            let send_message = institution_to_consumer
                .send_message_closure(institution.profile.inject_wallet())
                .await
                .unwrap();
            aries_vcx::handlers::revocation_notification::sender::RevocationNotificationSender::build()
                .clone()
                .send_revocation_notification(config, send_message)
                .await
                .unwrap();

            let rev_nots =
                aries_vcx::handlers::revocation_notification::test_utils::get_revocation_notification_messages(
                    &consumer.agency_client,
                    &consumer_to_institution,
                )
                .await
                .unwrap();
            assert_eq!(rev_nots.len(), 1);

            // consumer.receive_revocation_notification(rev_not).await;
            // let ack = aries_vcx::handlers::revocation_notification::test_utils::get_revocation_notification_ack_messages(&institution.agency_client, &institution_to_consumer).await.unwrap().pop().unwrap();
            // institution.handle_revocation_notification_ack(ack).await;
        })
        .await;
    }

    #[tokio::test]
    #[ignore]
    async fn test_agency_pool_local_revocation() {
        SetupPoolDirectory::run(|setup| async move {
            let mut institution = create_faber(setup.genesis_file_path.clone()).await;
            let mut consumer = create_alice(setup.genesis_file_path).await;

            let (consumer_to_institution, institution_to_consumer) =
                create_connected_connections(&mut consumer, &mut institution).await;
            let (schema_id, cred_def_id, _, _cred_def, rev_reg, issuer_credential) = issue_address_credential(
                &mut consumer,
                &mut institution,
                &consumer_to_institution,
                &institution_to_consumer,
            )
            .await;

            #[cfg(feature = "migration")]
            institution.migrate().await;

            revoke_credential_local(&mut institution, &issuer_credential, &rev_reg.rev_reg_id).await;
            assert!(!issuer_credential
                .is_revoked(&institution.profile.inject_anoncreds_ledger_read())
                .await
                .unwrap());
            let request_name1 = Some("request1");
            let mut verifier = verifier_create_proof_and_send_request(
                &mut institution,
                &institution_to_consumer,
                &schema_id,
                &cred_def_id,
                request_name1,
            )
            .await;

            #[cfg(feature = "migration")]
            consumer.migrate().await;

            prover_select_credentials_and_send_proof(&mut consumer, &consumer_to_institution, request_name1, None)
                .await;

            verifier
                .update_state(
                    &institution.profile.inject_wallet(),
                    &institution.profile.inject_anoncreds_ledger_read(),
                    &institution.profile.inject_anoncreds(),
                    &institution.agency_client,
                    &institution_to_consumer,
                )
                .await
                .unwrap();
            assert_eq!(verifier.get_state(), VerifierState::Finished);
            assert_eq!(
                verifier.get_verification_status(),
                PresentationVerificationStatus::Valid
            );

            assert!(!issuer_credential
                .is_revoked(&institution.profile.inject_anoncreds_ledger_read())
                .await
                .unwrap());

            publish_revocation(&mut institution, &rev_reg).await;
            let request_name2 = Some("request2");
            let mut verifier = verifier_create_proof_and_send_request(
                &mut institution,
                &institution_to_consumer,
                &schema_id,
                &cred_def_id,
                request_name2,
            )
            .await;
            prover_select_credentials_and_send_proof(&mut consumer, &consumer_to_institution, request_name2, None)
                .await;

            verifier
                .update_state(
                    &institution.profile.inject_wallet(),
                    &institution.profile.inject_anoncreds_ledger_read(),
                    &institution.profile.inject_anoncreds(),
                    &institution.agency_client,
                    &institution_to_consumer,
                )
                .await
                .unwrap();
            assert_eq!(verifier.get_state(), VerifierState::Finished);
            assert_eq!(
                verifier.get_verification_status(),
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
            let mut institution = create_faber(setup.genesis_file_path.clone()).await;
            let mut consumer1 = create_alice(setup.genesis_file_path.clone()).await;
            let mut consumer2 = create_alice(setup.genesis_file_path.clone()).await;
            let mut consumer3 = create_alice(setup.genesis_file_path).await;

            let (consumer_to_institution1, institution_to_consumer1) = create_connected_connections(&mut consumer1, &mut institution).await;
            let (consumer_to_institution2, institution_to_consumer2) = create_connected_connections(&mut consumer2, &mut institution).await;
            let (consumer_to_institution3, institution_to_consumer3) = create_connected_connections(&mut consumer3, &mut institution).await;

            // Issue and send three credentials of the same schema
            let (schema_id, _schema_json, cred_def_id, _cred_def_json, cred_def, rev_reg, _rev_reg_id) =
                _create_address_schema(&institution.profile, &institution.institution_did).await;
            let (address1, address2, city, state, zip) = attr_names();
            let credential_data1 = json!({address1.clone(): "123 Main St", address2.clone(): "Suite 3", city.clone(): "Draper", state.clone(): "UT", zip.clone(): "84000"}).to_string();
            let issuer_credential1 = _exchange_credential(
                &mut consumer1,
                &mut institution,
                credential_data1,
                &cred_def,
                &rev_reg,
                &consumer_to_institution1,
                &institution_to_consumer1,
                None,
            )
                .await;


            #[cfg(feature = "migration")]
            institution.migrate().await;

            let credential_data2 = json!({address1.clone(): "101 Tela Lane", address2.clone(): "Suite 1", city.clone(): "SLC", state.clone(): "WA", zip.clone(): "8721"}).to_string();
            let issuer_credential2 = _exchange_credential(
                &mut consumer2,
                &mut institution,
                credential_data2,
                &cred_def,
                &rev_reg,
                &consumer_to_institution2,
                &institution_to_consumer2,
                None,
            )
                .await;


            #[cfg(feature = "migration")]
            consumer1.migrate().await;

            let credential_data3 = json!({address1.clone(): "5th Avenue", address2.clone(): "Suite 1234", city.clone(): "NYC", state.clone(): "NYS", zip.clone(): "84712"}).to_string();
            let issuer_credential3 = _exchange_credential(
                &mut consumer3,
                &mut institution,
                credential_data3,
                &cred_def,
                &rev_reg,
                &consumer_to_institution3,
                &institution_to_consumer3,
                None,
            )
                .await;

            revoke_credential_local(&mut institution, &issuer_credential1, &rev_reg.rev_reg_id).await;
            revoke_credential_local(&mut institution, &issuer_credential2, &rev_reg.rev_reg_id).await;
            assert!(!issuer_credential1.is_revoked(&institution.profile.inject_anoncreds_ledger_read()).await.unwrap());
            assert!(!issuer_credential2.is_revoked(&institution.profile.inject_anoncreds_ledger_read()).await.unwrap());
            assert!(!issuer_credential3.is_revoked(&institution.profile.inject_anoncreds_ledger_read()).await.unwrap());

            // Revoke two locally and verify their are all still valid
            let request_name1 = Some("request1");
            let mut verifier1 = verifier_create_proof_and_send_request(&mut institution, &institution_to_consumer1, &schema_id, &cred_def_id, request_name1).await;

            #[cfg(feature = "migration")]
            consumer2.migrate().await;

            #[cfg(feature = "migration")]
            consumer3.migrate().await;

            prover_select_credentials_and_send_proof(&mut consumer1, &consumer_to_institution1, request_name1, None).await;
            let mut verifier2 = verifier_create_proof_and_send_request(&mut institution, &institution_to_consumer2, &schema_id, &cred_def_id, request_name1).await;
            prover_select_credentials_and_send_proof(&mut consumer2, &consumer_to_institution2, request_name1, None).await;
            let mut verifier3 = verifier_create_proof_and_send_request(&mut institution, &institution_to_consumer3, &schema_id, &cred_def_id, request_name1).await;
            prover_select_credentials_and_send_proof(&mut consumer3, &consumer_to_institution3, request_name1, None).await;

            verifier1
                .update_state(
                    &institution.profile.inject_wallet(),
                    &institution.profile.inject_anoncreds_ledger_read(),
                    &institution.profile.inject_anoncreds(),
                    &institution.agency_client,
                    &institution_to_consumer1,
                )
                .await
                .unwrap();
            verifier2
                .update_state(
                    &institution.profile.inject_wallet(),
                    &institution.profile.inject_anoncreds_ledger_read(),
                    &institution.profile.inject_anoncreds(),
                    &institution.agency_client,
                    &institution_to_consumer2,
                )
                .await
                .unwrap();
            verifier3
                .update_state(
                    &institution.profile.inject_wallet(),
                    &institution.profile.inject_anoncreds_ledger_read(),
                    &institution.profile.inject_anoncreds(),
                    &institution.agency_client,
                    &institution_to_consumer3,
                )
                .await
                .unwrap();
            assert_eq!(verifier1.get_verification_status(), PresentationVerificationStatus::Valid);
            assert_eq!(verifier2.get_verification_status(), PresentationVerificationStatus::Valid);
            assert_eq!(verifier3.get_verification_status(), PresentationVerificationStatus::Valid);

            // Publish revocations and verify the two are invalid, third still valid
            publish_revocation(&mut institution, &rev_reg).await;
            tokio::time::sleep(Duration::from_millis(1000)).await;

            assert!(issuer_credential1.is_revoked(&institution.profile.inject_anoncreds_ledger_read()).await.unwrap());
            assert!(issuer_credential2.is_revoked(&institution.profile.inject_anoncreds_ledger_read()).await.unwrap());
            assert!(!issuer_credential3.is_revoked(&institution.profile.inject_anoncreds_ledger_read()).await.unwrap());

            let request_name2 = Some("request2");
            let mut verifier1 = verifier_create_proof_and_send_request(&mut institution, &institution_to_consumer1, &schema_id, &cred_def_id, request_name2).await;
            prover_select_credentials_and_send_proof(&mut consumer1, &consumer_to_institution1, request_name2, None).await;
            let mut verifier2 = verifier_create_proof_and_send_request(&mut institution, &institution_to_consumer2, &schema_id, &cred_def_id, request_name2).await;
            prover_select_credentials_and_send_proof(&mut consumer2, &consumer_to_institution2, request_name2, None).await;
            let mut verifier3 = verifier_create_proof_and_send_request(&mut institution, &institution_to_consumer3, &schema_id, &cred_def_id, request_name2).await;
            prover_select_credentials_and_send_proof(&mut consumer3, &consumer_to_institution3, request_name2, None).await;
            assert_ne!(verifier1, verifier2);
            assert_ne!(verifier1, verifier3);
            assert_ne!(verifier2, verifier3);

            verifier1
                .update_state(
                    &institution.profile.inject_wallet(),
                    &institution.profile.inject_anoncreds_ledger_read(),
                    &institution.profile.inject_anoncreds(),
                    &institution.agency_client,
                    &institution_to_consumer1,
                )
                .await
                .unwrap();
            verifier2
                .update_state(
                    &institution.profile.inject_wallet(),
                    &institution.profile.inject_anoncreds_ledger_read(),
                    &institution.profile.inject_anoncreds(),
                    &institution.agency_client,
                    &institution_to_consumer2,
                )
                .await
                .unwrap();
            verifier3
                .update_state(
                    &institution.profile.inject_wallet(),
                    &institution.profile.inject_anoncreds_ledger_read(),
                    &institution.profile.inject_anoncreds(),
                    &institution.agency_client,
                    &institution_to_consumer3,
                )
                .await
                .unwrap();
            assert_eq!(verifier1.get_verification_status(), PresentationVerificationStatus::Invalid);

            assert_eq!(verifier2.get_verification_status(), PresentationVerificationStatus::Invalid);

            assert_eq!(verifier3.get_verification_status(), PresentationVerificationStatus::Valid);
        })
            .await;
    }

    #[tokio::test]
    #[ignore]
    async fn test_agency_pool_revoked_credential_might_still_work() {
        SetupPoolDirectory::run(|setup| async move {
            let mut institution = create_faber(setup.genesis_file_path.clone()).await;
            let mut consumer = create_alice(setup.genesis_file_path).await;

            let (consumer_to_institution, institution_to_consumer) = create_connected_connections(&mut consumer, &mut institution).await;
            let (schema_id, cred_def_id, _, _cred_def, rev_reg, issuer_credential) =
                issue_address_credential(&mut consumer, &mut institution, &consumer_to_institution, &institution_to_consumer).await;

            assert!(!issuer_credential.is_revoked(&institution.profile.inject_anoncreds_ledger_read()).await.unwrap());

            #[cfg(feature = "migration")]
            institution.migrate().await;

            tokio::time::sleep(Duration::from_millis(1000)).await;
            let time_before_revocation = time::OffsetDateTime::now_utc().unix_timestamp() as u64;
            tokio::time::sleep(Duration::from_millis(1000)).await;
            info!("test_revoked_credential_might_still_work :: verifier :: Going to revoke credential");
            revoke_credential_and_publish_accumulator(&mut institution, &issuer_credential, &rev_reg).await;
            tokio::time::sleep(Duration::from_millis(1000)).await;

            #[cfg(feature = "migration")]
            consumer.migrate().await;

            let from = time_before_revocation - 100;
            let to = time_before_revocation;
            let _requested_attrs = requested_attrs(&institution.institution_did, &schema_id, &cred_def_id, Some(from), Some(to));
            let interval = json!({"from": from, "to": to}).to_string();
            let requested_attrs_string = serde_json::to_string(&_requested_attrs).unwrap();

            info!("test_revoked_credential_might_still_work :: Going to seng proof request with attributes {}", &requested_attrs_string);
            let mut verifier = send_proof_request(&mut institution, &institution_to_consumer, &requested_attrs_string, "[]", &interval, None).await;

            info!("test_revoked_credential_might_still_work :: Going to create proof");
            let mut prover = create_proof(&mut consumer, &consumer_to_institution, None).await;
            info!("test_revoked_credential_might_still_work :: retrieving matching credentials");

            let retrieved_credentials = prover.retrieve_credentials(&consumer.profile.inject_anoncreds()).await.unwrap();
            info!(
                "test_revoked_credential_might_still_work :: prover :: based on proof, retrieved credentials: {:?}",
                &retrieved_credentials
            );

            let selected_credentials = retrieved_to_selected_credentials_simple(&retrieved_credentials, true);
            info!(
                "test_revoked_credential_might_still_work :: prover :: retrieved credential converted to selected: {:?}",
                &selected_credentials
            );
            generate_and_send_proof(&mut consumer, &mut prover, &consumer_to_institution, selected_credentials).await;
            assert_eq!(ProverState::PresentationSent, prover.get_state());

            info!("test_revoked_credential_might_still_work :: verifier :: going to verify proof");
            verifier
                .update_state(
                    &institution.profile.inject_wallet(),
                    &institution.profile.inject_anoncreds_ledger_read(),
                    &institution.profile.inject_anoncreds(),
                    &institution.agency_client,
                    &institution_to_consumer,
                )
                .await
                .unwrap();
            assert_eq!(verifier.get_state(), VerifierState::Finished);
            assert_eq!(verifier.get_verification_status(), PresentationVerificationStatus::Valid);
        })
            .await;
    }

    #[tokio::test]
    #[ignore]
    async fn test_agency_pool_two_creds_one_rev_reg_revoke_first() {
        SetupPoolDirectory::run(|setup| async move {
            let mut issuer = create_faber(setup.genesis_file_path.clone()).await;
            let mut verifier = create_faber(setup.genesis_file_path.clone()).await;
            let mut consumer = create_alice(setup.genesis_file_path).await;

            let (consumer_to_verifier, verifier_to_consumer) = create_connected_connections(&mut consumer, &mut verifier).await;
            let (consumer_to_issuer, issuer_to_consumer) = create_connected_connections(&mut consumer, &mut issuer).await;

            let (schema_id, _schema_json, cred_def_id, _cred_def_json, cred_def, rev_reg, _rev_reg_id) =
                _create_address_schema(&issuer.profile, &issuer.institution_did).await;
            let (address1, address2, city, state, zip) = attr_names();
            let (req1, req2) = (Some("request1"), Some("request2"));
            let credential_data1 = json!({address1.clone(): "123 Main St", address2.clone(): "Suite 3", city.clone(): "Draper", state.clone(): "UT", zip.clone(): "84000"}).to_string();
            let issuer_credential1 = _exchange_credential(
                &mut consumer,
                &mut issuer,
                credential_data1.clone(),
                &cred_def,
                &rev_reg,
                &consumer_to_issuer,
                &issuer_to_consumer,
                req1,
            )
                .await;


            #[cfg(feature = "migration")]
            issuer.migrate().await;

            let credential_data2 = json!({address1.clone(): "101 Tela Lane", address2.clone(): "Suite 1", city.clone(): "SLC", state.clone(): "WA", zip.clone(): "8721"}).to_string();
            let issuer_credential2 = _exchange_credential(
                &mut consumer,
                &mut issuer,
                credential_data2.clone(),
                &cred_def,
                &rev_reg,
                &consumer_to_issuer,
                &issuer_to_consumer,
                req2,
            )
                .await;

            assert!(!issuer_credential1.is_revoked(&issuer.profile.inject_anoncreds_ledger_read()).await.unwrap());
            assert!(!issuer_credential2.is_revoked(&issuer.profile.inject_anoncreds_ledger_read()).await.unwrap());

            #[cfg(feature = "migration")]
            verifier.migrate().await;

            revoke_credential_and_publish_accumulator(&mut issuer, &issuer_credential1, &rev_reg).await;

            let mut proof_verifier = verifier_create_proof_and_send_request(&mut verifier, &verifier_to_consumer, &schema_id, &cred_def_id, req1).await;
            prover_select_credentials_and_send_proof(&mut consumer, &consumer_to_verifier, req1, Some(&credential_data1)).await;
            proof_verifier
                .update_state(
                    &verifier.profile.inject_wallet(),
                    &verifier.profile.inject_anoncreds_ledger_read(),
                    &verifier.profile.inject_anoncreds(),
                    &verifier.agency_client,
                    &verifier_to_consumer,
                )
                .await
                .unwrap();
            assert_eq!(proof_verifier.get_state(), VerifierState::Finished);
            assert_eq!(proof_verifier.get_verification_status(), PresentationVerificationStatus::Invalid);

            let mut proof_verifier = verifier_create_proof_and_send_request(&mut verifier, &verifier_to_consumer, &schema_id, &cred_def_id, req2).await;
            prover_select_credentials_and_send_proof(&mut consumer, &consumer_to_verifier, req2, Some(&credential_data2)).await;

            #[cfg(feature = "migration")]
            consumer.migrate().await;

            proof_verifier
                .update_state(
                    &verifier.profile.inject_wallet(),
                    &verifier.profile.inject_anoncreds_ledger_read(),
                    &verifier.profile.inject_anoncreds(),
                    &verifier.agency_client,
                    &verifier_to_consumer,
                )
                .await
                .unwrap();
            assert_eq!(proof_verifier.get_verification_status(), PresentationVerificationStatus::Valid);

            assert!(issuer_credential1.is_revoked(&issuer.profile.inject_anoncreds_ledger_read()).await.unwrap());
            assert!(!issuer_credential2.is_revoked(&issuer.profile.inject_anoncreds_ledger_read()).await.unwrap());
        })
            .await;
    }

    #[tokio::test]
    #[ignore]
    async fn test_agency_pool_two_creds_one_rev_reg_revoke_second() {
        SetupPoolDirectory::run(|setup| async move {
            let mut issuer = create_faber(setup.genesis_file_path.clone()).await;
            let mut verifier = create_faber(setup.genesis_file_path.clone()).await;
            let mut consumer = create_alice(setup.genesis_file_path).await;

            let (consumer_to_verifier, verifier_to_consumer) = create_connected_connections(&mut consumer, &mut verifier).await;
            let (consumer_to_issuer, issuer_to_consumer) = create_connected_connections(&mut consumer, &mut issuer).await;

            let (schema_id, _schema_json, cred_def_id, _cred_def_json, cred_def, rev_reg, _rev_reg_id) =
                _create_address_schema(&issuer.profile, &issuer.institution_did).await;
            let (address1, address2, city, state, zip) = attr_names();
            let (req1, req2) = (Some("request1"), Some("request2"));
            let credential_data1 = json!({address1.clone(): "123 Main St", address2.clone(): "Suite 3", city.clone(): "Draper", state.clone(): "UT", zip.clone(): "84000"}).to_string();
            let issuer_credential1 = _exchange_credential(
                &mut consumer,
                &mut issuer,
                credential_data1.clone(),
                &cred_def,
                &rev_reg,
                &consumer_to_issuer,
                &issuer_to_consumer,
                req1,
            )
                .await;

            #[cfg(feature = "migration")]
            issuer.migrate().await;

            let credential_data2 = json!({address1.clone(): "101 Tela Lane", address2.clone(): "Suite 1", city.clone(): "SLC", state.clone(): "WA", zip.clone(): "8721"}).to_string();
            let issuer_credential2 = _exchange_credential(
                &mut consumer,
                &mut issuer,
                credential_data2.clone(),
                &cred_def,
                &rev_reg,
                &consumer_to_issuer,
                &issuer_to_consumer,
                req2,
            )
                .await;

            assert!(!issuer_credential1.is_revoked(&issuer.profile.inject_anoncreds_ledger_read()).await.unwrap());
            assert!(!issuer_credential2.is_revoked(&issuer.profile.inject_anoncreds_ledger_read()).await.unwrap());

            revoke_credential_and_publish_accumulator(&mut issuer, &issuer_credential2, &rev_reg).await;

            #[cfg(feature = "migration")]
            verifier.migrate().await;

            let mut proof_verifier = verifier_create_proof_and_send_request(&mut verifier, &verifier_to_consumer, &schema_id, &cred_def_id, req1).await;
            prover_select_credentials_and_send_proof(&mut consumer, &consumer_to_verifier, req1, Some(&credential_data1)).await;
            proof_verifier
                .update_state(
                    &verifier.profile.inject_wallet(),
                    &verifier.profile.inject_anoncreds_ledger_read(),
                    &verifier.profile.inject_anoncreds(),
                    &verifier.agency_client,
                    &verifier_to_consumer,
                )
                .await
                .unwrap();
            assert_eq!(proof_verifier.get_verification_status(), PresentationVerificationStatus::Valid);

            let mut proof_verifier = verifier_create_proof_and_send_request(&mut verifier, &verifier_to_consumer, &schema_id, &cred_def_id, req2).await;

            #[cfg(feature = "migration")]
            consumer.migrate().await;

            prover_select_credentials_and_send_proof(&mut consumer, &consumer_to_verifier, req2, Some(&credential_data2)).await;
            proof_verifier
                .update_state(
                    &verifier.profile.inject_wallet(),
                    &verifier.profile.inject_anoncreds_ledger_read(),
                    &verifier.profile.inject_anoncreds(),
                    &verifier.agency_client,
                    &verifier_to_consumer,
                )
                .await
                .unwrap();
            assert_eq!(proof_verifier.get_verification_status(), PresentationVerificationStatus::Invalid);

            assert!(!issuer_credential1.is_revoked(&issuer.profile.inject_anoncreds_ledger_read()).await.unwrap());
            assert!(issuer_credential2.is_revoked(&issuer.profile.inject_anoncreds_ledger_read()).await.unwrap());
        })
            .await;
    }

    #[tokio::test]
    #[ignore]
    async fn test_agency_pool_two_creds_two_rev_reg_id() {
        SetupPoolDirectory::run(|setup| async move {
            let mut issuer = create_faber(setup.genesis_file_path.clone()).await;
            let mut verifier = create_faber(setup.genesis_file_path.clone()).await;
            let mut consumer = create_alice(setup.genesis_file_path).await;

            let (consumer_to_verifier, verifier_to_consumer) = create_connected_connections(&mut consumer, &mut verifier).await;
            let (consumer_to_issuer, issuer_to_consumer) = create_connected_connections(&mut consumer, &mut issuer).await;

            let (schema_id, _schema_json, cred_def_id, _cred_def_json, cred_def, rev_reg, _) =
                _create_address_schema(&issuer.profile, &issuer.institution_did).await;
            let (address1, address2, city, state, zip) = attr_names();
            let (req1, req2) = (Some("request1"), Some("request2"));
            let credential_data1 = json!({address1.clone(): "123 Main St", address2.clone(): "Suite 3", city.clone(): "Draper", state.clone(): "UT", zip.clone(): "84000"}).to_string();
            let issuer_credential1 = _exchange_credential(
                &mut consumer,
                &mut issuer,
                credential_data1.clone(),
                &cred_def,
                &rev_reg,
                &consumer_to_issuer,
                &issuer_to_consumer,
                req1,
            )
                .await;

            #[cfg(feature = "migration")]
            issuer.migrate().await;

            let rev_reg_2 = rotate_rev_reg(&mut issuer, &cred_def, &rev_reg).await;
            let credential_data2 = json!({address1.clone(): "101 Tela Lane", address2.clone(): "Suite 1", city.clone(): "SLC", state.clone(): "WA", zip.clone(): "8721"}).to_string();
            let issuer_credential2 = _exchange_credential(
                &mut consumer,
                &mut issuer,
                credential_data2.clone(),
                &cred_def,
                &rev_reg_2,
                &consumer_to_issuer,
                &issuer_to_consumer,
                req2,
            )
                .await;

            let mut proof_verifier = verifier_create_proof_and_send_request(&mut verifier, &verifier_to_consumer, &schema_id, &cred_def_id, req1).await;
            prover_select_credentials_and_send_proof(&mut consumer, &consumer_to_verifier, req1, Some(&credential_data1)).await;
            proof_verifier
                .update_state(
                    &verifier.profile.inject_wallet(),
                    &verifier.profile.inject_anoncreds_ledger_read(),
                    &verifier.profile.inject_anoncreds(),
                    &verifier.agency_client,
                    &verifier_to_consumer,
                )
                .await
                .unwrap();
            assert_eq!(proof_verifier.get_verification_status(), PresentationVerificationStatus::Valid);

            #[cfg(feature = "migration")]
            verifier.migrate().await;

            let mut proof_verifier = verifier_create_proof_and_send_request(&mut verifier, &verifier_to_consumer, &schema_id, &cred_def_id, req2).await;

            #[cfg(feature = "migration")]
            consumer.migrate().await;

            prover_select_credentials_and_send_proof(&mut consumer, &consumer_to_verifier, req2, Some(&credential_data2)).await;
            proof_verifier
                .update_state(
                    &verifier.profile.inject_wallet(),
                    &verifier.profile.inject_anoncreds_ledger_read(),
                    &verifier.profile.inject_anoncreds(),
                    &verifier.agency_client,
                    &verifier_to_consumer,
                )
                .await
                .unwrap();
            assert_eq!(proof_verifier.get_verification_status(), PresentationVerificationStatus::Valid);

            assert!(!issuer_credential1.is_revoked(&issuer.profile.inject_anoncreds_ledger_read()).await.unwrap());
            assert!(!issuer_credential2.is_revoked(&issuer.profile.inject_anoncreds_ledger_read()).await.unwrap());
        })
            .await;
    }

    #[tokio::test]
    #[ignore]
    async fn test_agency_pool_two_creds_two_rev_reg_id_revoke_first() {
        SetupPoolDirectory::run(|setup| async move {
            let mut issuer = create_faber(setup.genesis_file_path.clone()).await;
            let mut verifier = create_faber(setup.genesis_file_path.clone()).await;
            let mut consumer = create_alice(setup.genesis_file_path).await;

            let (consumer_to_verifier, verifier_to_consumer) = create_connected_connections(&mut consumer, &mut verifier).await;
            let (consumer_to_issuer, issuer_to_consumer) = create_connected_connections(&mut consumer, &mut issuer).await;

            let (schema_id, _schema_json, cred_def_id, _cred_def_json, cred_def, rev_reg, _) =
                _create_address_schema(&issuer.profile, &issuer.institution_did).await;
            let (address1, address2, city, state, zip) = attr_names();
            let (req1, req2) = (Some("request1"), Some("request2"));
            let credential_data1 = json!({address1.clone(): "123 Main St", address2.clone(): "Suite 3", city.clone(): "Draper", state.clone(): "UT", zip.clone(): "84000"}).to_string();
            let issuer_credential1 = _exchange_credential(
                &mut consumer,
                &mut issuer,
                credential_data1.clone(),
                &cred_def,
                &rev_reg,
                &consumer_to_issuer,
                &issuer_to_consumer,
                req1,
            )
                .await;

            #[cfg(feature = "migration")]
            issuer.migrate().await;

            let rev_reg_2 = rotate_rev_reg(&mut issuer, &cred_def, &rev_reg).await;
            let credential_data2 = json!({address1.clone(): "101 Tela Lane", address2.clone(): "Suite 1", city.clone(): "SLC", state.clone(): "WA", zip.clone(): "8721"}).to_string();
            let issuer_credential2 = _exchange_credential(
                &mut consumer,
                &mut issuer,
                credential_data2.clone(),
                &cred_def,
                &rev_reg_2,
                &consumer_to_issuer,
                &issuer_to_consumer,
                req2,
            )
                .await;

            assert!(!issuer_credential1.is_revoked(&issuer.profile.inject_anoncreds_ledger_read()).await.unwrap());
            assert!(!issuer_credential2.is_revoked(&issuer.profile.inject_anoncreds_ledger_read()).await.unwrap());

            revoke_credential_and_publish_accumulator(&mut issuer, &issuer_credential1, &rev_reg).await;

            #[cfg(feature = "migration")]
            verifier.migrate().await;

            let mut proof_verifier = verifier_create_proof_and_send_request(&mut verifier, &verifier_to_consumer, &schema_id, &cred_def_id, req1).await;
            prover_select_credentials_and_send_proof(&mut consumer, &consumer_to_verifier, req1, Some(&credential_data1)).await;
            proof_verifier
                .update_state(
                    &verifier.profile.inject_wallet(),
                    &verifier.profile.inject_anoncreds_ledger_read(),
                    &verifier.profile.inject_anoncreds(),
                    &verifier.agency_client,
                    &verifier_to_consumer,
                )
                .await
                .unwrap();
            assert_eq!(proof_verifier.get_verification_status(), PresentationVerificationStatus::Invalid);

            #[cfg(feature = "migration")]
            consumer.migrate().await;

            let mut proof_verifier = verifier_create_proof_and_send_request(&mut verifier, &verifier_to_consumer, &schema_id, &cred_def_id, req2).await;
            prover_select_credentials_and_send_proof(&mut consumer, &consumer_to_verifier, req2, Some(&credential_data2)).await;
            proof_verifier
                .update_state(
                    &verifier.profile.inject_wallet(),
                    &verifier.profile.inject_anoncreds_ledger_read(),
                    &verifier.profile.inject_anoncreds(),
                    &verifier.agency_client,
                    &verifier_to_consumer,
                )
                .await
                .unwrap();
            assert_eq!(proof_verifier.get_verification_status(), PresentationVerificationStatus::Valid);

            assert!(issuer_credential1.is_revoked(&issuer.profile.inject_anoncreds_ledger_read()).await.unwrap());
            assert!(!issuer_credential2.is_revoked(&issuer.profile.inject_anoncreds_ledger_read()).await.unwrap());
        })
            .await;
    }

    #[tokio::test]
    #[ignore]
    async fn test_agency_pool_two_creds_two_rev_reg_id_revoke_second() {
        SetupPoolDirectory::run(|setup| async move {
            let mut issuer = create_faber(setup.genesis_file_path.clone()).await;
            let mut verifier = create_faber(setup.genesis_file_path.clone()).await;
            let mut consumer = create_alice(setup.genesis_file_path).await;

            let (consumer_to_verifier, verifier_to_consumer) = create_connected_connections(&mut consumer, &mut verifier).await;
            let (consumer_to_issuer, issuer_to_consumer) = create_connected_connections(&mut consumer, &mut issuer).await;

            let (schema_id, _schema_json, cred_def_id, _cred_def_json, cred_def, rev_reg, _) =
                _create_address_schema(&issuer.profile, &issuer.institution_did).await;
            let (address1, address2, city, state, zip) = attr_names();
            let (req1, req2) = (Some("request1"), Some("request2"));
            let credential_data1 = json!({address1.clone(): "123 Main St", address2.clone(): "Suite 3", city.clone(): "Draper", state.clone(): "UT", zip.clone(): "84000"}).to_string();
            let issuer_credential1 = _exchange_credential(
                &mut consumer,
                &mut issuer,
                credential_data1.clone(),
                &cred_def,
                &rev_reg,
                &consumer_to_issuer,
                &issuer_to_consumer,
                req1,
            )
                .await;

            #[cfg(feature = "migration")]
            issuer.migrate().await;

            let rev_reg_2 = rotate_rev_reg(&mut issuer, &cred_def, &rev_reg).await;
            let credential_data2 = json!({address1.clone(): "101 Tela Lane", address2.clone(): "Suite 1", city.clone(): "SLC", state.clone(): "WA", zip.clone(): "8721"}).to_string();
            let issuer_credential2 = _exchange_credential(
                &mut consumer,
                &mut issuer,
                credential_data2.clone(),
                &cred_def,
                &rev_reg_2,
                &consumer_to_issuer,
                &issuer_to_consumer,
                req2,
            )
                .await;

            assert!(!issuer_credential1.is_revoked(&issuer.profile.inject_anoncreds_ledger_read()).await.unwrap());
            assert!(!issuer_credential2.is_revoked(&issuer.profile.inject_anoncreds_ledger_read()).await.unwrap());

            revoke_credential_and_publish_accumulator(&mut issuer, &issuer_credential2, &rev_reg_2).await;

            let mut proof_verifier = verifier_create_proof_and_send_request(&mut verifier, &verifier_to_consumer, &schema_id, &cred_def_id, req1).await;
            prover_select_credentials_and_send_proof(&mut consumer, &consumer_to_verifier, req1, Some(&credential_data1)).await;

            #[cfg(feature = "migration")]
            verifier.migrate().await;

            proof_verifier
                .update_state(
                    &verifier.profile.inject_wallet(),
                    &verifier.profile.inject_anoncreds_ledger_read(),
                    &verifier.profile.inject_anoncreds(),
                    &verifier.agency_client,
                    &verifier_to_consumer,
                )
                .await
                .unwrap();
            assert_eq!(proof_verifier.get_verification_status(), PresentationVerificationStatus::Valid);

            let mut proof_verifier = verifier_create_proof_and_send_request(&mut verifier, &verifier_to_consumer, &schema_id, &cred_def_id, req2).await;

            #[cfg(feature = "migration")]
            consumer.migrate().await;

            prover_select_credentials_and_send_proof(&mut consumer, &consumer_to_verifier, req2, Some(&credential_data2)).await;
            proof_verifier
                .update_state(
                    &verifier.profile.inject_wallet(),
                    &verifier.profile.inject_anoncreds_ledger_read(),
                    &verifier.profile.inject_anoncreds(),
                    &verifier.agency_client,
                    &verifier_to_consumer,
                )
                .await
                .unwrap();
            assert_eq!(proof_verifier.get_verification_status(), PresentationVerificationStatus::Invalid);

            assert!(!issuer_credential1.is_revoked(&issuer.profile.inject_anoncreds_ledger_read()).await.unwrap());
            assert!(issuer_credential2.is_revoked(&issuer.profile.inject_anoncreds_ledger_read()).await.unwrap());
        })
            .await;
    }

    #[tokio::test]
    #[ignore]
    async fn test_agency_pool_three_creds_one_rev_reg_revoke_all() {
        SetupPoolDirectory::run(|setup| async move {
            let mut issuer = create_faber(setup.genesis_file_path.clone()).await;
            let mut consumer = create_alice(setup.genesis_file_path.clone()).await;

            let (consumer_to_issuer, issuer_to_consumer) =
                create_connected_connections(&mut consumer, &mut issuer).await;

            let (_schema_id, _schema_json, _cred_def_id, _cred_def_json, cred_def, rev_reg, _rev_reg_id) =
                _create_address_schema(&issuer.profile, &issuer.institution_did).await;

            let (address1, address2, city, state, zip) = attr_names();
            let (req1, req2, req3) = (Some("request1"), Some("request2"), Some("request3"));

            let credential_data1 = json!({address1.clone(): "123 Main St", address2.clone(): "Suite 3", city.clone(): "Draper", state.clone(): "UT", zip.clone(): "84000"}).to_string();
            let issuer_credential1 = _exchange_credential(
                &mut consumer,
                &mut issuer,
                credential_data1.clone(),
                &cred_def,
                &rev_reg,
                &consumer_to_issuer,
                &issuer_to_consumer,
                req1,
            )
                .await;

            assert!(!issuer_credential1
                .is_revoked(&issuer.profile.inject_anoncreds_ledger_read())
                .await
                .unwrap());
            revoke_credential_local(&mut issuer, &issuer_credential1, &rev_reg.rev_reg_id).await;

            let credential_data2 = json!({address1.clone(): "101 Tela Lane", address2.clone(): "Suite 1", city.clone(): "SLC", state.clone(): "WA", zip.clone(): "8721"}).to_string();

            let issuer_credential2 = _exchange_credential(
                &mut consumer,
                &mut issuer,
                credential_data2.clone(),
                &cred_def,
                &rev_reg,
                &consumer_to_issuer,
                &issuer_to_consumer,
                req2,
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

            let credential_data3 = json!({address1.clone(): "221 Baker Street", address2.clone(): "Apt. B", city.clone(): "London", state.clone(): "N/A", zip.clone(): "NW1 6XE."}).to_string();

            let issuer_credential3 = _exchange_credential(
                &mut consumer,
                &mut issuer,
                credential_data3.clone(),
                &cred_def,
                &rev_reg,
                &consumer_to_issuer,
                &issuer_to_consumer,
                req3,
            )
                .await;

            revoke_credential_and_publish_accumulator(&mut issuer, &issuer_credential3, &rev_reg).await;

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
        }).await;
    }
}
