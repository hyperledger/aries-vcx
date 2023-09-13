#[macro_use]
extern crate log;
#[macro_use]
extern crate serde_json;

pub mod utils;

use std::collections::HashMap;

use aries_vcx::common::proofs::proof_request::PresentationRequestData;
use aries_vcx::common::test_utils::{create_and_store_credential, create_and_store_nonrevocable_credential};
use aries_vcx::handlers::proof_presentation::prover::Prover;
use aries_vcx::handlers::proof_presentation::types::RetrievedCredentials;
use aries_vcx::handlers::proof_presentation::verifier::Verifier;
use aries_vcx::handlers::util::AttachmentId;
use aries_vcx::protocols::issuance::issuer::state_machine::IssuerState;
use aries_vcx::protocols::proof_presentation::prover::state_machine::ProverState;
use aries_vcx::protocols::proof_presentation::verifier::state_machine::VerifierState;
use aries_vcx::protocols::proof_presentation::verifier::verification_status::PresentationVerificationStatus;
use aries_vcx::utils::constants::DEFAULT_SCHEMA_ATTRS;
use aries_vcx::utils::devsetup::SetupProfile;
use aries_vcx::utils::devsetup::*;

use messages::decorators::attachment::{Attachment, AttachmentData, AttachmentType};
use messages::misc::MimeType;
use messages::msg_fields::protocols::present_proof::request::{RequestPresentation, RequestPresentationContent};
use messages::msg_fields::protocols::present_proof::PresentProof;
use messages::AriesMessage;

#[cfg(feature = "migration")]
use crate::utils::migration::Migratable;
use crate::utils::scenarios::{
    _create_address_schema_creddef_revreg, _exchange_credential, _exchange_credential_with_proposal,
    accept_credential_proposal, accept_offer, accept_proof_proposal, attr_names, create_credential_proposal,
    create_holder_from_proposal, create_issuer_from_proposal, create_proof_proposal, decline_offer, exchange_proof,
    generate_and_send_proof, prover_select_credentials, receive_proof_proposal_rejection, reject_proof_proposal,
    send_credential, verify_proof,
};
use crate::utils::test_agent::{create_test_agent, create_test_agent_trustee};

/*
 * RETRIEVING CREDENTIALS
 */

#[tokio::test]
#[ignore]
// TODO: This should be a unit test
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

        let attach_type = AttachmentType::Base64(base64::encode(&json!(pres_req_data).to_string()));
        let attach_data = AttachmentData::builder().content(attach_type).build();
        let attach = Attachment::builder()
            .data(attach_data)
            .id(AttachmentId::PresentationRequest.as_ref().to_owned())
            .mime_type(MimeType::Json)
            .build();

        let content = RequestPresentationContent::builder()
            .request_presentations_attach(vec![attach])
            .build();

        // test retrieving credentials for empty proof request returns "{}"
        let id = "test_id".to_owned();
        let proof_req = RequestPresentation::builder().id(id).content(content).build();
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

        let attach_type = AttachmentType::Base64(base64::encode(&json!(pres_req_data).to_string()));
        let attach_data = AttachmentData::builder().content(attach_type).build();
        let attach = Attachment::builder()
            .data(attach_data)
            .id(AttachmentId::PresentationRequest.as_ref().to_owned())
            .mime_type(MimeType::Json)
            .build();

        let content = RequestPresentationContent::builder()
            .request_presentations_attach(vec![attach])
            .build();

        // test retrieving credentials for the proof request returns the referent with no cred matches
        let id = "test_id".to_owned();
        let proof_req = RequestPresentation::builder().id(id).content(content).build();
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
// TODO: This should be a unit test
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

        let attach_type = AttachmentType::Base64(base64::encode(&json!(pres_req_data).to_string()));
        let attach_data = AttachmentData::builder().content(attach_type).build();
        let attach = Attachment::builder()
            .data(attach_data)
            .id(AttachmentId::PresentationRequest.as_ref().to_owned())
            .mime_type(MimeType::Json)
            .build();

        let content = RequestPresentationContent::builder()
            .request_presentations_attach(vec![attach])
            .build();

        let proof_req = RequestPresentation::builder().id(id).content(content).build();
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

        let attach_type = AttachmentType::Base64(base64::encode(&json!(pres_req_data).to_string()));
        let attach_data = AttachmentData::builder().content(attach_type).build();
        let attach = Attachment::builder()
            .data(attach_data)
            .id(AttachmentId::PresentationRequest.as_ref().to_owned())
            .mime_type(MimeType::Json)
            .build();

        let content = RequestPresentationContent::builder()
            .request_presentations_attach(vec![attach])
            .build();

        #[cfg(feature = "migration")]
        setup.migrate().await;

        let proof_req = RequestPresentation::builder().id(id).content(content).build();
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

        let attach_type = AttachmentType::Base64(base64::encode(&json!(pres_req_data).to_string()));
        let attach_data = AttachmentData::builder().content(attach_type).build();
        let attach = Attachment::builder()
            .data(attach_data)
            .id(AttachmentId::PresentationRequest.as_ref().to_owned())
            .mime_type(MimeType::Json)
            .build();

        let content = RequestPresentationContent::builder()
            .request_presentations_attach(vec![attach])
            .build();

        let proof_req = RequestPresentation::builder().id(id).content(content).build();
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

// todo: credx implementation does not support checking credential value in respect to predicate
#[cfg(not(feature = "modular_libs"))]
#[tokio::test]
#[ignore]
async fn test_agency_pool_it_should_fail_to_select_credentials_for_predicate() {
    use crate::utils::scenarios::{
        create_proof_request_data, create_prover_from_request, create_verifier_from_request_data,
        issue_address_credential,
    };

    SetupPoolDirectory::run(|setup| async move {
        let mut institution = create_test_agent_trustee(setup.genesis_file_path.clone()).await;
        let mut consumer = create_test_agent(setup.genesis_file_path).await;

        issue_address_credential(&mut consumer, &mut institution).await;

        #[cfg(feature = "migration")]
        institution.migrate().await;

        let requested_preds_string = serde_json::to_string(&json!([{
            "name": "zip",
            "p_type": ">=",
            "p_value": 85000
        }]))
        .unwrap();

        let presentation_request_data =
            create_proof_request_data(&mut institution, "[]", &requested_preds_string, "{}", None).await;
        let mut verifier = create_verifier_from_request_data(presentation_request_data).await;

        #[cfg(feature = "migration")]
        consumer.migrate().await;

        let presentation_request = verifier.get_presentation_request_msg().unwrap();
        let mut prover = create_prover_from_request(presentation_request.clone()).await;
        let selected_credentials =
            prover_select_credentials(&mut prover, &mut consumer, presentation_request.into(), None).await;

        assert!(selected_credentials.credential_for_referent.is_empty());
    })
    .await;
}

/*
 * PRESENTATION GENERATION
 */

#[tokio::test]
#[ignore]
async fn test_agency_pool_generate_proof_with_predicates() {
    SetupProfile::run(|mut setup| async move {
        let (schema_id, _, cred_def_id, _, _, _, _, _, _, _, tails_dir, _) = create_and_store_credential(
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
                    "restrictions": [{"issuer_did": "abcdef0000000000000000"}, {"issuer_did": setup.institution_did}],
                    "non_revoked":  {"from": 123, "to": to}
                },
                "state_2": {
                    "name": "state",
                    "restrictions": {
                        "issuer_did": setup.institution_did,
                        "schema_id": schema_id,
                        "cred_def_id": cred_def_id,
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

/*
 * DOUBLE ISSUANCE
 */

#[tokio::test]
#[ignore]
async fn test_agency_pool_double_issuance_issuer_is_verifier() {
    SetupPoolDirectory::run(|setup| async move {
        let mut institution = create_test_agent_trustee(setup.genesis_file_path.clone()).await;
        let mut consumer = create_test_agent(setup.genesis_file_path.clone()).await;

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

/*
 * ISSUANCE SCENARIOS
 */

#[tokio::test]
#[ignore]
async fn test_agency_pool_credential_exchange_via_proposal() {
    SetupPoolDirectory::run(|setup| async move {
        let mut institution = create_test_agent_trustee(setup.genesis_file_path.clone()).await;
        let mut consumer = create_test_agent(setup.genesis_file_path).await;

        let (schema_id, _schema_json, cred_def_id, _cred_def_json, _cred_def, rev_reg, rev_reg_id) =
            _create_address_schema_creddef_revreg(&institution.profile, &institution.institution_did).await;
        let tails_dir = rev_reg.get_tails_dir();

        #[cfg(feature = "migration")]
        institution.migrate().await;

        _exchange_credential_with_proposal(
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
            _create_address_schema_creddef_revreg(&institution.profile, &institution.institution_did).await;
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
            _create_address_schema_creddef_revreg(&institution.profile, &institution.institution_did).await;
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

/*
 * PRESENTATION SCENARIOS
 */

#[tokio::test]
#[ignore]
async fn test_agency_pool_presentation_via_proposal() {
    SetupPoolDirectory::run(|setup| async move {
        let mut institution = create_test_agent_trustee(setup.genesis_file_path.clone()).await;
        let mut consumer = create_test_agent(setup.genesis_file_path.clone()).await;

        let (schema_id, _schema_json, cred_def_id, _cred_def_json, _cred_def, rev_reg, rev_reg_id) =
            _create_address_schema_creddef_revreg(&institution.profile, &institution.institution_did).await;
        let tails_dir = rev_reg.get_tails_dir();

        #[cfg(feature = "migration")]
        institution.migrate().await;

        _exchange_credential_with_proposal(
            &mut consumer,
            &mut institution,
            &schema_id,
            &cred_def_id,
            rev_reg_id,
            Some(tails_dir),
            "comment",
        )
        .await;
        let mut prover = Prover::create("1").unwrap();
        let mut verifier = Verifier::create("1").unwrap();
        let presentation_proposal = create_proof_proposal(&mut prover, &cred_def_id).await;
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

        let (schema_id, _schema_json, cred_def_id, _cred_def_json, _cred_def, rev_reg, rev_reg_id) =
            _create_address_schema_creddef_revreg(&institution.profile, &institution.institution_did).await;
        let tails_dir = rev_reg.get_tails_dir();

        #[cfg(feature = "migration")]
        institution.migrate().await;

        _exchange_credential_with_proposal(
            &mut consumer,
            &mut institution,
            &schema_id,
            &cred_def_id,
            rev_reg_id,
            Some(tails_dir),
            "comment",
        )
        .await;
        let mut prover = Prover::create("1").unwrap();
        let presentation_proposal = create_proof_proposal(&mut prover, &cred_def_id).await;
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

        let (schema_id, _schema_json, cred_def_id, _cred_def_json, _cred_def, rev_reg, rev_reg_id) =
            _create_address_schema_creddef_revreg(&institution.profile, &institution.institution_did).await;
        let tails_dir = rev_reg.get_tails_dir();

        #[cfg(feature = "migration")]
        institution.migrate().await;

        _exchange_credential_with_proposal(
            &mut consumer,
            &mut institution,
            &schema_id,
            &cred_def_id,
            rev_reg_id,
            Some(tails_dir),
            "comment",
        )
        .await;
        let mut prover = Prover::create("1").unwrap();
        let mut verifier = Verifier::create("1").unwrap();

        #[cfg(feature = "migration")]
        consumer.migrate().await;

        let presentation_proposal = create_proof_proposal(&mut prover, &cred_def_id).await;
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
