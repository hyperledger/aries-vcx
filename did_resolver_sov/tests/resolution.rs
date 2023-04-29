use std::pin::Pin;
use std::sync::Arc;
use std::time::Duration;
use std::{num::NonZeroUsize, thread};

use aries_vcx::core::profile::profile::Profile;
use aries_vcx::{
    common::ledger::{
        service_didsov::{DidSovServiceType, EndpointDidSov},
        transactions::write_endpoint,
    },
    utils::devsetup::SetupProfile,
};
use did_resolver::did_parser::ParsedDID;
use did_resolver::traits::resolvable::resolution_output::DIDResolutionOutput;
use did_resolver::traits::resolvable::{
    resolution_options::DIDResolutionOptions, DIDResolvable, DIDResolvableMut,
};
use did_resolver_sov::reader::{ConcreteAttrReader, MockAttrReader};
use did_resolver_sov::resolution::DIDSovResolver;
use mockall::predicate::eq;

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
        let resolver = DIDSovResolver::new(
            Arc::<ConcreteAttrReader>::new(init.profile.inject_ledger().into()),
            NonZeroUsize::new(10).unwrap(),
        );
        let did_doc = resolver
            .resolve(
                &ParsedDID::parse(did.clone()).unwrap(),
                &DIDResolutionOptions::default(),
            )
            .await
            .unwrap();
        assert_eq!(did_doc.did_document().id().to_string(), did);
    })
    .await;
}

#[tokio::test]
async fn test_resolver_caching_behavior() {
    SetupProfile::run(|init| async move {
        async fn resolve_did_doc(
            did: String,
            resolver: &mut DIDSovResolver,
        ) -> DIDResolutionOutput {
            let did_doc = resolver
                .resolve_mut(
                    &ParsedDID::parse(did.clone()).unwrap(),
                    &DIDResolutionOptions::default(),
                )
                .await
                .unwrap();
            assert_eq!(did_doc.did_document().id().to_string(), did);
            did_doc
        }

        let did = format!("did:sov:{}", init.institution_did);
        write_test_endpoint(&init.profile, &init.institution_did).await;

        let cache_size = NonZeroUsize::new(2).unwrap();
        let actual_ledger_response = init
            .profile
            .inject_ledger()
            .get_attr(&did, "endpoint")
            .await
            .unwrap();
        let mut mock_reader = MockAttrReader::new();
        mock_reader
            .expect_get_attr()
            .with(eq(did.clone()), eq("endpoint"))
            .once()
            .return_once(move |_, _| {
                let future = async move { Ok(actual_ledger_response.clone()) };
                Pin::from(Box::new(future))
            });

        let arc_mock_reader = Arc::new(mock_reader);
        let mut resolver = DIDSovResolver::new(arc_mock_reader, cache_size);

        let did_doc = resolve_did_doc(did.clone(), &mut resolver).await;
        let did_doc_cached = resolve_did_doc(did, &mut resolver).await;

        assert_eq!(did_doc.did_document(), did_doc_cached.did_document());
    })
    .await;
}

#[tokio::test]
async fn test_error_handling_during_resolution() {
    SetupProfile::run(|init| async move {
        let did = format!("did:unknownmethod:{}", init.institution_did);

        let cache_size = NonZeroUsize::new(2).unwrap();
        let resolver = DIDSovResolver::new(
            Arc::<ConcreteAttrReader>::new(init.profile.inject_ledger().into()),
            cache_size,
        );

        let result = resolver
            .resolve(
                &ParsedDID::parse(did.clone()).unwrap(),
                &DIDResolutionOptions::default(),
            )
            .await;

        assert!(result.is_err());
    })
    .await;
}
