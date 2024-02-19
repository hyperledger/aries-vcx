use std::collections::HashMap;

use anoncreds_types::{
    data_types::{
        identifiers::{cred_def_id::CredentialDefinitionId, schema_id::SchemaId},
        messages::pres_request::{AttributeInfo, NonRevokedInterval},
    },
    utils::query::Query,
};
use did_parser::Did;
use messages::{
    misc::MimeType,
    msg_fields::protocols::{
        cred_issuance::{
            common::CredentialAttr,
            v1::{
                propose_credential::{ProposeCredentialV1, ProposeCredentialV1Content},
                CredentialPreviewV1,
            },
        },
        present_proof::v1::propose::PresentationAttr,
    },
};
use serde_json::{json, Value};

pub fn attr_names_address() -> (String, String, String, String, String) {
    let address1 = "Address1".to_string();
    let address2 = "address2".to_string();
    let city = "CITY".to_string();
    let state = "State".to_string();
    let zip = "zip".to_string();
    (address1, address2, city, state, zip)
}

pub fn attr_names_address_list() -> Vec<String> {
    let (address1, address2, city, state, zip) = attr_names_address();
    vec![address1, address2, city, state, zip]
}

pub fn requested_attrs_address(
    did: &Did,
    schema_id: &SchemaId,
    cred_def_id: &CredentialDefinitionId,
    from: Option<u64>,
    to: Option<u64>,
) -> HashMap<String, AttributeInfo> {
    let restrictions = Query::And(vec![
        Query::Eq("issuer_did".to_string(), did.to_string()),
        Query::Eq("schema_id".to_string(), schema_id.to_string()),
        Query::Eq("cred_def_id".to_string(), cred_def_id.to_string()),
    ]);
    attr_names_address_list()
        .into_iter()
        .map(|name| {
            (
                format!("{}_1", name),
                AttributeInfo {
                    name: Some(name),
                    restrictions: Some(restrictions.to_owned()),
                    non_revoked: Some(NonRevokedInterval::new(from, to)),
                    ..Default::default()
                },
            )
        })
        .collect()
}

pub(super) fn requested_attr_objects(
    cred_def_id: &CredentialDefinitionId,
) -> Vec<PresentationAttr> {
    credential_data_address_1()
        .as_object()
        .unwrap()
        .iter()
        .map(|(key, value)| {
            PresentationAttr::builder()
                .name(key.to_string())
                .cred_def_id(cred_def_id.to_string())
                .value(value.to_string())
                .build()
        })
        .collect()
}

pub fn create_credential_proposal(
    schema_id: &SchemaId,
    cred_def_id: &CredentialDefinitionId,
    comment: &str,
) -> ProposeCredentialV1 {
    let attrs = credential_data_address_1()
        .as_object()
        .unwrap()
        .iter()
        .map(|(key, value)| {
            CredentialAttr::builder()
                .name(key.to_string())
                .value(value.to_string())
                .mime_type(MimeType::Plain)
                .build()
        })
        .collect();
    let content = ProposeCredentialV1Content::builder()
        .credential_proposal(CredentialPreviewV1::new(attrs))
        .schema_id(schema_id.to_string())
        .cred_def_id(cred_def_id.to_string())
        .comment(comment.to_owned())
        .build();
    ProposeCredentialV1::builder()
        .id("test".to_owned())
        .content(content)
        .build()
}

pub fn credential_data_address_1() -> Value {
    let (address1, address2, city, state, zip) = attr_names_address();
    json!({address1: "123 Main St", address2: "Suite 3", city: "Draper", state: "UT", zip: "84000"})
}

pub fn credential_data_address_2() -> Value {
    let (address1, address2, city, state, zip) = attr_names_address();
    json!({address1: "321 Test St", address2: "Suite Foo", city: "Kickapoo", state: "LU", zip: "87210"})
}

pub fn credential_data_address_3() -> Value {
    let (address1, address2, city, state, zip) = attr_names_address();
    json!({address1: "007 Mock St", address2: "Yes", city: "None", state: "KO", zip: "11111"})
}
