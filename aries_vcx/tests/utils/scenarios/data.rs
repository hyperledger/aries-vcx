use messages::msg_fields::protocols::present_proof::propose::PresentationAttr;
use serde_json::{json, Value};

pub fn attr_names() -> (String, String, String, String, String) {
    let address1 = "Address1".to_string();
    let address2 = "address2".to_string();
    let city = "CITY".to_string();
    let state = "State".to_string();
    let zip = "zip".to_string();
    (address1, address2, city, state, zip)
}

pub fn requested_attrs(did: &str, schema_id: &str, cred_def_id: &str, from: Option<u64>, to: Option<u64>) -> Value {
    let (address1, address2, city, state, zip) = attr_names();
    json!([
       {
           "name": address1,
           "non_revoked": {"from": from, "to": to},
           "restrictions": [{
             "issuer_did": did,
             "schema_id": schema_id,
             "cred_def_id": cred_def_id,
           }]
       },
       {
           "name": address2,
           "non_revoked": {"from": from, "to": to},
           "restrictions": [{
             "issuer_did": did,
             "schema_id": schema_id,
             "cred_def_id": cred_def_id,
           }],
       },
       {
           "name": city,
           "non_revoked": {"from": from, "to": to},
           "restrictions": [{
             "issuer_did": did,
             "schema_id": schema_id,
             "cred_def_id": cred_def_id,
           }]
       },
       {
           "name": state,
           "non_revoked": {"from": from, "to": to},
           "restrictions": [{
             "issuer_did": did,
             "schema_id": schema_id,
             "cred_def_id": cred_def_id,
           }]
       },
       {
           "name": zip,
           "non_revoked": {"from": from, "to": to},
           "restrictions": [{
             "issuer_did": did,
             "schema_id": schema_id,
             "cred_def_id": cred_def_id,
           }]
       }
    ])
}

pub fn requested_attr_objects(cred_def_id: &str) -> Vec<PresentationAttr> {
    let (address1, address2, city, state, zip) = attr_names();
    let address1_attr = PresentationAttr::builder()
        .name(address1)
        .cred_def_id(cred_def_id.to_owned())
        .value("123 Main St".to_owned())
        .build();

    let address2_attr = PresentationAttr::builder()
        .name(address2)
        .cred_def_id(cred_def_id.to_owned())
        .value("Suite 3".to_owned())
        .build();

    let city_attr = PresentationAttr::builder()
        .name(city)
        .cred_def_id(cred_def_id.to_owned())
        .value("Draper".to_owned())
        .build();

    let state_attr = PresentationAttr::builder()
        .name(state)
        .cred_def_id(cred_def_id.to_owned())
        .value("UT".to_owned())
        .build();

    let zip_attr = PresentationAttr::builder()
        .name(zip)
        .cred_def_id(cred_def_id.to_owned())
        .value("84000".to_owned())
        .build();

    vec![address1_attr, address2_attr, city_attr, state_attr, zip_attr]
}
