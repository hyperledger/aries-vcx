#[macro_use]
extern crate log;
#[macro_use]
extern crate serde_json;

pub mod utils;

use std::collections::HashMap;

use aries_vcx::{
    common::{
        proofs::proof_request::PresentationRequestData,
        test_utils::{
            create_and_write_credential, create_and_write_test_cred_def,
            create_and_write_test_schema,
        },
    },
    handlers::{
        proof_presentation::{prover::Prover, types::RetrievedCredentials},
        util::AttachmentId,
    },
    utils::{constants::DEFAULT_SCHEMA_ATTRS, devsetup::SetupProfile},
};
use messages::{
    decorators::attachment::{Attachment, AttachmentData, AttachmentType},
    misc::MimeType,
    msg_fields::protocols::present_proof::request::{
        RequestPresentation, RequestPresentationContent,
    },
};

#[cfg(feature = "migration")]
use crate::utils::migration::Migratable;

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

        let pres_req_data: PresentationRequestData =
            serde_json::from_str(&req.to_string()).unwrap();

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
        let proof_req = RequestPresentation::builder()
            .id(id)
            .content(content)
            .build();
        let proof: Prover = Prover::create_from_request("1", proof_req).unwrap();

        #[cfg(feature = "migration")]
        setup.migrate().await;

        let retrieved_creds = proof
            .retrieve_credentials(&setup.profile.inject_anoncreds())
            .await
            .unwrap();
        assert_eq!(
            serde_json::to_string(&retrieved_creds).unwrap(),
            "{}".to_string()
        );
        assert!(retrieved_creds.credentials_by_referent.is_empty());

        // populate proof request with a single attribute referent request
        req["requested_attributes"]["address1_1"] = json!({"name": "address1"});
        let pres_req_data: PresentationRequestData =
            serde_json::from_str(&req.to_string()).unwrap();

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

        // test retrieving credentials for the proof request returns the referent with no cred
        // matches
        let id = "test_id".to_owned();
        let proof_req = RequestPresentation::builder()
            .id(id)
            .content(content)
            .build();
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
        let schema = create_and_write_test_schema(
            &setup.profile.inject_anoncreds(),
            &setup.profile.inject_anoncreds_ledger_write(),
            &setup.institution_did,
            DEFAULT_SCHEMA_ATTRS,
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
        create_and_write_credential(
            &setup.profile.inject_anoncreds(),
            &setup.profile.inject_anoncreds(),
            &setup.institution_did,
            &cred_def,
            None,
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

        let pres_req_data: PresentationRequestData =
            serde_json::from_str(&req.to_string()).unwrap();
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

        let proof_req = RequestPresentation::builder()
            .id(id)
            .content(content)
            .build();
        let proof: Prover = Prover::create_from_request("1", proof_req).unwrap();

        // All lower case
        let retrieved_creds = proof
            .retrieve_credentials(&setup.profile.inject_anoncreds())
            .await
            .unwrap();
        assert_eq!(
            retrieved_creds.credentials_by_referent["zip_1"][0]
                .cred_info
                .attributes["zip"],
            "84000"
        );

        // First letter upper
        req["requested_attributes"]["zip_1"]["name"] = json!("Zip");
        let pres_req_data: PresentationRequestData =
            serde_json::from_str(&req.to_string()).unwrap();
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

        let proof_req = RequestPresentation::builder()
            .id(id)
            .content(content)
            .build();
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
        let pres_req_data: PresentationRequestData =
            serde_json::from_str(&req.to_string()).unwrap();
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

        let proof_req = RequestPresentation::builder()
            .id(id)
            .content(content)
            .build();
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
    use aries_vcx::utils::devsetup::SetupPoolDirectory;
    use utils::{
        scenarios::prover_select_credentials,
        test_agent::{create_test_agent, create_test_agent_trustee},
    };

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
            create_proof_request_data(&mut institution, "[]", &requested_preds_string, "{}", None)
                .await;
        let mut verifier = create_verifier_from_request_data(presentation_request_data).await;

        #[cfg(feature = "migration")]
        consumer.migrate().await;

        let presentation_request = verifier.get_presentation_request_msg().unwrap();
        let mut prover = create_prover_from_request(presentation_request.clone()).await;
        let selected_credentials =
            prover_select_credentials(&mut prover, &mut consumer, presentation_request, None).await;

        assert!(selected_credentials.credential_for_referent.is_empty());
    })
    .await;
}
