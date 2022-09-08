#[macro_use]
extern crate log;
extern crate serde;
extern crate serde_derive;
#[macro_use]
extern crate serde_json;
extern crate tokio;

pub mod utils;

#[cfg(test)]
#[cfg(feature = "agency_pool_tests")]
mod integration_tests {
    use std::thread;
    use std::time::Duration;

    use aries_vcx::global::settings;
    use aries_vcx::protocols::proof_presentation::prover::state_machine::ProverState;
    use aries_vcx::utils::devsetup::*;

    use crate::utils::devsetup_agent::test_utils::{Alice, Faber};
    use crate::utils::scenarios::test_utils::{
        _create_address_schema, _exchange_credential, attr_names, create_connected_connections, create_proof,
        generate_and_send_proof, issue_address_credential, prover_select_credentials_and_send_proof,
        publish_revocation, requested_attrs, retrieved_to_selected_credentials_simple, revoke_credential,
        revoke_credential_local, rotate_rev_reg, send_proof_request, verifier_create_proof_and_send_request,
    };
    use crate::utils::test_macros::ProofStateType;

    use super::*;

    #[cfg(feature = "agency_pool_tests")]
    #[tokio::test]
    async fn test_basic_revocation() {
        let _setup = SetupLibraryAgencyV2::init().await;
        let mut institution = Faber::setup().await;
        let mut consumer = Alice::setup().await;

        let (consumer_to_institution, institution_to_consumer) =
            create_connected_connections(&mut consumer, &mut institution).await;
        let (schema_id, cred_def_id, _, _cred_def, rev_reg, credential_handle) = issue_address_credential(
            &mut consumer,
            &mut institution,
            &consumer_to_institution,
            &institution_to_consumer,
        )
        .await;

        let time_before_revocation = time::get_time().sec as u64;
        info!("test_basic_revocation :: verifier :: Going to revoke credential");
        revoke_credential(&mut institution, &credential_handle, rev_reg.rev_reg_id).await;
        thread::sleep(Duration::from_millis(2000));
        let time_after_revocation = time::get_time().sec as u64;

        let _requested_attrs = requested_attrs(
            &institution.config_issuer.institution_did,
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
                institution.wallet_handle,
                institution.pool_handle,
                &institution.agency_client,
                &institution_to_consumer,
            )
            .await
            .unwrap();
        assert_eq!(verifier.get_presentation_status(), ProofStateType::ProofInvalid as u32);
    }

    #[cfg(feature = "agency_pool_tests")]
    #[tokio::test]
    async fn test_local_revocation() {
        let _setup = SetupLibraryAgencyV2::init().await;
        let mut institution = Faber::setup().await;
        let mut consumer = Alice::setup().await;

        let (consumer_to_institution, institution_to_consumer) =
            create_connected_connections(&mut consumer, &mut institution).await;
        let (schema_id, cred_def_id, _, _cred_def, rev_reg, issuer_credential) = issue_address_credential(
            &mut consumer,
            &mut institution,
            &consumer_to_institution,
            &institution_to_consumer,
        )
        .await;

        revoke_credential_local(&mut institution, &issuer_credential, rev_reg.rev_reg_id.clone()).await;
        let request_name1 = Some("request1");
        let mut verifier = verifier_create_proof_and_send_request(
            &mut institution,
            &institution_to_consumer,
            &schema_id,
            &cred_def_id,
            request_name1,
        )
        .await;
        prover_select_credentials_and_send_proof(&mut consumer, &consumer_to_institution, request_name1, None).await;

        verifier
            .update_state(
                institution.wallet_handle,
                institution.pool_handle,
                &institution.agency_client,
                &institution_to_consumer,
            )
            .await
            .unwrap();
        assert_eq!(
            verifier.get_presentation_status(),
            ProofStateType::ProofValidated as u32
        );

        publish_revocation(&mut institution, rev_reg.rev_reg_id.clone()).await;
        let request_name2 = Some("request2");
        let mut verifier = verifier_create_proof_and_send_request(
            &mut institution,
            &institution_to_consumer,
            &schema_id,
            &cred_def_id,
            request_name2,
        )
        .await;
        prover_select_credentials_and_send_proof(&mut consumer, &consumer_to_institution, request_name2, None).await;

        verifier
            .update_state(
                institution.wallet_handle,
                institution.pool_handle,
                &institution.agency_client,
                &institution_to_consumer,
            )
            .await
            .unwrap();
        assert_eq!(verifier.get_presentation_status(), ProofStateType::ProofInvalid as u32);
    }

    #[cfg(feature = "agency_pool_tests")]
    #[tokio::test]
    async fn test_batch_revocation() {
        let _setup = SetupLibraryAgencyV2::init().await;
        let mut institution = Faber::setup().await;
        let mut consumer1 = Alice::setup().await;
        let mut consumer2 = Alice::setup().await;
        let mut consumer3 = Alice::setup().await;
        let (consumer_to_institution1, institution_to_consumer1) =
            create_connected_connections(&mut consumer1, &mut institution).await;
        let (consumer_to_institution2, institution_to_consumer2) =
            create_connected_connections(&mut consumer2, &mut institution).await;
        let (consumer_to_institution3, institution_to_consumer3) =
            create_connected_connections(&mut consumer3, &mut institution).await;

        // Issue and send three credentials of the same schema
        let (schema_id, _schema_json, cred_def_id, _cred_def_json, cred_def, rev_reg, rev_reg_id) =
            _create_address_schema(institution.wallet_handle, institution.pool_handle, &institution.config_issuer.institution_did).await;
        let (address1, address2, city, state, zip) = attr_names();
        let credential_data1 = json!({address1.clone(): "123 Main St", address2.clone(): "Suite 3", city.clone(): "Draper", state.clone(): "UT", zip.clone(): "84000"}).to_string();
        let credential_handle1 = _exchange_credential(
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
        let credential_data2 = json!({address1.clone(): "101 Tela Lane", address2.clone(): "Suite 1", city.clone(): "SLC", state.clone(): "WA", zip.clone(): "8721"}).to_string();
        let credential_handle2 = _exchange_credential(
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
        let credential_data3 = json!({address1.clone(): "5th Avenue", address2.clone(): "Suite 1234", city.clone(): "NYC", state.clone(): "NYS", zip.clone(): "84712"}).to_string();
        let _credential_handle3 = _exchange_credential(
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

        revoke_credential_local(&mut institution, &credential_handle1, rev_reg.rev_reg_id.clone()).await;
        revoke_credential_local(&mut institution, &credential_handle2, rev_reg.rev_reg_id.clone()).await;

        // Revoke two locally and verify their are all still valid
        let request_name1 = Some("request1");
        let mut verifier1 = verifier_create_proof_and_send_request(
            &mut institution,
            &institution_to_consumer1,
            &schema_id,
            &cred_def_id,
            request_name1,
        )
        .await;
        prover_select_credentials_and_send_proof(&mut consumer1, &consumer_to_institution1, request_name1, None).await;
        let mut verifier2 = verifier_create_proof_and_send_request(
            &mut institution,
            &institution_to_consumer2,
            &schema_id,
            &cred_def_id,
            request_name1,
        )
        .await;
        prover_select_credentials_and_send_proof(&mut consumer2, &consumer_to_institution2, request_name1, None).await;
        let mut verifier3 = verifier_create_proof_and_send_request(
            &mut institution,
            &institution_to_consumer3,
            &schema_id,
            &cred_def_id,
            request_name1,
        )
        .await;
        prover_select_credentials_and_send_proof(&mut consumer3, &consumer_to_institution3, request_name1, None).await;

        verifier1
            .update_state(
                institution.wallet_handle,
                institution.pool_handle,
                &institution.agency_client,
                &institution_to_consumer1,
            )
            .await
            .unwrap();
        verifier2
            .update_state(
                institution.wallet_handle,
                institution.pool_handle,
                &institution.agency_client,
                &institution_to_consumer2,
            )
            .await
            .unwrap();
        verifier3
            .update_state(
                institution.wallet_handle,
                institution.pool_handle,
                &institution.agency_client,
                &institution_to_consumer3,
            )
            .await
            .unwrap();
        assert_eq!(
            verifier1.get_presentation_status(),
            ProofStateType::ProofValidated as u32
        );
        assert_eq!(
            verifier2.get_presentation_status(),
            ProofStateType::ProofValidated as u32
        );
        assert_eq!(
            verifier3.get_presentation_status(),
            ProofStateType::ProofValidated as u32
        );

        // Publish revocations and verify the two are invalid, third still valid
        publish_revocation(&mut institution, rev_reg_id.clone().unwrap()).await;
        thread::sleep(Duration::from_millis(2000));
        let request_name2 = Some("request2");
        let mut verifier1 = verifier_create_proof_and_send_request(
            &mut institution,
            &institution_to_consumer1,
            &schema_id,
            &cred_def_id,
            request_name2,
        )
        .await;
        prover_select_credentials_and_send_proof(&mut consumer1, &consumer_to_institution1, request_name2, None).await;
        let mut verifier2 = verifier_create_proof_and_send_request(
            &mut institution,
            &institution_to_consumer2,
            &schema_id,
            &cred_def_id,
            request_name2,
        )
        .await;
        prover_select_credentials_and_send_proof(&mut consumer2, &consumer_to_institution2, request_name2, None).await;
        let mut verifier3 = verifier_create_proof_and_send_request(
            &mut institution,
            &institution_to_consumer3,
            &schema_id,
            &cred_def_id,
            request_name2,
        )
        .await;
        prover_select_credentials_and_send_proof(&mut consumer3, &consumer_to_institution3, request_name2, None).await;
        assert_ne!(verifier1, verifier2);
        assert_ne!(verifier1, verifier3);
        assert_ne!(verifier2, verifier3);

        verifier1
            .update_state(
                institution.wallet_handle,
                institution.pool_handle,
                &institution.agency_client,
                &institution_to_consumer1,
            )
            .await
            .unwrap();
        verifier2
            .update_state(
                institution.wallet_handle,
                institution.pool_handle,
                &institution.agency_client,
                &institution_to_consumer2,
            )
            .await
            .unwrap();
        verifier3
            .update_state(
                institution.wallet_handle,
                institution.pool_handle,
                &institution.agency_client,
                &institution_to_consumer3,
            )
            .await
            .unwrap();
        assert_eq!(verifier1.get_presentation_status(), ProofStateType::ProofInvalid as u32);
        assert_eq!(verifier2.get_presentation_status(), ProofStateType::ProofInvalid as u32);
        assert_eq!(
            verifier3.get_presentation_status(),
            ProofStateType::ProofValidated as u32
        );
    }

    #[cfg(feature = "agency_pool_tests")]
    #[tokio::test]
    async fn test_revoked_credential_might_still_work() {
        let _setup = SetupLibraryAgencyV2::init().await;
        let mut institution = Faber::setup().await;
        let mut consumer = Alice::setup().await;

        let (consumer_to_institution, institution_to_consumer) =
            create_connected_connections(&mut consumer, &mut institution).await;
        let (schema_id, cred_def_id, _, _cred_def, rev_reg, credential_handle) = issue_address_credential(
            &mut consumer,
            &mut institution,
            &consumer_to_institution,
            &institution_to_consumer,
        )
        .await;

        thread::sleep(Duration::from_millis(1000));
        let time_before_revocation = time::get_time().sec as u64;
        thread::sleep(Duration::from_millis(2000));
        info!("test_revoked_credential_might_still_work :: verifier :: Going to revoke credential");
        revoke_credential(&mut institution, &credential_handle, rev_reg.rev_reg_id.clone()).await;
        thread::sleep(Duration::from_millis(2000));

        let from = time_before_revocation - 100;
        let to = time_before_revocation;
        let _requested_attrs = requested_attrs(&institution.config_issuer.institution_did, &schema_id, &cred_def_id, Some(from), Some(to));
        let interval = json!({"from": from, "to": to}).to_string();
        let requested_attrs_string = serde_json::to_string(&_requested_attrs).unwrap();

        info!(
            "test_revoked_credential_might_still_work :: Going to seng proof request with attributes {}",
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

        info!("test_revoked_credential_might_still_work :: Going to create proof");
        let mut prover = create_proof(&mut consumer, &consumer_to_institution, None).await;
        info!("test_revoked_credential_might_still_work :: retrieving matching credentials");

        let retrieved_credentials = prover.retrieve_credentials(consumer.wallet_handle).await.unwrap();
        info!(
            "test_revoked_credential_might_still_work :: prover :: based on proof, retrieved credentials: {}",
            &retrieved_credentials
        );

        let selected_credentials_value = retrieved_to_selected_credentials_simple(&retrieved_credentials, true);
        let selected_credentials_str = serde_json::to_string(&selected_credentials_value).unwrap();
        info!(
            "test_revoked_credential_might_still_work :: prover :: retrieved credential converted to selected: {}",
            &selected_credentials_str
        );
        generate_and_send_proof(
            &mut consumer,
            &mut prover,
            &consumer_to_institution,
            &selected_credentials_str,
        )
        .await;
        assert_eq!(ProverState::PresentationSent, prover.get_state());

        info!("test_revoked_credential_might_still_work :: verifier :: going to verify proof");
        verifier
            .update_state(
                institution.wallet_handle,
                institution.pool_handle,
                &institution.agency_client,
                &institution_to_consumer,
            )
            .await
            .unwrap();
        assert_eq!(
            verifier.get_presentation_status(),
            ProofStateType::ProofValidated as u32
        );
    }

    #[tokio::test]
    #[cfg(feature = "agency_pool_tests")]
    async fn test_two_creds_one_rev_reg_revoke_first() {
        let _setup = SetupLibraryAgencyV2::init().await;
        let mut issuer = Faber::setup().await;
        let mut verifier = Faber::setup().await;
        let mut consumer = Alice::setup().await;
        let (consumer_to_verifier, verifier_to_consumer) =
            create_connected_connections(&mut consumer, &mut verifier).await;
        let (consumer_to_issuer, issuer_to_consumer) = create_connected_connections(&mut consumer, &mut issuer).await;

        let (schema_id, _schema_json, cred_def_id, _cred_def_json, cred_def, rev_reg, rev_reg_id) =
            _create_address_schema(issuer.wallet_handle, issuer.pool_handle, &issuer.config_issuer.institution_did).await;
        let (address1, address2, city, state, zip) = attr_names();
        let (req1, req2) = (Some("request1"), Some("request2"));
        let credential_data1 = json!({address1.clone(): "123 Main St", address2.clone(): "Suite 3", city.clone(): "Draper", state.clone(): "UT", zip.clone(): "84000"}).to_string();
        let credential_handle1 = _exchange_credential(
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
        let credential_data2 = json!({address1.clone(): "101 Tela Lane", address2.clone(): "Suite 1", city.clone(): "SLC", state.clone(): "WA", zip.clone(): "8721"}).to_string();
        let _credential_handle2 = _exchange_credential(
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

        revoke_credential(&mut issuer, &credential_handle1, rev_reg_id.unwrap()).await;

        let mut proof_verifier = verifier_create_proof_and_send_request(
            &mut verifier,
            &verifier_to_consumer,
            &schema_id,
            &cred_def_id,
            req1,
        )
        .await;
        prover_select_credentials_and_send_proof(&mut consumer, &consumer_to_verifier, req1, Some(&credential_data1))
            .await;
        proof_verifier
            .update_state(verifier.wallet_handle, verifier.pool_handle, &verifier.agency_client, &verifier_to_consumer)
            .await
            .unwrap();
        assert_eq!(
            proof_verifier.get_presentation_status(),
            ProofStateType::ProofInvalid as u32
        );

        let mut proof_verifier = verifier_create_proof_and_send_request(
            &mut verifier,
            &verifier_to_consumer,
            &schema_id,
            &cred_def_id,
            req2,
        )
        .await;
        prover_select_credentials_and_send_proof(&mut consumer, &consumer_to_verifier, req2, Some(&credential_data2))
            .await;
        proof_verifier
            .update_state(verifier.wallet_handle, verifier.pool_handle, &verifier.agency_client, &verifier_to_consumer)
            .await
            .unwrap();
        assert_eq!(
            proof_verifier.get_presentation_status(),
            ProofStateType::ProofValidated as u32
        );
    }

    #[tokio::test]
    #[cfg(feature = "agency_pool_tests")]
    async fn test_two_creds_one_rev_reg_revoke_second() {
        let _setup = SetupLibraryAgencyV2::init().await;
        let mut issuer = Faber::setup().await;
        let mut verifier = Faber::setup().await;
        let mut consumer = Alice::setup().await;
        let (consumer_to_verifier, verifier_to_consumer) =
            create_connected_connections(&mut consumer, &mut verifier).await;
        let (consumer_to_issuer, issuer_to_consumer) = create_connected_connections(&mut consumer, &mut issuer).await;

        let (schema_id, _schema_json, cred_def_id, _cred_def_json, cred_def, rev_reg, rev_reg_id) =
            _create_address_schema(issuer.wallet_handle, issuer.pool_handle, &issuer.config_issuer.institution_did).await;
        let (address1, address2, city, state, zip) = attr_names();
        let (req1, req2) = (Some("request1"), Some("request2"));
        let credential_data1 = json!({address1.clone(): "123 Main St", address2.clone(): "Suite 3", city.clone(): "Draper", state.clone(): "UT", zip.clone(): "84000"}).to_string();
        let _credential_handle1 = _exchange_credential(
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
        let credential_data2 = json!({address1.clone(): "101 Tela Lane", address2.clone(): "Suite 1", city.clone(): "SLC", state.clone(): "WA", zip.clone(): "8721"}).to_string();
        let credential_handle2 = _exchange_credential(
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

        revoke_credential(&mut issuer, &credential_handle2, rev_reg_id.unwrap()).await;

        let mut proof_verifier = verifier_create_proof_and_send_request(
            &mut verifier,
            &verifier_to_consumer,
            &schema_id,
            &cred_def_id,
            req1,
        )
        .await;
        prover_select_credentials_and_send_proof(&mut consumer, &consumer_to_verifier, req1, Some(&credential_data1))
            .await;
        proof_verifier
            .update_state(verifier.wallet_handle, verifier.pool_handle, &verifier.agency_client, &verifier_to_consumer)
            .await
            .unwrap();
        assert_eq!(
            proof_verifier.get_presentation_status(),
            ProofStateType::ProofValidated as u32
        );

        let mut proof_verifier = verifier_create_proof_and_send_request(
            &mut verifier,
            &verifier_to_consumer,
            &schema_id,
            &cred_def_id,
            req2,
        )
        .await;
        prover_select_credentials_and_send_proof(&mut consumer, &consumer_to_verifier, req2, Some(&credential_data2))
            .await;
        proof_verifier
            .update_state(verifier.wallet_handle, verifier.pool_handle, &verifier.agency_client, &verifier_to_consumer)
            .await
            .unwrap();
        assert_eq!(
            proof_verifier.get_presentation_status(),
            ProofStateType::ProofInvalid as u32
        );
    }

    #[tokio::test]
    #[cfg(feature = "agency_pool_tests")]
    async fn test_two_creds_two_rev_reg_id() {
        let _setup = SetupLibraryAgencyV2::init().await;
        let mut issuer = Faber::setup().await;
        let mut verifier = Faber::setup().await;
        let mut consumer = Alice::setup().await;
        let (consumer_to_verifier, verifier_to_consumer) =
            create_connected_connections(&mut consumer, &mut verifier).await;
        let (consumer_to_issuer, issuer_to_consumer) = create_connected_connections(&mut consumer, &mut issuer).await;

        let (schema_id, _schema_json, cred_def_id, _cred_def_json, cred_def, rev_reg, _) =
            _create_address_schema(issuer.wallet_handle, issuer.pool_handle, &issuer.config_issuer.institution_did).await;
        let (address1, address2, city, state, zip) = attr_names();
        let (req1, req2) = (Some("request1"), Some("request2"));
        let credential_data1 = json!({address1.clone(): "123 Main St", address2.clone(): "Suite 3", city.clone(): "Draper", state.clone(): "UT", zip.clone(): "84000"}).to_string();
        let _credential_handle1 = _exchange_credential(
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
        let rev_reg_2 = rotate_rev_reg(&mut issuer, &cred_def, &rev_reg).await;
        let credential_data2 = json!({address1.clone(): "101 Tela Lane", address2.clone(): "Suite 1", city.clone(): "SLC", state.clone(): "WA", zip.clone(): "8721"}).to_string();
        let _credential_handle2 = _exchange_credential(
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

        let mut proof_verifier = verifier_create_proof_and_send_request(
            &mut verifier,
            &verifier_to_consumer,
            &schema_id,
            &cred_def_id,
            req1,
        )
        .await;
        prover_select_credentials_and_send_proof(&mut consumer, &consumer_to_verifier, req1, Some(&credential_data1))
            .await;
        proof_verifier
            .update_state(verifier.wallet_handle, verifier.pool_handle, &verifier.agency_client, &verifier_to_consumer)
            .await
            .unwrap();
        assert_eq!(
            proof_verifier.get_presentation_status(),
            ProofStateType::ProofValidated as u32
        );

        let mut proof_verifier = verifier_create_proof_and_send_request(
            &mut verifier,
            &verifier_to_consumer,
            &schema_id,
            &cred_def_id,
            req2,
        )
        .await;
        prover_select_credentials_and_send_proof(&mut consumer, &consumer_to_verifier, req2, Some(&credential_data2))
            .await;
        proof_verifier
            .update_state(verifier.wallet_handle, verifier.pool_handle, &verifier.agency_client, &verifier_to_consumer)
            .await
            .unwrap();
        assert_eq!(
            proof_verifier.get_presentation_status(),
            ProofStateType::ProofValidated as u32
        );
    }

    #[tokio::test]
    #[cfg(feature = "agency_pool_tests")]
    async fn test_two_creds_two_rev_reg_id_revoke_first() {
        let _setup = SetupLibraryAgencyV2::init().await;
        let mut issuer = Faber::setup().await;
        let mut verifier = Faber::setup().await;
        let mut consumer = Alice::setup().await;
        let (consumer_to_verifier, verifier_to_consumer) =
            create_connected_connections(&mut consumer, &mut verifier).await;
        let (consumer_to_issuer, issuer_to_consumer) = create_connected_connections(&mut consumer, &mut issuer).await;

        let (schema_id, _schema_json, cred_def_id, _cred_def_json, cred_def, rev_reg, _) =
            _create_address_schema(issuer.wallet_handle, issuer.pool_handle, &issuer.config_issuer.institution_did).await;
        let (address1, address2, city, state, zip) = attr_names();
        let (req1, req2) = (Some("request1"), Some("request2"));
        let credential_data1 = json!({address1.clone(): "123 Main St", address2.clone(): "Suite 3", city.clone(): "Draper", state.clone(): "UT", zip.clone(): "84000"}).to_string();
        let credential_handle1 = _exchange_credential(
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
        let rev_reg_2 = rotate_rev_reg(&mut issuer, &cred_def, &rev_reg).await;
        let credential_data2 = json!({address1.clone(): "101 Tela Lane", address2.clone(): "Suite 1", city.clone(): "SLC", state.clone(): "WA", zip.clone(): "8721"}).to_string();
        let _credential_handle2 = _exchange_credential(
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

        revoke_credential(&mut issuer, &credential_handle1, rev_reg.rev_reg_id).await;

        let mut proof_verifier = verifier_create_proof_and_send_request(
            &mut verifier,
            &verifier_to_consumer,
            &schema_id,
            &cred_def_id,
            req1,
        )
        .await;
        prover_select_credentials_and_send_proof(&mut consumer, &consumer_to_verifier, req1, Some(&credential_data1))
            .await;
        proof_verifier
            .update_state(verifier.wallet_handle, verifier.pool_handle, &verifier.agency_client, &verifier_to_consumer)
            .await
            .unwrap();
        assert_eq!(
            proof_verifier.get_presentation_status(),
            ProofStateType::ProofInvalid as u32
        );

        let mut proof_verifier = verifier_create_proof_and_send_request(
            &mut verifier,
            &verifier_to_consumer,
            &schema_id,
            &cred_def_id,
            req2,
        )
        .await;
        prover_select_credentials_and_send_proof(&mut consumer, &consumer_to_verifier, req2, Some(&credential_data2))
            .await;
        proof_verifier
            .update_state(verifier.wallet_handle, verifier.pool_handle, &verifier.agency_client, &verifier_to_consumer)
            .await
            .unwrap();
        assert_eq!(
            proof_verifier.get_presentation_status(),
            ProofStateType::ProofValidated as u32
        );
    }

    #[tokio::test]
    #[cfg(feature = "agency_pool_tests")]
    async fn test_two_creds_two_rev_reg_id_revoke_second() {
        let _setup = SetupLibraryAgencyV2::init().await;
        let mut issuer = Faber::setup().await;
        let mut verifier = Faber::setup().await;
        let mut consumer = Alice::setup().await;
        let (consumer_to_verifier, verifier_to_consumer) =
            create_connected_connections(&mut consumer, &mut verifier).await;
        let (consumer_to_issuer, issuer_to_consumer) = create_connected_connections(&mut consumer, &mut issuer).await;

        let (schema_id, _schema_json, cred_def_id, _cred_def_json, cred_def, rev_reg, _) =
            _create_address_schema(issuer.wallet_handle, issuer.pool_handle, &issuer.config_issuer.institution_did).await;
        let (address1, address2, city, state, zip) = attr_names();
        let (req1, req2) = (Some("request1"), Some("request2"));
        let credential_data1 = json!({address1.clone(): "123 Main St", address2.clone(): "Suite 3", city.clone(): "Draper", state.clone(): "UT", zip.clone(): "84000"}).to_string();
        let _credential_handle1 = _exchange_credential(
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
        let rev_reg_2 = rotate_rev_reg(&mut issuer, &cred_def, &rev_reg).await;
        let credential_data2 = json!({address1.clone(): "101 Tela Lane", address2.clone(): "Suite 1", city.clone(): "SLC", state.clone(): "WA", zip.clone(): "8721"}).to_string();
        let credential_handle2 = _exchange_credential(
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

        revoke_credential(&mut issuer, &credential_handle2, rev_reg_2.rev_reg_id).await;

        let mut proof_verifier = verifier_create_proof_and_send_request(
            &mut verifier,
            &verifier_to_consumer,
            &schema_id,
            &cred_def_id,
            req1,
        )
        .await;
        prover_select_credentials_and_send_proof(&mut consumer, &consumer_to_verifier, req1, Some(&credential_data1))
            .await;
        proof_verifier
            .update_state(verifier.wallet_handle, verifier.pool_handle, &verifier.agency_client, &verifier_to_consumer)
            .await
            .unwrap();
        assert_eq!(
            proof_verifier.get_presentation_status(),
            ProofStateType::ProofValidated as u32
        );

        let mut proof_verifier = verifier_create_proof_and_send_request(
            &mut verifier,
            &verifier_to_consumer,
            &schema_id,
            &cred_def_id,
            req2,
        )
        .await;
        prover_select_credentials_and_send_proof(&mut consumer, &consumer_to_verifier, req2, Some(&credential_data2))
            .await;
        proof_verifier
            .update_state(verifier.wallet_handle, verifier.pool_handle, &verifier.agency_client, &verifier_to_consumer)
            .await
            .unwrap();
        assert_eq!(
            proof_verifier.get_presentation_status(),
            ProofStateType::ProofInvalid as u32
        );
    }
}
