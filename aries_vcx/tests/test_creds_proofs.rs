#[macro_use]
extern crate log;
#[macro_use]
extern crate serde_json;

pub mod utils;

mod integration_tests {
    use std::collections::HashMap;
    use std::sync::Arc;

    use aries_vcx::common::proofs::proof_request::PresentationRequestData;
    use aries_vcx::common::test_utils::{
        create_and_store_credential, create_and_store_nonrevocable_credential,
        create_and_store_nonrevocable_credential_def, create_indy_proof,
    };
    use aries_vcx::handlers::proof_presentation::prover::Prover;
    use aries_vcx::handlers::proof_presentation::types::RetrievedCredentials;
    use aries_vcx::handlers::proof_presentation::verifier::Verifier;
    use aries_vcx::handlers::util::AttachmentId;
    use aries_vcx::protocols::proof_presentation::prover::state_machine::ProverState;
    use aries_vcx::protocols::proof_presentation::verifier::verification_status::PresentationVerificationStatus;
    use aries_vcx::utils::constants::DEFAULT_SCHEMA_ATTRS;
    use aries_vcx::utils::devsetup::SetupProfile;
    use messages::msg_fields::protocols::present_proof::request::{
        RequestPresentation, RequestPresentationContent, RequestPresentationDecorators,
    };
    use messages::AriesMessage;

    #[cfg(feature = "migration")]
    use crate::utils::migration::Migratable;

    #[tokio::test]
    #[ignore]
    async fn test_agency_pool_retrieve_credentials() {
        SetupProfile::run(|mut setup: SetupProfile| async move {
            create_and_store_nonrevocable_credential(
                &setup.profile.inject_anoncreds(),
                &setup.profile.inject_anoncreds(),
                &setup.profile.inject_anoncreds_ledger_read(),
                &setup.profile.inject_anoncreds_ledger_write(),
                &setup.institution_did,
                DEFAULT_SCHEMA_ATTRS,
            )
            .await;

            #[cfg(feature = "migration")]
            setup.migrate().await;

            let (_, _, req, _) = create_indy_proof(
                &setup.profile.inject_anoncreds(),
                &setup.profile.inject_anoncreds(),
                &setup.profile.inject_anoncreds_ledger_read(),
                &setup.profile.inject_anoncreds_ledger_write(),
                &setup.institution_did,
            )
            .await;

            let pres_req_data: PresentationRequestData = serde_json::from_str(&req).unwrap();
            let id = "test_id".to_owned();

            let attach_type = messages::decorators::attachment::AttachmentType::Base64(base64::encode(
                &json!(pres_req_data).to_string(),
            ));
            let attach_data = messages::decorators::attachment::AttachmentData::new(attach_type);
            let mut attach = messages::decorators::attachment::Attachment::new(attach_data);
            attach.id = Some(AttachmentId::PresentationRequest.as_ref().to_owned());
            attach.mime_type = Some(messages::misc::MimeType::Json);

            let content = RequestPresentationContent::new(vec![attach]);
            let decorators = RequestPresentationDecorators::default();

            let proof_req = RequestPresentation::with_decorators(id, content, decorators);
            let proof: Prover = Prover::create_from_request("1", proof_req).unwrap();

            let retrieved_creds = proof
                .retrieve_credentials(&setup.profile.inject_anoncreds())
                .await
                .unwrap();
            // assert number of cred matches for different requested referents
            assert_eq!(retrieved_creds.credentials_by_referent["address1_1"].len(), 2);
            assert_eq!(retrieved_creds.credentials_by_referent["zip_2"].len(), 2);
            assert_eq!(retrieved_creds.credentials_by_referent["self_attest_3"].len(), 0);
        })
        .await;
    }

    #[tokio::test]
    #[ignore]
    async fn test_agency_pool_get_credential_def() {
        SetupProfile::run(|mut setup| async move {
            let (_, _, cred_def_id, cred_def_json, _) = create_and_store_nonrevocable_credential_def(
                &setup.profile.inject_anoncreds(),
                &setup.profile.inject_anoncreds_ledger_read(),
                &setup.profile.inject_anoncreds_ledger_write(),
                &setup.institution_did,
                DEFAULT_SCHEMA_ATTRS,
            )
            .await;

            #[cfg(feature = "migration")]
            setup.migrate().await;

            let ledger = Arc::clone(&setup.profile).inject_anoncreds_ledger_read();
            let r_cred_def_json = ledger.get_cred_def(&cred_def_id, None).await.unwrap();

            let def1: serde_json::Value = serde_json::from_str(&cred_def_json).unwrap();
            let def2: serde_json::Value = serde_json::from_str(&r_cred_def_json).unwrap();
            assert_eq!(def1, def2);
        })
        .await;
    }

    #[tokio::test]
    #[ignore]
    async fn test_agency_pool_retrieve_credentials_empty() {
        SetupProfile::run(|mut setup| async move {
            // create skeleton proof request attachment data
            let mut req = json!({
               "nonce":"123432421212",
               "name":"proof_req_1",
               "version":"0.1",
               "requested_attributes": json!({}),
               "requested_predicates": json!({}),
            });

            let pres_req_data: PresentationRequestData = serde_json::from_str(&req.to_string()).unwrap();

            let attach_type = messages::decorators::attachment::AttachmentType::Base64(base64::encode(
                &json!(pres_req_data).to_string(),
            ));
            let attach_data = messages::decorators::attachment::AttachmentData::new(attach_type);
            let mut attach = messages::decorators::attachment::Attachment::new(attach_data);
            attach.id = Some(AttachmentId::PresentationRequest.as_ref().to_owned());
            attach.mime_type = Some(messages::misc::MimeType::Json);

            let content = RequestPresentationContent::new(vec![attach]);
            let decorators = RequestPresentationDecorators::default();

            // test retrieving credentials for empty proof request returns "{}"
            let id = "test_id".to_owned();
            let proof_req = RequestPresentation::with_decorators(id, content, decorators);
            let proof: Prover = Prover::create_from_request("1", proof_req).unwrap();

            #[cfg(feature = "migration")]
            setup.migrate().await;

            let retrieved_creds = proof
                .retrieve_credentials(&setup.profile.inject_anoncreds())
                .await
                .unwrap();
            assert_eq!(serde_json::to_string(&retrieved_creds).unwrap(), "{}".to_string());
            assert!(retrieved_creds.credentials_by_referent.is_empty());

            // populate proof request with a single attribute referent request
            req["requested_attributes"]["address1_1"] = json!({"name": "address1"});
            let pres_req_data: PresentationRequestData = serde_json::from_str(&req.to_string()).unwrap();

            let attach_type = messages::decorators::attachment::AttachmentType::Base64(base64::encode(
                &json!(pres_req_data).to_string(),
            ));
            let attach_data = messages::decorators::attachment::AttachmentData::new(attach_type);
            let mut attach = messages::decorators::attachment::Attachment::new(attach_data);
            attach.id = Some(AttachmentId::PresentationRequest.as_ref().to_owned());
            attach.mime_type = Some(messages::misc::MimeType::Json);

            let content = RequestPresentationContent::new(vec![attach]);
            let decorators = RequestPresentationDecorators::default();

            // test retrieving credentials for the proof request returns the referent with no cred matches
            let id = "test_id".to_owned();
            let proof_req = RequestPresentation::with_decorators(id, content, decorators);
            let proof: Prover = Prover::create_from_request("2", proof_req).unwrap();

            let retrieved_creds = proof
                .retrieve_credentials(&setup.profile.inject_anoncreds())
                .await
                .unwrap();
            assert_eq!(
                serde_json::to_string(&retrieved_creds).unwrap(),
                json!({"attrs":{"address1_1":[]}}).to_string()
            );
            assert_eq!(
                retrieved_creds,
                RetrievedCredentials {
                    credentials_by_referent: HashMap::from([("address1_1".to_string(), vec![])])
                }
            )
        })
        .await;
    }

    #[tokio::test]
    #[ignore]
    async fn test_agency_pool_case_for_proof_req_doesnt_matter_for_retrieve_creds() {
        SetupProfile::run(|mut setup| async move {
            create_and_store_nonrevocable_credential(
                &setup.profile.inject_anoncreds(),
                &setup.profile.inject_anoncreds(),
                &setup.profile.inject_anoncreds_ledger_read(),
                &setup.profile.inject_anoncreds_ledger_write(),
                &setup.institution_did,
                DEFAULT_SCHEMA_ATTRS,
            )
            .await;

            let mut req = json!({
               "nonce":"123432421212",
               "name":"proof_req_1",
               "version":"0.1",
               "requested_attributes": json!({
                   "zip_1": json!({
                       "name":"zip",
                       "restrictions": [json!({ "issuer_did": setup.institution_did })]
                   })
               }),
               "requested_predicates": json!({}),
            });

            let pres_req_data: PresentationRequestData = serde_json::from_str(&req.to_string()).unwrap();
            let id = "test_id".to_owned();

            let attach_type = messages::decorators::attachment::AttachmentType::Base64(base64::encode(
                &json!(pres_req_data).to_string(),
            ));
            let attach_data = messages::decorators::attachment::AttachmentData::new(attach_type);
            let mut attach = messages::decorators::attachment::Attachment::new(attach_data);
            attach.id = Some(AttachmentId::PresentationRequest.as_ref().to_owned());
            attach.mime_type = Some(messages::misc::MimeType::Json);

            let content = RequestPresentationContent::new(vec![attach]);
            let decorators = RequestPresentationDecorators::default();

            let proof_req = RequestPresentation::with_decorators(id, content, decorators);
            let proof: Prover = Prover::create_from_request("1", proof_req).unwrap();

            // All lower case
            let retrieved_creds = proof
                .retrieve_credentials(&setup.profile.inject_anoncreds())
                .await
                .unwrap();
            assert_eq!(
                retrieved_creds.credentials_by_referent["zip_1"][0].cred_info.attributes["zip"],
                "84000"
            );

            // First letter upper
            req["requested_attributes"]["zip_1"]["name"] = json!("Zip");
            let pres_req_data: PresentationRequestData = serde_json::from_str(&req.to_string()).unwrap();
            let id = "test_id".to_owned();

            let attach_type = messages::decorators::attachment::AttachmentType::Base64(base64::encode(
                &json!(pres_req_data).to_string(),
            ));
            let attach_data = messages::decorators::attachment::AttachmentData::new(attach_type);
            let mut attach = messages::decorators::attachment::Attachment::new(attach_data);
            attach.id = Some(AttachmentId::PresentationRequest.as_ref().to_owned());
            attach.mime_type = Some(messages::misc::MimeType::Json);

            let content = RequestPresentationContent::new(vec![attach]);
            let decorators = RequestPresentationDecorators::default();

            #[cfg(feature = "migration")]
            setup.migrate().await;

            let proof_req = RequestPresentation::with_decorators(id, content, decorators);
            let proof: Prover = Prover::create_from_request("2", proof_req).unwrap();
            let retrieved_creds2 = proof
                .retrieve_credentials(&setup.profile.inject_anoncreds())
                .await
                .unwrap();
            assert_eq!(
                retrieved_creds2.credentials_by_referent["zip_1"][0]
                    .cred_info
                    .attributes["zip"],
                "84000"
            );

            // Entire word upper
            req["requested_attributes"]["zip_1"]["name"] = json!("ZIP");
            let pres_req_data: PresentationRequestData = serde_json::from_str(&req.to_string()).unwrap();
            let id = "test_id".to_owned();

            let attach_type = messages::decorators::attachment::AttachmentType::Base64(base64::encode(
                &json!(pres_req_data).to_string(),
            ));
            let attach_data = messages::decorators::attachment::AttachmentData::new(attach_type);
            let mut attach = messages::decorators::attachment::Attachment::new(attach_data);
            attach.id = Some(AttachmentId::PresentationRequest.as_ref().to_owned());
            attach.mime_type = Some(messages::misc::MimeType::Json);

            let content = RequestPresentationContent::new(vec![attach]);
            let decorators = RequestPresentationDecorators::default();

            let proof_req = RequestPresentation::with_decorators(id, content, decorators);
            let proof: Prover = Prover::create_from_request("1", proof_req).unwrap();
            let retrieved_creds3 = proof
                .retrieve_credentials(&setup.profile.inject_anoncreds())
                .await
                .unwrap();
            assert_eq!(
                retrieved_creds3.credentials_by_referent["zip_1"][0]
                    .cred_info
                    .attributes["zip"],
                "84000"
            );
        })
        .await;
    }

    #[tokio::test]
    #[ignore]
    async fn test_agency_pool_generate_proof() {
        SetupProfile::run(|mut setup| async move {
            let (_, _, _, _, _, _, _, _, _, _, tails_dir, _) = create_and_store_credential(
                &setup.profile.inject_anoncreds(),
                &setup.profile.inject_anoncreds(),
                &setup.profile.inject_anoncreds_ledger_read(),
                &setup.profile.inject_anoncreds_ledger_write(),
                &setup.institution_did,
                DEFAULT_SCHEMA_ATTRS,
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
                        "restrictions": [{"issuer_did": setup.institution_did}],
                        "non_revoked":  {"from": 123, "to": to}
                    },
                    "zip_2": { "name": "zip" }
                },
                "self_attested_attr_3": json!({
                       "name":"self_attested_attr",
                 }),
                "requested_predicates": {},
                "non_revoked": {"from": 98, "to": to}
            })
            .to_string();

            #[cfg(feature = "migration")]
            setup.migrate().await;

            let pres_req_data: PresentationRequestData = serde_json::from_str(&indy_proof_req).unwrap();
            let id = "test_id".to_owned();

            let attach_type = messages::decorators::attachment::AttachmentType::Base64(base64::encode(
                &json!(pres_req_data).to_string(),
            ));
            let attach_data = messages::decorators::attachment::AttachmentData::new(attach_type);
            let mut attach = messages::decorators::attachment::Attachment::new(attach_data);
            attach.id = Some(AttachmentId::PresentationRequest.as_ref().to_owned());
            attach.mime_type = Some(messages::misc::MimeType::Json);

            let content = RequestPresentationContent::new(vec![attach]);
            let decorators = RequestPresentationDecorators::default();

            let proof_req = RequestPresentation::with_decorators(id, content, decorators);
            let mut proof: Prover = Prover::create_from_request("1", proof_req).unwrap();

            let all_creds = proof
                .retrieve_credentials(&setup.profile.inject_anoncreds())
                .await
                .unwrap();
            let selected_credentials: serde_json::Value = json!({
               "attrs":{
                  "address1_1": {
                    "credential": all_creds.credentials_by_referent["address1_1"][0],
                    "tails_dir": tails_dir.clone()
                  },
                  "zip_2": {
                    "credential": all_creds.credentials_by_referent["zip_2"][0],
                    "tails_dir": tails_dir.clone()
                  },
               }
            });

            let self_attested: serde_json::Value = json!({
                  "self_attested_attr_3":"attested_val"
            });

            let generated_proof = proof
                .generate_presentation(
                    &setup.profile.inject_anoncreds_ledger_read(),
                    &setup.profile.inject_anoncreds(),
                    serde_json::from_value(selected_credentials).unwrap(),
                    serde_json::from_value(self_attested).unwrap(),
                )
                .await;
            assert!(generated_proof.is_ok());
            assert!(matches!(proof.get_state(), ProverState::PresentationPrepared));
        })
        .await;
    }

    #[tokio::test]
    #[ignore]
    async fn test_agency_pool_generate_proof_with_predicates() {
        SetupProfile::run(|mut setup| async move {
            let (_, _, _, _, _, _, _, _, rev_reg_id, cred_rev_id, tails_dir, rev_reg) = create_and_store_credential(
                &setup.profile.inject_anoncreds(),
                &setup.profile.inject_anoncreds(),
                &setup.profile.inject_anoncreds_ledger_read(),
                &setup.profile.inject_anoncreds_ledger_write(),
                &setup.institution_did,
                DEFAULT_SCHEMA_ATTRS,
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
                        "restrictions": [{"issuer_did": setup.institution_did}],
                        "non_revoked":  {"from": 123, "to": to}
                    },
                    "state_2": { "name": "state" }
                },
                "self_attested_attr_3": json!({
                       "name":"self_attested_attr",
                 }),
                "requested_predicates": json!({
                    "zip_3": {"name":"zip", "p_type":">=", "p_value":18}
                }),
                "non_revoked": {"from": 98, "to": to}
            })
            .to_string();

            let pres_req_data: PresentationRequestData = serde_json::from_str(&indy_proof_req).unwrap();
            let id = "test_id".to_owned();

            let attach_type = messages::decorators::attachment::AttachmentType::Base64(base64::encode(
                &json!(pres_req_data).to_string(),
            ));
            let attach_data = messages::decorators::attachment::AttachmentData::new(attach_type);
            let mut attach = messages::decorators::attachment::Attachment::new(attach_data);
            attach.id = Some(AttachmentId::PresentationRequest.as_ref().to_owned());
            attach.mime_type = Some(messages::misc::MimeType::Json);

            let content = RequestPresentationContent::new(vec![attach]);
            let decorators = RequestPresentationDecorators::default();

            let proof_req = RequestPresentation::with_decorators(id, content, decorators);
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
                    "tails_dir": tails_dir
                  },
                  "state_2": {
                    "credential": all_creds.credentials_by_referent["state_2"][0],
                    "tails_dir": tails_dir
                  },
                  "zip_3": {
                    "credential": all_creds.credentials_by_referent["zip_3"][0],
                    "tails_dir": tails_dir
                  },
               },
            });
            let self_attested: serde_json::Value = json!({
                  "self_attested_attr_3":"attested_val"
            });
            let generated_proof = proof
                .generate_presentation(
                    &setup.profile.inject_anoncreds_ledger_read(),
                    &setup.profile.inject_anoncreds(),
                    serde_json::from_value(selected_credentials).unwrap(),
                    serde_json::from_value(self_attested).unwrap(),
                )
                .await;
            assert!(generated_proof.is_ok());

            assert!(matches!(proof.get_state(), ProverState::PresentationPrepared));
        })
        .await;
    }

    #[tokio::test]
    #[ignore]
    async fn test_agency_pool_generate_self_attested_proof() {
        SetupProfile::run(|mut setup| async move {
            let indy_proof_req = json!({
               "nonce":"123432421212",
               "name":"proof_req_1",
               "version":"0.1",
               "requested_attributes": json!({
                   "address1_1": json!({
                       "name":"address1",
                   }),
                   "zip_2": json!({
                       "name":"zip",
                   }),
               }),
               "requested_predicates": json!({}),
            })
            .to_string();
            let pres_req_data: PresentationRequestData = serde_json::from_str(&indy_proof_req).unwrap();
            let mut verifier = Verifier::create_from_request("foo".into(), &pres_req_data).unwrap();
            let presentation_request = verifier.get_presentation_request_msg().unwrap();
            verifier.mark_presentation_request_sent().unwrap();

            #[cfg(feature = "migration")]
            setup.migrate().await;

            // prover receives request and generates presentation
            let mut proof: Prover = Prover::create_from_request("1", presentation_request).unwrap();

            let selected_credentials: serde_json::Value = json!({});
            let self_attested: serde_json::Value = json!({
                  "address1_1":"attested_address",
                  "zip_2": "attested_zip"
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
            let presentation = proof.get_presentation_msg().unwrap();

            // verifier receives the presentation
            verifier
                .verify_presentation(
                    &setup.profile.inject_anoncreds_ledger_read(),
                    &setup.profile.inject_anoncreds(),
                    presentation,
                )
                .await
                .unwrap();

            assert_eq!(
                verifier.get_verification_status(),
                PresentationVerificationStatus::Valid
            );
        })
        .await;
    }
}

mod tests {
    use std::collections::HashMap;
    use std::time::Duration;

    use serde_json::Value;

    use crate::utils::devsetup_util::issuer_update_with_mediator;
    use crate::utils::devsetup_util::verifier_update_with_mediator;
    use aries_vcx::common::test_utils::create_and_store_nonrevocable_credential_def;
    use aries_vcx::handlers::issuance::holder::Holder;
    use aries_vcx::handlers::proof_presentation::prover::Prover;
    use aries_vcx::handlers::proof_presentation::verifier::Verifier;
    use aries_vcx::protocols::issuance::holder::state_machine::HolderState;
    use aries_vcx::protocols::issuance::issuer::state_machine::IssuerState;
    use aries_vcx::protocols::proof_presentation::prover::state_machine::ProverState;
    use aries_vcx::protocols::proof_presentation::verifier::verification_status::PresentationVerificationStatus;
    use aries_vcx::utils::devsetup::*;
    use messages::msg_fields::protocols::cred_issuance::offer_credential::OfferCredential;
    use messages::msg_fields::protocols::present_proof::request::RequestPresentation;

    use crate::utils::devsetup_alice::create_alice;
    use crate::utils::devsetup_faber::create_faber_trustee;
    use crate::utils::devsetup_util::test_utils::PayloadKinds;
    #[cfg(feature = "migration")]
    use crate::utils::migration::Migratable;
    use crate::utils::scenarios::test_utils::{
        _create_address_schema_creddef_revreg, _exchange_credential, _exchange_credential_with_proposal,
        accept_cred_proposal, accept_cred_proposal_1, accept_offer, accept_proof_proposal, attr_names,
        create_and_send_nonrevocable_cred_offer, create_connected_connections, create_proof, decline_offer,
        generate_and_send_proof, issue_address_credential, prover_select_credentials,
        prover_select_credentials_and_send_proof, receive_proof_proposal_rejection, reject_proof_proposal,
        retrieved_to_selected_credentials_simple, send_cred_proposal, send_cred_proposal_1, send_cred_req,
        send_credential, send_proof_proposal, send_proof_proposal_1, send_proof_request,
        verifier_create_proof_and_send_request, verify_proof,
    };

    #[tokio::test]
    #[ignore]
    async fn test_agency_pool_proof_should_be_validated() {
        SetupPoolDirectory::run(|setup| async move {
            let mut institution = create_faber_trustee(setup.genesis_file_path.clone()).await;
            let mut consumer = create_alice(setup.genesis_file_path).await;

            let (consumer_to_institution, institution_to_consumer) =
                create_connected_connections(&mut consumer, &mut institution).await;
            let (schema_id, cred_def_id, _rev_reg_id, _cred_def, _rev_reg, _credential_handle) =
                issue_address_credential(
                    &mut consumer,
                    &mut institution,
                    &consumer_to_institution,
                    &institution_to_consumer,
                )
                .await;

            #[cfg(feature = "migration")]
            institution.migrate().await;

            let requested_attrs_string = serde_json::to_string(&json!([
            {
                "name": "address1",
                "restrictions": [{
                  "issuer_did": institution.institution_did,
                  "schema_id": schema_id,
                  "cred_def_id": cred_def_id,
                }]
            }]))
            .unwrap();

            info!(
                "test_proof_should_be_validated :: Going to seng proof request with attributes {}",
                &requested_attrs_string
            );
            let mut verifier = send_proof_request(
                &mut institution,
                &institution_to_consumer,
                &requested_attrs_string,
                "[]",
                "{}",
                None,
            )
            .await;

            #[cfg(feature = "migration")]
            consumer.migrate().await;

            prover_select_credentials_and_send_proof(&mut consumer, &consumer_to_institution, None, None).await;

            info!("test_proof_should_be_validated :: verifier :: going to verify proof");
            verifier_update_with_mediator(
                &mut verifier,
                &institution.profile.inject_wallet(),
                &institution.profile.inject_anoncreds_ledger_read(),
                &institution.profile.inject_anoncreds(),
                &institution.agency_client,
                &institution_to_consumer,
            )
            .await
            .unwrap();
            assert_eq!(
                verifier.get_verification_status(),
                PresentationVerificationStatus::Valid
            );
        })
        .await;
    }

    #[tokio::test]
    #[ignore]
    async fn test_agency_pool_proof_with_predicates_should_be_validated() {
        SetupPoolDirectory::run(|setup| async move {
            let mut institution = create_faber_trustee(setup.genesis_file_path.clone()).await;
            let mut consumer = create_alice(setup.genesis_file_path).await;

            let (consumer_to_institution, institution_to_consumer) =
                create_connected_connections(&mut consumer, &mut institution).await;
            issue_address_credential(
                &mut consumer,
                &mut institution,
                &consumer_to_institution,
                &institution_to_consumer,
            )
            .await;

            #[cfg(feature = "migration")]
            institution.migrate().await;

            let requested_preds_string = serde_json::to_string(&json!([
            {
                "name": "zip",
                "p_type": ">=",
                "p_value": 83000
            }]))
            .unwrap();

            info!(
                "test_basic_proof :: Going to seng proof request with attributes {}",
                &requested_preds_string
            );
            let mut verifier = send_proof_request(
                &mut institution,
                &institution_to_consumer,
                "[]",
                &requested_preds_string,
                "{}",
                None,
            )
            .await;

            #[cfg(feature = "migration")]
            consumer.migrate().await;

            prover_select_credentials_and_send_proof(&mut consumer, &consumer_to_institution, None, None).await;

            info!("test_proof_with_predicates_should_be_validated :: verifier :: going to verify proof");
            verifier_update_with_mediator(
                &mut verifier,
                &institution.profile.inject_wallet(),
                &institution.profile.inject_anoncreds_ledger_read(),
                &institution.profile.inject_anoncreds(),
                &institution.agency_client,
                &institution_to_consumer,
            )
            .await
            .unwrap();
            assert_eq!(
                verifier.get_verification_status(),
                PresentationVerificationStatus::Valid
            );
            info!(
                "test_proof_with_predicates_should_be_validated :: verifier received presentation!: {}",
                verifier.get_presentation_attachment().unwrap()
            );
        })
        .await;
    }

    // todo: credx implementation does not support checking credential value in respect to predicate
    #[cfg(not(feature = "modular_libs"))]
    #[tokio::test]
    #[ignore]
    async fn test_agency_pool_it_should_fail_to_select_credentials_for_predicate() {
        SetupPoolDirectory::run(|setup| async move {
            let mut institution = create_faber_trustee(setup.genesis_file_path.clone()).await;
            let mut consumer = create_alice(setup.genesis_file_path).await;

            let (consumer_to_institution, institution_to_consumer) =
                create_connected_connections(&mut consumer, &mut institution).await;
            issue_address_credential(
                &mut consumer,
                &mut institution,
                &consumer_to_institution,
                &institution_to_consumer,
            )
            .await;

            #[cfg(feature = "migration")]
            institution.migrate().await;

            let requested_preds_string = serde_json::to_string(&json!([{
                "name": "zip",
                "p_type": ">=",
                "p_value": 85000
            }]))
            .unwrap();

            send_proof_request(
                &mut institution,
                &institution_to_consumer,
                "[]",
                &requested_preds_string,
                "{}",
                None,
            )
            .await;

            #[cfg(feature = "migration")]
            consumer.migrate().await;

            let mut prover = create_proof(&mut consumer, &consumer_to_institution, None).await;
            let selected_credentials =
                prover_select_credentials(&mut prover, &mut consumer, &consumer_to_institution, None).await;

            assert!(selected_credentials.credential_for_referent.is_empty());
        })
        .await;
    }

    #[tokio::test]
    #[ignore]
    async fn test_agency_pool_it_should_select_credentials_for_satisfiable_restriction() {
        SetupPoolDirectory::run(|setup| async move {
            let mut institution = create_faber_trustee(setup.genesis_file_path.clone()).await;
            let mut consumer = create_alice(setup.genesis_file_path).await;

            let (consumer_to_institution, institution_to_consumer) =
                create_connected_connections(&mut consumer, &mut institution).await;
            issue_address_credential(
                &mut consumer,
                &mut institution,
                &consumer_to_institution,
                &institution_to_consumer,
            )
            .await;

            let requested_attrs_string = serde_json::to_string(&json!([
            {
                "name": "address1",
                "restrictions": [{
                  "issuer_did": "abcdef0000000000000000",
                },
                {
                  "issuer_did": institution.institution_did,
                }]
            }]))
            .unwrap();

            send_proof_request(
                &mut institution,
                &institution_to_consumer,
                &requested_attrs_string,
                "[]",
                "{}",
                None,
            )
            .await;

            let mut prover = create_proof(&mut consumer, &consumer_to_institution, None).await;
            let selected_credentials =
                prover_select_credentials(&mut prover, &mut consumer, &consumer_to_institution, None).await;
            assert_eq!(selected_credentials.credential_for_referent.is_empty(), false);
        })
        .await;
    }

    #[tokio::test]
    #[ignore]
    async fn test_agency_pool_double_issuance_separate_issuer_and_consumers() {
        SetupPoolDirectory::run(|setup| async move {
            let mut issuer = create_faber_trustee(setup.genesis_file_path.clone()).await;
            let mut verifier = create_faber_trustee(setup.genesis_file_path.clone()).await;
            let mut consumer1 = create_alice(setup.genesis_file_path.clone()).await;
            let mut consumer2 = create_alice(setup.genesis_file_path).await;

            let (consumer1_to_verifier, verifier_to_consumer1) =
                create_connected_connections(&mut consumer1, &mut verifier).await;
            let (consumer1_to_issuer, issuer_to_consumer1) =
                create_connected_connections(&mut consumer1, &mut issuer).await;
            let (consumer2_to_verifier, verifier_to_consumer2) =
                create_connected_connections(&mut consumer2, &mut verifier).await;
            let (consumer2_to_issuer, issuer_to_consumer2) =
                create_connected_connections(&mut consumer2, &mut issuer).await;

            #[cfg(feature = "migration")]
            issuer.migrate().await;

            let (schema_id, _schema_json, cred_def_id, _cred_def_json, cred_def, rev_reg, _rev_reg_id) =
                _create_address_schema_creddef_revreg(&issuer.profile, &issuer.institution_did).await;
            let (address1, address2, city, state, zip) = attr_names();
            let credential_data1 = json!({address1.clone(): "123 Main St", address2.clone(): "Suite 3", city.clone(): "Draper", state.clone(): "UT", zip.clone(): "84000"}).to_string();
            let _credential_handle1 = _exchange_credential(
                &mut consumer1,
                &mut issuer,
                credential_data1,
                &cred_def,
                &rev_reg,
                &consumer1_to_issuer,
                &issuer_to_consumer1,
                None,
            )
                .await;

            #[cfg(feature = "migration")]
            verifier.migrate().await;

            let credential_data2 = json!({address1.clone(): "101 Tela Lane", address2.clone(): "Suite 1", city.clone(): "SLC", state.clone(): "WA", zip.clone(): "8721"}).to_string();
            let _credential_handle2 = _exchange_credential(
                &mut consumer2,
                &mut issuer,
                credential_data2,
                &cred_def,
                &rev_reg,
                &consumer2_to_issuer,
                &issuer_to_consumer2,
                None,
            )
                .await;

            let request_name1 = Some("request1");
            let mut proof_verifier = verifier_create_proof_and_send_request(
                &mut verifier,
                &verifier_to_consumer1,
                &schema_id,
                &cred_def_id,
                request_name1,
            )
                .await;

            #[cfg(feature = "migration")]
            consumer1.migrate().await;

            prover_select_credentials_and_send_proof(&mut consumer1, &consumer1_to_verifier, None, None).await;
                verifier_update_with_mediator(
                    &mut proof_verifier,
                    &verifier.profile.inject_wallet(),
                    &verifier.profile.inject_anoncreds_ledger_read(),
                    &verifier.profile.inject_anoncreds(),
                    &verifier.agency_client,
                    &verifier_to_consumer1
                )
                .await
                .unwrap();
            assert_eq!(
                proof_verifier.get_verification_status(),
                PresentationVerificationStatus::Valid
            );

            let request_name2 = Some("request2");
            let mut proof_verifier = verifier_create_proof_and_send_request(
                &mut verifier,
                &verifier_to_consumer2,
                &schema_id,
                &cred_def_id,
                request_name2,
            )
                .await;

            #[cfg(feature = "migration")]
            consumer2.migrate().await;

            prover_select_credentials_and_send_proof(&mut consumer2, &consumer2_to_verifier, None, None).await;
            verifier_update_with_mediator(
                &mut proof_verifier,
                    &verifier.profile.inject_wallet(),
                    &verifier.profile.inject_anoncreds_ledger_read(),
                    &verifier.profile.inject_anoncreds(),
                    &verifier.agency_client,
                    &verifier_to_consumer2,
                )
                .await
                .unwrap();
            assert_eq!(
                proof_verifier.get_verification_status(),
                PresentationVerificationStatus::Valid
            );
        }).await;
    }

    #[tokio::test]
    #[ignore]
    async fn test_agency_pool_double_issuance_separate_issuer() {
        SetupPoolDirectory::run(|setup| async move {
            let mut issuer = create_faber_trustee(setup.genesis_file_path.clone()).await;
            let mut verifier = create_faber_trustee(setup.genesis_file_path.clone()).await;
            let mut consumer = create_alice(setup.genesis_file_path).await;

            let (consumer_to_verifier, verifier_to_consumer) =
                create_connected_connections(&mut consumer, &mut verifier).await;
            let (consumer_to_issuer, issuer_to_consumer) =
                create_connected_connections(&mut consumer, &mut issuer).await;

            let (schema_id, cred_def_id, _rev_reg_id, _cred_def, _rev_reg, _credential_handle) =
                issue_address_credential(&mut consumer, &mut issuer, &consumer_to_issuer, &issuer_to_consumer).await;

            #[cfg(feature = "migration")]
            issuer.migrate().await;

            let request_name1 = Some("request1");
            let mut proof_verifier = verifier_create_proof_and_send_request(
                &mut verifier,
                &verifier_to_consumer,
                &schema_id,
                &cred_def_id,
                request_name1,
            )
            .await;

            #[cfg(feature = "migration")]
            verifier.migrate().await;

            prover_select_credentials_and_send_proof(&mut consumer, &consumer_to_verifier, request_name1, None).await;
            verifier_update_with_mediator(
                &mut proof_verifier,
                &verifier.profile.inject_wallet(),
                &verifier.profile.inject_anoncreds_ledger_read(),
                &verifier.profile.inject_anoncreds(),
                &verifier.agency_client,
                &verifier_to_consumer,
            )
            .await
            .unwrap();
            assert_eq!(
                proof_verifier.get_verification_status(),
                PresentationVerificationStatus::Valid
            );

            let request_name2 = Some("request2");
            let mut proof_verifier = verifier_create_proof_and_send_request(
                &mut verifier,
                &verifier_to_consumer,
                &schema_id,
                &cred_def_id,
                request_name2,
            )
            .await;

            #[cfg(feature = "migration")]
            consumer.migrate().await;

            prover_select_credentials_and_send_proof(&mut consumer, &consumer_to_verifier, request_name2, None).await;
            verifier_update_with_mediator(
                &mut proof_verifier,
                &verifier.profile.inject_wallet(),
                &verifier.profile.inject_anoncreds_ledger_read(),
                &verifier.profile.inject_anoncreds(),
                &verifier.agency_client,
                &verifier_to_consumer,
            )
            .await
            .unwrap();
            assert_eq!(
                proof_verifier.get_verification_status(),
                PresentationVerificationStatus::Valid
            );
        })
        .await;
    }

    #[tokio::test]
    #[ignore]
    async fn test_agency_pool_double_issuance_issuer_is_verifier() {
        SetupPoolDirectory::run(|setup| async move {
            let mut institution = create_faber_trustee(setup.genesis_file_path.clone()).await;
            let mut consumer = create_alice(setup.genesis_file_path.clone()).await;

            let (consumer_to_institution, institution_to_consumer) =
                create_connected_connections(&mut consumer, &mut institution).await;

            let (schema_id, _schema_json, cred_def_id, _cred_def_json, cred_def, rev_reg, _rev_reg_id) =
                _create_address_schema_creddef_revreg(&institution.profile, &institution.institution_did).await;
            let (address1, address, city, state, zip) = attr_names();
            let credential_data = json!({address1.clone(): "5th Avenue", address.clone(): "Suite 1234", city.clone(): "NYC", state.clone(): "NYS", zip.clone(): "84712"}).to_string();
            let _credential_handle = _exchange_credential(
                &mut consumer,
                &mut institution,
                credential_data,
                &cred_def,
                &rev_reg,
                &consumer_to_institution,
                &institution_to_consumer,
                None,
            )
                .await;

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
            institution.migrate().await;

            prover_select_credentials_and_send_proof(&mut consumer, &consumer_to_institution, request_name1, None).await;
            verifier_update_with_mediator(
                &mut verifier,
                    &institution.profile.inject_wallet(),
                    &institution.profile.inject_anoncreds_ledger_read(),
                    &institution.profile.inject_anoncreds(),
                    &institution.agency_client,
                    &institution_to_consumer,
                )
                .await
                .unwrap();
            assert_eq!(
                verifier.get_verification_status(),
                PresentationVerificationStatus::Valid
            );

            let request_name2 = Some("request2");
            let mut verifier = verifier_create_proof_and_send_request(
                &mut institution,
                &institution_to_consumer,
                &schema_id,
                &cred_def_id,
                request_name2,
            )
                .await;

            #[cfg(feature = "migration")]
            consumer.migrate().await;

            prover_select_credentials_and_send_proof(&mut consumer, &consumer_to_institution, request_name2, None).await;
            verifier_update_with_mediator(
                &mut verifier,
                    &institution.profile.inject_wallet(),
                    &institution.profile.inject_anoncreds_ledger_read(),
                    &institution.profile.inject_anoncreds(),
                    &institution.agency_client,
                    &institution_to_consumer,
                )
                .await
                .unwrap();
            assert_eq!(
                verifier.get_verification_status(),
                PresentationVerificationStatus::Valid
            );
        }).await;
    }

    #[tokio::test]
    #[ignore]
    async fn test_agency_pool_real_proof() {
        SetupPoolDirectory::run(|setup| async move {
            let mut institution = create_faber_trustee(setup.genesis_file_path.clone()).await;
            let mut consumer = create_alice(setup.genesis_file_path.clone()).await;

            let (consumer_to_issuer, issuer_to_consumer) =
                create_connected_connections(&mut consumer, &mut institution).await;

            info!("test_real_proof >>>");
            let number_of_attributes = 10;

            info!("test_real_proof :: AS INSTITUTION SEND CREDENTIAL OFFER");
            let mut attrs_list: Value = serde_json::Value::Array(vec![]);
            for i in 1..number_of_attributes {
                attrs_list.as_array_mut().unwrap().push(json!(format!("key{}", i)));
            }
            let attrs_list = attrs_list.to_string();
            let (schema_id, _schema_json, cred_def_id, _cred_def_json, cred_def) =
                create_and_store_nonrevocable_credential_def(
                    &institution.profile.inject_anoncreds(),
                    &institution.profile.inject_anoncreds_ledger_read(),
                    &institution.profile.inject_anoncreds_ledger_write(),
                    &institution.institution_did,
                    &attrs_list,
                )
                .await;
            let mut credential_data = json!({});
            for i in 1..number_of_attributes {
                credential_data[format!("key{}", i)] = Value::String(format!("value{}", i));
            }

            #[cfg(feature = "migration")]
            institution.migrate().await;

            info!("test_real_proof :: sending credential offer");
            let credential_data = credential_data.to_string();
            info!("test_real_proof :: generated credential data: {}", credential_data);
            let mut issuer_credential = create_and_send_nonrevocable_cred_offer(
                &mut institution,
                &cred_def,
                &issuer_to_consumer,
                &credential_data,
                None,
            )
            .await;
            let issuance_thread_id = issuer_credential.get_thread_id().unwrap();

            info!("test_real_proof :: AS CONSUMER SEND CREDENTIAL REQUEST");
            let mut holder_credential = send_cred_req(&mut consumer, &consumer_to_issuer, None).await;

            #[cfg(feature = "migration")]
            consumer.migrate().await;

            info!("test_real_proof :: AS INSTITUTION SEND CREDENTIAL");
            send_credential(
                &mut consumer,
                &mut institution,
                &mut issuer_credential,
                &issuer_to_consumer,
                &consumer_to_issuer,
                &mut holder_credential,
                false,
            )
            .await;
            assert_eq!(issuance_thread_id, holder_credential.get_thread_id().unwrap());
            assert_eq!(issuance_thread_id, issuer_credential.get_thread_id().unwrap());

            info!("test_real_proof :: AS INSTITUTION SEND PROOF REQUEST");

            let institution_did = &institution.institution_did.clone();
            let restrictions =
                json!({ "issuer_did": institution_did, "schema_id": schema_id, "cred_def_id": cred_def_id, });
            let mut attrs: Value = serde_json::Value::Array(vec![]);
            for i in 1..number_of_attributes {
                attrs
                    .as_array_mut()
                    .unwrap()
                    .push(json!({ "name":format!("key{}", i), "restrictions": [restrictions]}));
            }
            let requested_attrs = attrs.to_string();
            info!(
                "test_real_proof :: Going to seng proof request with attributes {}",
                requested_attrs
            );
            let mut verifier = send_proof_request(
                &mut institution,
                &issuer_to_consumer,
                &requested_attrs,
                "[]",
                "{}",
                None,
            )
            .await;
            let presentation_thread_id = verifier.get_thread_id().unwrap();

            info!("test_real_proof :: Going to create proof");
            let mut prover = create_proof(&mut consumer, &consumer_to_issuer, None).await;
            info!("test_real_proof :: retrieving matching credentials");

            let retrieved_credentials = prover
                .retrieve_credentials(&consumer.profile.inject_anoncreds())
                .await
                .unwrap();
            let selected_credentials = retrieved_to_selected_credentials_simple(&retrieved_credentials, false);

            info!("test_real_proof :: generating and sending proof");
            generate_and_send_proof(&mut consumer, &mut prover, &consumer_to_issuer, selected_credentials).await;
            assert_eq!(ProverState::PresentationSent, prover.get_state());
            assert_eq!(presentation_thread_id, prover.get_thread_id().unwrap());
            assert_eq!(presentation_thread_id, verifier.get_thread_id().unwrap());

            info!("test_real_proof :: AS INSTITUTION VALIDATE PROOF");
            verifier_update_with_mediator(
                &mut verifier,
                &institution.profile.inject_wallet(),
                &institution.profile.inject_anoncreds_ledger_read(),
                &institution.profile.inject_anoncreds(),
                &institution.agency_client,
                &issuer_to_consumer,
            )
            .await
            .unwrap();
            assert_eq!(
                verifier.get_verification_status(),
                PresentationVerificationStatus::Valid
            );
            assert_eq!(presentation_thread_id, verifier.get_thread_id().unwrap());
        })
        .await;
    }

    #[tokio::test]
    #[ignore]
    async fn test_agency_pool_two_creds_one_rev_reg() {
        SetupPoolDirectory::run(|setup| async move {
            let mut issuer = create_faber_trustee(setup.genesis_file_path.clone()).await;
            let mut verifier = create_faber_trustee(setup.genesis_file_path.clone()).await;
            let mut consumer = create_alice(setup.genesis_file_path).await;

            let (consumer_to_verifier, verifier_to_consumer) =
                create_connected_connections(&mut consumer, &mut verifier).await;
            let (consumer_to_issuer, issuer_to_consumer) = create_connected_connections(&mut consumer, &mut issuer).await;

            let (schema_id, _schema_json, cred_def_id, _cred_def_json, cred_def, rev_reg, _rev_reg_id) =
                _create_address_schema_creddef_revreg(&issuer.profile, &issuer.institution_did).await;
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

            #[cfg(feature = "migration")]
            issuer.migrate().await;

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

            #[cfg(feature = "migration")]
            verifier.migrate().await;

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
                verifier_update_with_mediator(
                    &mut proof_verifier,
                    &verifier.profile.inject_wallet(),
                    &verifier.profile.inject_anoncreds_ledger_read(),
                    &verifier.profile.inject_anoncreds(),
                    &verifier.agency_client,
                    &verifier_to_consumer
                )
                .await
                .unwrap();
            assert_eq!(
                proof_verifier.get_verification_status(),
                PresentationVerificationStatus::Valid
            );

            let mut proof_verifier = verifier_create_proof_and_send_request(
                &mut verifier,
                &verifier_to_consumer,
                &schema_id,
                &cred_def_id,
                req2,
            )
                .await;


            #[cfg(feature = "migration")]
            consumer.migrate().await;

            prover_select_credentials_and_send_proof(&mut consumer, &consumer_to_verifier, req2, Some(&credential_data2))
                .await;
            verifier_update_with_mediator(
                &mut proof_verifier,
                    &verifier.profile.inject_wallet(),
                    &verifier.profile.inject_anoncreds_ledger_read(),
                    &verifier.profile.inject_anoncreds(),
                    &verifier.agency_client,
                    &verifier_to_consumer
                )
                .await
                .unwrap();
            assert_eq!(
                proof_verifier.get_verification_status(),
                PresentationVerificationStatus::Valid
            );
        }).await;
    }

    #[tokio::test]
    #[ignore]
    async fn test_agency_pool_credential_exchange_via_proposal() {
        SetupPoolDirectory::run(|setup| async move {
            let mut institution = create_faber_trustee(setup.genesis_file_path.clone()).await;
            let mut consumer = create_alice(setup.genesis_file_path).await;

            let (consumer_to_institution, institution_to_consumer) =
                create_connected_connections(&mut consumer, &mut institution).await;
            let (schema_id, _schema_json, cred_def_id, _cred_def_json, _cred_def, rev_reg, rev_reg_id) =
                _create_address_schema_creddef_revreg(&institution.profile, &institution.institution_did).await;
            let tails_dir = rev_reg.get_tails_dir();

            #[cfg(feature = "migration")]
            institution.migrate().await;

            _exchange_credential_with_proposal(
                &mut consumer,
                &mut institution,
                &consumer_to_institution,
                &institution_to_consumer,
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
            let mut institution = create_faber_trustee(setup.genesis_file_path.clone()).await;
            let mut consumer = create_alice(setup.genesis_file_path.clone()).await;

            let (consumer_to_institution, institution_to_consumer) =
                create_connected_connections(&mut consumer, &mut institution).await;
            let (schema_id, _schema_json, cred_def_id, _cred_def_json, _cred_def, rev_reg, rev_reg_id) =
                _create_address_schema_creddef_revreg(&institution.profile, &institution.institution_did).await;
            let tails_dir = rev_reg.get_tails_dir();

            let mut holder = send_cred_proposal(
                &mut consumer,
                &consumer_to_institution,
                &schema_id,
                &cred_def_id,
                "comment",
            )
            .await;

            #[cfg(feature = "migration")]
            institution.migrate().await;

            let mut issuer =
                accept_cred_proposal(&mut institution, &institution_to_consumer, rev_reg_id, Some(tails_dir)).await;
            decline_offer(&mut consumer, &consumer_to_institution, &mut holder).await;
            assert_eq!(IssuerState::OfferSent, issuer.get_state());
            tokio::time::sleep(Duration::from_millis(1000)).await;
            issuer_update_with_mediator(&mut issuer, &institution.agency_client, &institution_to_consumer)
                .await
                .unwrap();
            assert_eq!(IssuerState::Failed, issuer.get_state());
        })
        .await;
    }

    #[tokio::test]
    #[ignore]
    async fn test_agency_pool_credential_exchange_via_proposal_with_negotiation() {
        SetupPoolDirectory::run(|setup| async move {
            let mut institution = create_faber_trustee(setup.genesis_file_path.clone()).await;
            let mut consumer = create_alice(setup.genesis_file_path.clone()).await;

            let (consumer_to_institution, institution_to_consumer) =
                create_connected_connections(&mut consumer, &mut institution).await;
            let (schema_id, _schema_json, cred_def_id, _cred_def_json, _cred_def, rev_reg, rev_reg_id) =
                _create_address_schema_creddef_revreg(&institution.profile, &institution.institution_did).await;
            let tails_dir = rev_reg.get_tails_dir();

            #[cfg(feature = "migration")]
            institution.migrate().await;

            let mut holder = send_cred_proposal(
                &mut consumer,
                &consumer_to_institution,
                &schema_id,
                &cred_def_id,
                "comment",
            )
            .await;
            let mut issuer = accept_cred_proposal(
                &mut institution,
                &institution_to_consumer,
                rev_reg_id.clone(),
                Some(tails_dir.clone()),
            )
            .await;

            #[cfg(feature = "migration")]
            consumer.migrate().await;

            send_cred_proposal_1(
                &mut holder,
                &mut consumer,
                &consumer_to_institution,
                &schema_id,
                &cred_def_id,
                "comment",
            )
            .await;
            accept_cred_proposal_1(
                &mut issuer,
                &mut institution,
                &institution_to_consumer,
                rev_reg_id,
                Some(tails_dir),
            )
            .await;
            accept_offer(&mut consumer, &consumer_to_institution, &mut holder).await;
            tokio::time::sleep(Duration::from_millis(1000)).await;
            send_credential(
                &mut consumer,
                &mut institution,
                &mut issuer,
                &institution_to_consumer,
                &consumer_to_institution,
                &mut holder,
                true,
            )
            .await;
        })
        .await;
    }

    #[tokio::test]
    #[ignore]
    async fn test_agency_pool_presentation_via_proposal() {
        SetupPoolDirectory::run(|setup| async move {
            let mut institution = create_faber_trustee(setup.genesis_file_path.clone()).await;
            let mut consumer = create_alice(setup.genesis_file_path.clone()).await;

            let (consumer_to_institution, institution_to_consumer) =
                create_connected_connections(&mut consumer, &mut institution).await;
            let (schema_id, _schema_json, cred_def_id, _cred_def_json, _cred_def, rev_reg, rev_reg_id) =
                _create_address_schema_creddef_revreg(&institution.profile, &institution.institution_did).await;
            let tails_dir = rev_reg.get_tails_dir();

            #[cfg(feature = "migration")]
            institution.migrate().await;

            _exchange_credential_with_proposal(
                &mut consumer,
                &mut institution,
                &consumer_to_institution,
                &institution_to_consumer,
                &schema_id,
                &cred_def_id,
                rev_reg_id,
                Some(tails_dir),
                "comment",
            )
            .await;
            let mut prover = send_proof_proposal(&mut consumer, &consumer_to_institution, &cred_def_id).await;
            let mut verifier = Verifier::create("1").unwrap();
            accept_proof_proposal(&mut institution, &mut verifier, &institution_to_consumer).await;

            #[cfg(feature = "migration")]
            consumer.migrate().await;

            let selected_credentials =
                prover_select_credentials(&mut prover, &mut consumer, &consumer_to_institution, None).await;
            generate_and_send_proof(
                &mut consumer,
                &mut prover,
                &consumer_to_institution,
                selected_credentials,
            )
            .await;
            verify_proof(&mut institution, &mut verifier, &institution_to_consumer).await;
        })
        .await;
    }

    #[tokio::test]
    #[ignore]
    async fn test_agency_pool_presentation_via_proposal_with_rejection() {
        SetupPoolDirectory::run(|setup| async move {
            let mut institution = create_faber_trustee(setup.genesis_file_path.clone()).await;
            let mut consumer = create_alice(setup.genesis_file_path.clone()).await;

            let (consumer_to_institution, institution_to_consumer) =
                create_connected_connections(&mut consumer, &mut institution).await;
            let (schema_id, _schema_json, cred_def_id, _cred_def_json, _cred_def, rev_reg, rev_reg_id) =
                _create_address_schema_creddef_revreg(&institution.profile, &institution.institution_did).await;
            let tails_dir = rev_reg.get_tails_dir();

            #[cfg(feature = "migration")]
            institution.migrate().await;

            _exchange_credential_with_proposal(
                &mut consumer,
                &mut institution,
                &consumer_to_institution,
                &institution_to_consumer,
                &schema_id,
                &cred_def_id,
                rev_reg_id,
                Some(tails_dir),
                "comment",
            )
            .await;
            let mut prover = send_proof_proposal(&mut consumer, &consumer_to_institution, &cred_def_id).await;
            reject_proof_proposal(&mut institution, &institution_to_consumer).await;
            receive_proof_proposal_rejection(&mut consumer, &mut prover, &consumer_to_institution).await;
        })
        .await;
    }

    #[tokio::test]
    #[ignore]
    async fn test_agency_pool_presentation_via_proposal_with_negotiation() {
        SetupPoolDirectory::run(|setup| async move {
            let mut institution = create_faber_trustee(setup.genesis_file_path.clone()).await;
            let mut consumer = create_alice(setup.genesis_file_path.clone()).await;

            let (consumer_to_institution, institution_to_consumer) =
                create_connected_connections(&mut consumer, &mut institution).await;
            let (schema_id, _schema_json, cred_def_id, _cred_def_json, _cred_def, rev_reg, rev_reg_id) =
                _create_address_schema_creddef_revreg(&institution.profile, &institution.institution_did).await;
            let tails_dir = rev_reg.get_tails_dir();

            #[cfg(feature = "migration")]
            institution.migrate().await;

            _exchange_credential_with_proposal(
                &mut consumer,
                &mut institution,
                &consumer_to_institution,
                &institution_to_consumer,
                &schema_id,
                &cred_def_id,
                rev_reg_id,
                Some(tails_dir),
                "comment",
            )
            .await;
            let mut prover = send_proof_proposal(&mut consumer, &consumer_to_institution, &cred_def_id).await;
            let mut verifier = Verifier::create("1").unwrap();

            #[cfg(feature = "migration")]
            consumer.migrate().await;

            accept_proof_proposal(&mut institution, &mut verifier, &institution_to_consumer).await;
            send_proof_proposal_1(&mut consumer, &mut prover, &consumer_to_institution, &cred_def_id).await;
            accept_proof_proposal(&mut institution, &mut verifier, &institution_to_consumer).await;
            let selected_credentials =
                prover_select_credentials(&mut prover, &mut consumer, &consumer_to_institution, None).await;
            generate_and_send_proof(
                &mut consumer,
                &mut prover,
                &consumer_to_institution,
                selected_credentials,
            )
            .await;
            verify_proof(&mut institution, &mut verifier, &institution_to_consumer).await;
        })
        .await;
    }

    #[tokio::test]
    #[ignore]
    async fn test_agency_pool_aries_demo() {
        let _setup = SetupEmpty::init();
        SetupPoolDirectory::run(|setup| async move {
            let mut faber = create_faber_trustee(setup.genesis_file_path.clone()).await;
            let mut alice = create_alice(setup.genesis_file_path.clone()).await;

            // Publish Schema and Credential Definition
            faber.create_schema().await.unwrap();

            std::thread::sleep(std::time::Duration::from_secs(2));

            faber.create_nonrevocable_credential_definition().await;

            #[cfg(feature = "migration")]
            faber.migrate().await;

            // Connection
            let invite = faber.create_invite().await;
            alice.accept_invite(&invite).await;

            faber.update_state(3).await;
            alice.update_state(4).await;
            faber.update_state(4).await;

            #[cfg(feature = "migration")]
            alice.migrate().await;

            // Credential issuance
            faber.offer_non_revocable_credential().await;
            alice.accept_offer().await;
            faber.send_credential().await;
            alice.accept_credential().await;

            // Credential Presentation
            faber.request_presentation().await;
            alice.send_presentation().await;
            faber.verify_presentation().await;
            alice.ensure_presentation_verified().await;
        })
        .await;
    }

    #[tokio::test]
    #[ignore]
    async fn test_agency_pool_aries_demo_create_with_message_id_flow() {
        let _setup = SetupEmpty::init();
        SetupPoolDirectory::run(|setup| async move {
            let mut faber = create_faber_trustee(setup.genesis_file_path.clone()).await;
            let mut alice = create_alice(setup.genesis_file_path.clone()).await;

            // Publish Schema and Credential Definition
            faber.create_schema().await.unwrap();

            std::thread::sleep(std::time::Duration::from_secs(2));

            faber.create_nonrevocable_credential_definition().await;

            #[cfg(feature = "migration")]
            faber.migrate().await;

            // Connection
            let invite = faber.create_invite().await;
            alice.accept_invite(&invite).await;

            faber.update_state(3).await;
            alice.update_state(4).await;
            faber.update_state(4).await;

            /*
             Create with message id flow
            */

            // Credential issuance
            faber.offer_non_revocable_credential().await;

            // Alice creates Credential object with message id
            {
                let message = alice.download_message(PayloadKinds::CredOffer).await.unwrap();
                let cred_offer = alice.get_credential_offer_by_msg_id(&message.uid).await.unwrap();
                alice.credential = Holder::create_from_offer("test", cred_offer).unwrap();

                let pw_did = alice.connection.pairwise_info().pw_did.to_string();
                alice
                    .credential
                    .send_request(
                        &alice.profile.inject_anoncreds_ledger_read(),
                        &alice.profile.inject_anoncreds(),
                        pw_did,
                        alice
                            .connection
                            .send_message_closure(alice.profile.inject_wallet())
                            .await
                            .unwrap(),
                    )
                    .await
                    .unwrap();
                assert_eq!(HolderState::RequestSent, alice.credential.get_state());
            }

            #[cfg(feature = "migration")]
            alice.migrate().await;

            faber.send_credential().await;
            alice.accept_credential().await;

            // Credential Presentation
            faber.request_presentation().await;

            // Alice creates Presentation object with message id
            {
                let message = alice.download_message(PayloadKinds::ProofRequest).await.unwrap();
                let presentation_request = alice.get_proof_request_by_msg_id(&message.uid).await.unwrap();
                alice.prover = Prover::create_from_request("test", presentation_request).unwrap();

                let credentials = alice.get_credentials_for_presentation().await;

                alice
                    .prover
                    .generate_presentation(
                        &alice.profile.inject_anoncreds_ledger_read(),
                        &alice.profile.inject_anoncreds(),
                        credentials,
                        HashMap::new(),
                    )
                    .await
                    .unwrap();
                assert_eq!(ProverState::PresentationPrepared, alice.prover.get_state());

                let send_closure = alice
                    .connection
                    .send_message_closure(alice.profile.inject_wallet())
                    .await
                    .unwrap();
                let message = alice.prover.mark_presentation_sent().unwrap();
                send_closure(message).await.unwrap();
                assert_eq!(ProverState::PresentationSent, alice.prover.get_state());
            }

            faber.verify_presentation().await;
        })
        .await;
    }

    #[tokio::test]
    #[ignore]
    async fn test_agency_pool_aries_demo_download_message_flow() {
        SetupEmpty::init();
        SetupPoolDirectory::run(|setup| async move {
            let mut faber = create_faber_trustee(setup.genesis_file_path.clone()).await;
            let mut alice = create_alice(setup.genesis_file_path.clone()).await;

            // Publish Schema and Credential Definition
            faber.create_schema().await.unwrap();

            std::thread::sleep(std::time::Duration::from_secs(2));

            faber.create_nonrevocable_credential_definition().await;

            #[cfg(feature = "migration")]
            faber.migrate().await;

            // Connection
            let invite = faber.create_invite().await;
            alice.accept_invite(&invite).await;

            faber.update_state(3).await;
            alice.update_state(4).await;
            faber.update_state(4).await;

            /*
             Create with message flow
            */

            // Credential issuance
            faber.offer_non_revocable_credential().await;

            // Alice creates Credential object with Offer
            {
                let message = alice.download_message(PayloadKinds::CredOffer).await.unwrap();

                let cred_offer: OfferCredential = serde_json::from_str(&message.decrypted_msg).unwrap();
                alice.credential = Holder::create_from_offer("test", cred_offer).unwrap();

                alice
                    .connection
                    .update_message_status(&message.uid, &alice.agency_client)
                    .await
                    .unwrap();

                let pw_did = alice.connection.pairwise_info().pw_did.to_string();
                alice
                    .credential
                    .send_request(
                        &alice.profile.inject_anoncreds_ledger_read(),
                        &alice.profile.inject_anoncreds(),
                        pw_did,
                        alice
                            .connection
                            .send_message_closure(alice.profile.inject_wallet())
                            .await
                            .unwrap(),
                    )
                    .await
                    .unwrap();
                assert_eq!(HolderState::RequestSent, alice.credential.get_state());
            }

            faber.send_credential().await;
            alice.accept_credential().await;

            #[cfg(feature = "migration")]
            alice.migrate().await;

            // Credential Presentation
            faber.request_presentation().await;

            // Alice creates Presentation object with Proof Request
            {
                let agency_msg = alice.download_message(PayloadKinds::ProofRequest).await.unwrap();

                let presentation_request: RequestPresentation =
                    serde_json::from_str(&agency_msg.decrypted_msg).unwrap();
                alice.prover = Prover::create_from_request("test", presentation_request).unwrap();

                alice
                    .connection
                    .update_message_status(&agency_msg.uid, &alice.agency_client)
                    .await
                    .unwrap();

                let credentials = alice.get_credentials_for_presentation().await;

                alice
                    .prover
                    .generate_presentation(
                        &alice.profile.inject_anoncreds_ledger_read(),
                        &alice.profile.inject_anoncreds(),
                        credentials,
                        HashMap::new(),
                    )
                    .await
                    .unwrap();
                assert_eq!(ProverState::PresentationPrepared, alice.prover.get_state());

                let send_closure = alice
                    .connection
                    .send_message_closure(alice.profile.inject_wallet())
                    .await
                    .unwrap();
                let message = alice.prover.mark_presentation_sent().unwrap();
                send_closure(message).await.unwrap();
                assert_eq!(ProverState::PresentationSent, alice.prover.get_state());
            }

            faber.verify_presentation().await;
        })
        .await;
    }
}
