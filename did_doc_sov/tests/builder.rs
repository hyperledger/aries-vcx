use did_doc::schema::types::{uri::Uri, url::Url};
use did_doc_sov::{
    extra_fields::{aip1::ExtraFieldsAIP1, didcommv1::ExtraFieldsDidCommV1, didcommv2::ExtraFieldsDidCommV2},
    service::{aip1::ServiceAIP1, didcommv1::ServiceDidCommV1, didcommv2::ServiceDidCommV2, ServiceSov},
    DidDocumentSov,
};

const ID: &'static str = "did:sov:WRfXPg8dantKVubE3HX8pw";
const SERVICE_ENDPOINT: &'static str = "https://example.com";

#[test]
fn test_service_build_aip1() {
    let service = ServiceAIP1::new(
        ID.parse().unwrap(),
        SERVICE_ENDPOINT.parse().unwrap(),
        ExtraFieldsAIP1::default(),
    )
    .unwrap();
    let did_doc = DidDocumentSov::builder()
        .add_service(ServiceSov::AIP1(service))
        .unwrap()
        .build();
    assert_eq!(did_doc.service().len(), 1);
    let first_service = did_doc.service().get(0).unwrap();
    assert_eq!(first_service.id().clone(), ID.parse::<Uri>().unwrap());
    assert_eq!(
        first_service.service_endpoint().clone(),
        SERVICE_ENDPOINT.parse::<Url>().unwrap()
    );
    let first_extra = first_service.extra();
    assert!(first_extra.priority().is_err());
    assert!(first_extra.recipient_keys().is_err());
    assert!(first_extra.routing_keys().is_err());
}

#[test]
fn test_service_build_didcommv1() {
    let extra_fields_didcommv1 = ExtraFieldsDidCommV1::builder()
        .set_priority(1)
        .set_routing_keys(vec!["foo".to_owned().into()])
        .set_recipient_keys(vec!["bar".to_owned().into()])
        .build();
    let service = ServiceDidCommV1::new(
        ID.parse().unwrap(),
        SERVICE_ENDPOINT.parse().unwrap(),
        extra_fields_didcommv1,
    )
    .unwrap();
    let did_doc = DidDocumentSov::builder()
        .add_service(ServiceSov::DIDCommV1(service))
        .unwrap()
        .build();
    assert_eq!(did_doc.service().len(), 1);
    let first_service = did_doc.service().get(0).unwrap();
    assert_eq!(first_service.id().clone(), ID.parse::<Uri>().unwrap());
    assert_eq!(
        first_service.service_endpoint().clone(),
        SERVICE_ENDPOINT.parse::<Url>().unwrap()
    );
    let first_extra = first_service.extra();
    assert_eq!(first_extra.priority().unwrap(), 1);
    assert_eq!(first_extra.recipient_keys().unwrap(), &["bar".to_owned().into()]);
    assert_eq!(first_extra.routing_keys().unwrap(), &["foo".to_owned().into()]);
}

#[test]
fn test_service_build_didcommv2() {
    let extra_fields_didcommv2 = ExtraFieldsDidCommV2::builder()
        .set_routing_keys(vec!["foo".to_owned().into()])
        .build();
    let service = ServiceDidCommV2::new(
        ID.parse().unwrap(),
        SERVICE_ENDPOINT.parse().unwrap(),
        extra_fields_didcommv2,
    )
    .unwrap();
    let did_doc = DidDocumentSov::builder()
        .add_service(ServiceSov::DIDCommV2(service))
        .unwrap()
        .build();
    assert_eq!(did_doc.service().len(), 1);
    let first_service = did_doc.service().get(0).unwrap();
    assert_eq!(first_service.id().clone(), ID.parse::<Uri>().unwrap());
    assert_eq!(
        first_service.service_endpoint().clone(),
        SERVICE_ENDPOINT.parse::<Url>().unwrap()
    );
    let first_extra = first_service.extra();
    assert!(first_extra.priority().is_err());
    assert!(first_extra.recipient_keys().is_err());
    assert_eq!(first_extra.routing_keys().unwrap(), &["foo".to_owned().into()]);
}
