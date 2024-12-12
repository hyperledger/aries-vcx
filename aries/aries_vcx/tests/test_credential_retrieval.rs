#![allow(clippy::diverging_sub_expression)]

use std::{collections::HashMap, error::Error};

use anoncreds_types::data_types::messages::{
    cred_selection::RetrievedCredentials,
    nonce::Nonce,
    pres_request::{AttributeInfo, PresentationRequest, PresentationRequestPayload},
};
use aries_vcx::handlers::{proof_presentation::prover::Prover, util::AttachmentId};
use base64::{engine::general_purpose, Engine};
use messages::{
    decorators::attachment::{Attachment, AttachmentData, AttachmentType},
    misc::MimeType,
    msg_fields::protocols::present_proof::v1::request::{
        RequestPresentationV1, RequestPresentationV1Content,
    },
};
use serde_json::json;
use test_utils::{constants::DEFAULT_SCHEMA_ATTRS, devsetup::build_setup_profile};

use crate::utils::{
    create_and_write_credential, create_and_write_test_cred_def, create_and_write_test_schema,
};

pub mod utils;

#[tokio::test]
#[ignore]
// TODO: This should be a unit test
async fn test_agency_pool_retrieve_credentials_empty() -> Result<(), Box<dyn Error>> {
    let setup = build_setup_profile().await;
    let pres_req_data = PresentationRequestPayload::builder()
        .name("proof_req_1".into())
        .nonce(Nonce::new().unwrap())
        .build();

    let attach_type =
        AttachmentType::Base64(general_purpose::STANDARD.encode(json!(pres_req_data).to_string()));
    let attach_data = AttachmentData::builder().content(attach_type).build();
    let attach = Attachment::builder()
        .data(attach_data)
        .id(AttachmentId::PresentationRequest.as_ref().to_owned())
        .mime_type(MimeType::Json)
        .build();

    let content = RequestPresentationV1Content::builder()
        .request_presentations_attach(vec![attach])
        .build();

    // test retrieving credentials for empty proof request returns "{}"
    let id = "test_id".to_owned();
    let proof_req = RequestPresentationV1::builder()
        .id(id)
        .content(content)
        .build();
    let proof: Prover = Prover::create_from_request("1", proof_req)?;

    let retrieved_creds = proof
        .retrieve_credentials(&setup.wallet, &setup.anoncreds)
        .await?;
    assert_eq!(serde_json::to_string(&retrieved_creds)?, "{}".to_string());
    assert!(retrieved_creds.credentials_by_referent.is_empty());

    // populate proof request with a single attribute referent request
    let pres_req_data = PresentationRequestPayload::builder()
        .name("proof_req_1".into())
        .requested_attributes(
            vec![(
                "address1_1".into(),
                AttributeInfo {
                    name: Some("address1".into()),
                    ..Default::default()
                },
            )]
            .into_iter()
            .collect(),
        )
        .nonce(Nonce::new().unwrap())
        .build();

    let attach_type =
        AttachmentType::Base64(general_purpose::STANDARD.encode(json!(pres_req_data).to_string()));
    let attach_data = AttachmentData::builder().content(attach_type).build();
    let attach = Attachment::builder()
        .data(attach_data)
        .id(AttachmentId::PresentationRequest.as_ref().to_owned())
        .mime_type(MimeType::Json)
        .build();

    let content = RequestPresentationV1Content::builder()
        .request_presentations_attach(vec![attach])
        .build();

    // test retrieving credentials for the proof request returns the referent with no cred
    // matches
    let id = "test_id".to_owned();
    let proof_req = RequestPresentationV1::builder()
        .id(id)
        .content(content)
        .build();
    let proof: Prover = Prover::create_from_request("2", proof_req)?;

    let retrieved_creds = proof
        .retrieve_credentials(&setup.wallet, &setup.anoncreds)
        .await?;
    assert_eq!(
        serde_json::to_string(&retrieved_creds)?,
        json!({"attrs":{"address1_1":[]}}).to_string()
    );
    assert_eq!(
        retrieved_creds,
        RetrievedCredentials {
            credentials_by_referent: HashMap::from([("address1_1".to_string(), vec![])])
        }
    );
    Ok(())
}

#[tokio::test]
#[ignore]
// TODO: This should be a unit test
async fn test_agency_pool_case_for_proof_req_doesnt_matter_for_retrieve_creds(
) -> Result<(), Box<dyn Error>> {
    let setup = build_setup_profile().await;
    let schema = create_and_write_test_schema(
        &setup.wallet,
        &setup.anoncreds,
        &setup.ledger_write,
        &setup.institution_did,
        DEFAULT_SCHEMA_ATTRS,
    )
    .await;
    let cred_def = create_and_write_test_cred_def(
        &setup.wallet,
        &setup.anoncreds,
        &setup.ledger_read,
        &setup.ledger_write,
        &setup.institution_did,
        &schema.schema_id,
        true,
    )
    .await;
    create_and_write_credential(
        &setup.wallet,
        &setup.wallet,
        &setup.anoncreds,
        &setup.anoncreds,
        &setup.institution_did,
        &schema,
        &cred_def,
        None,
    )
    .await;

    let mut req = json!({
       "nonce":"123432421212",
       "name":"proof_req_1",
       "version":"1.0",
       "requested_attributes": json!({
           "zip_1": json!({
               "name":"zip",
               "restrictions": [json!({ "issuer_did": setup.institution_did })]
           })
       }),
       "requested_predicates": json!({}),
    });

    let pres_req_data: PresentationRequest = serde_json::from_str(&req.to_string())?;
    let id = "test_id".to_owned();

    let attach_type =
        AttachmentType::Base64(general_purpose::STANDARD.encode(json!(pres_req_data).to_string()));
    let attach_data = AttachmentData::builder().content(attach_type).build();
    let attach = Attachment::builder()
        .data(attach_data)
        .id(AttachmentId::PresentationRequest.as_ref().to_owned())
        .mime_type(MimeType::Json)
        .build();

    let content = RequestPresentationV1Content::builder()
        .request_presentations_attach(vec![attach])
        .build();

    let proof_req = RequestPresentationV1::builder()
        .id(id)
        .content(content)
        .build();
    let proof: Prover = Prover::create_from_request("1", proof_req)?;

    // All lower case
    let retrieved_creds = proof
        .retrieve_credentials(&setup.wallet, &setup.anoncreds)
        .await?;
    assert_eq!(
        retrieved_creds.credentials_by_referent["zip_1"][0]
            .cred_info
            .attributes["zip"],
        "84000"
    );

    // First letter upper
    req["requested_attributes"]["zip_1"]["name"] = json!("Zip");
    let pres_req_data: PresentationRequest = serde_json::from_str(&req.to_string())?;
    let id = "test_id".to_owned();

    let attach_type =
        AttachmentType::Base64(general_purpose::STANDARD.encode(json!(pres_req_data).to_string()));
    let attach_data = AttachmentData::builder().content(attach_type).build();
    let attach = Attachment::builder()
        .data(attach_data)
        .id(AttachmentId::PresentationRequest.as_ref().to_owned())
        .mime_type(MimeType::Json)
        .build();

    let content = RequestPresentationV1Content::builder()
        .request_presentations_attach(vec![attach])
        .build();

    let proof_req = RequestPresentationV1::builder()
        .id(id)
        .content(content)
        .build();
    let proof: Prover = Prover::create_from_request("2", proof_req)?;
    let retrieved_creds2 = proof
        .retrieve_credentials(&setup.wallet, &setup.anoncreds)
        .await?;
    assert_eq!(
        retrieved_creds2.credentials_by_referent["zip_1"][0]
            .cred_info
            .attributes["zip"],
        "84000"
    );

    // Entire word upper
    req["requested_attributes"]["zip_1"]["name"] = json!("ZIP");
    let pres_req_data: PresentationRequest = serde_json::from_str(&req.to_string())?;
    let id = "test_id".to_owned();

    let attach_type =
        AttachmentType::Base64(general_purpose::STANDARD.encode(json!(pres_req_data).to_string()));
    let attach_data = AttachmentData::builder().content(attach_type).build();
    let attach = Attachment::builder()
        .data(attach_data)
        .id(AttachmentId::PresentationRequest.as_ref().to_owned())
        .mime_type(MimeType::Json)
        .build();

    let content = RequestPresentationV1Content::builder()
        .request_presentations_attach(vec![attach])
        .build();

    let proof_req = RequestPresentationV1::builder()
        .id(id)
        .content(content)
        .build();
    let proof: Prover = Prover::create_from_request("1", proof_req)?;
    let retrieved_creds3 = proof
        .retrieve_credentials(&setup.wallet, &setup.anoncreds)
        .await?;
    assert_eq!(
        retrieved_creds3.credentials_by_referent["zip_1"][0]
            .cred_info
            .attributes["zip"],
        "84000"
    );
    Ok(())
}
