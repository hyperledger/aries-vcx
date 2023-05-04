use std::sync::Arc;
use std::thread;
use std::time::Duration;

use aries_vcx::core::profile::profile::Profile;
use aries_vcx::{
    common::ledger::{
        service_didsov::{DidSovServiceType, EndpointDidSov},
        transactions::write_endpoint,
    },
    utils::devsetup::SetupProfile,
};
use did_resolver::did_parser::ParsedDid;
use did_resolver::traits::resolvable::{resolution_options::DidResolutionOptions, DidResolvable};
use did_resolver_sov::reader::ConcreteAttrReader;
use did_resolver_sov::resolution::DidSovResolver;

async fn write_test_endpoint(profile: &Arc<dyn Profile>, did: &str) {
    let endpoint = EndpointDidSov::create()
        .set_service_endpoint("http://localhost:8080".parse().unwrap())
        .set_routing_keys(Some(vec!["key1".to_string(), "key2".to_string()]))
        .set_types(Some(vec![DidSovServiceType::Endpoint]));
    write_endpoint(profile, did, &endpoint).await.unwrap();
    thread::sleep(Duration::from_millis(50));
}

#[tokio::test]
async fn write_service_on_ledger_and_resolve_did_doc() {
    SetupProfile::run(|init| async move {
        let did = format!("did:sov:{}", init.institution_did);
        write_test_endpoint(&init.profile, &init.institution_did).await;
        let resolver = DidSovResolver::new(Arc::<ConcreteAttrReader>::new(
            init.profile.inject_ledger().into(),
        ));
        let did_doc = resolver
            .resolve(
                &ParsedDid::parse(did.clone()).unwrap(),
                &DidResolutionOptions::default(),
            )
            .await
            .unwrap();
        assert_eq!(did_doc.did_document().id().to_string(), did);
    })
    .await;
}

#[tokio::test]
async fn test_error_handling_during_resolution() {
    SetupProfile::run(|init| async move {
        let did = format!("did:unknownmethod:{}", init.institution_did);

        let resolver = DidSovResolver::new(Arc::<ConcreteAttrReader>::new(
            init.profile.inject_ledger().into(),
        ));

        let result = resolver
            .resolve(
                &ParsedDid::parse(did.clone()).unwrap(),
                &DidResolutionOptions::default(),
            )
            .await;

        assert!(result.is_err());
    })
    .await;
}
