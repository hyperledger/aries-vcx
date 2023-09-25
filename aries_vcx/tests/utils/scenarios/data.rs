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
        present_proof::propose::PresentationAttr,
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
    did: &str,
    schema_id: &str,
    cred_def_id: &str,
    from: Option<u64>,
    to: Option<u64>,
) -> Value {
    attr_names_address_list()
        .iter()
        .map(|attr_name| {
            json!({
                "name": attr_name,
                "non_revoked": {"from": from, "to": to},
                "restrictions": [{
                  "issuer_did": did,
                  "schema_id": schema_id,
                  "cred_def_id": cred_def_id,
                }]
            })
        })
        .collect()
}

pub(super) fn requested_attr_objects(cred_def_id: &str) -> Vec<PresentationAttr> {
    credential_data_address_1()
        .as_object()
        .unwrap()
        .iter()
        .map(|(key, value)| {
            PresentationAttr::builder()
                .name(key.to_string())
                .cred_def_id(cred_def_id.to_owned())
                .value(value.to_string())
                .build()
        })
        .collect()
}

pub fn create_credential_proposal(
    schema_id: &str,
    cred_def_id: &str,
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
        .schema_id(schema_id.to_owned())
        .cred_def_id(cred_def_id.to_owned())
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
