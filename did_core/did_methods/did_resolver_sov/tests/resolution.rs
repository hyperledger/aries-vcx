use std::{thread, time::Duration};

use aries_vcx::common::ledger::{service_didsov::EndpointDidSov, transactions::write_endpoint};
use aries_vcx_ledger::ledger::base_ledger::IndyLedgerWrite;
use aries_vcx_wallet::wallet::base_wallet::{did_wallet::DidWallet, BaseWallet};
use did_resolver::{
    did_doc::schema::service::typed::ServiceType,
    did_parser_nom::Did,
    traits::resolvable::{resolution_output::DidResolutionOutput, DidResolvable},
};
use did_resolver_sov::resolution::DidSovResolver;
use test_utils::devsetup::build_setup_profile;

async fn write_test_endpoint(
    wallet: &impl BaseWallet,
    ledger_write: &impl IndyLedgerWrite,
    did: &Did,
) {
    let endpoint = EndpointDidSov::create()
        .set_service_endpoint("http://localhost:8080".parse().unwrap())
        .set_routing_keys(Some(vec!["key1".to_string(), "key2".to_string()]))
        .set_types(Some(vec![ServiceType::AIP1.to_string()]));
    write_endpoint(wallet, ledger_write, did, &endpoint)
        .await
        .unwrap();
    thread::sleep(Duration::from_millis(50));
}

#[tokio::test]
async fn write_service_on_ledger_and_resolve_did_doc() {
    let profile = build_setup_profile().await;
    write_test_endpoint(
        &profile.wallet,
        &profile.ledger_write,
        &profile.institution_did,
    )
    .await;
    let resolver = DidSovResolver::new(profile.ledger_read);
    let did = format!("did:sov:{}", profile.institution_did);

    let DidResolutionOutput { did_document, .. } = resolver
        .resolve(&Did::parse(did.clone()).unwrap(), &())
        .await
        .unwrap();

    assert_eq!(did_document.id().to_string(), did);
}

#[tokio::test]
async fn test_error_handling_during_resolution() {
    let profile = build_setup_profile().await;
    let resolver = DidSovResolver::new(profile.ledger_read);
    let did = format!("did:unknownmethod:{}", profile.institution_did);

    let result = resolver
        .resolve(&Did::parse(did.clone()).unwrap(), &())
        .await;

    assert!(result.is_err());
}

#[tokio::test]
async fn write_new_nym_and_get_did_doc() {
    let profile = build_setup_profile().await;
    let did_data = profile
        .wallet
        .create_and_store_my_did(None, None)
        .await
        .unwrap();

    profile
        .ledger_write
        .publish_nym(
            &profile.wallet,
            &profile.institution_did,
            &did_data.did().parse().unwrap(),
            Some(did_data.verkey()),
            None,
            None,
        )
        .await
        .unwrap();

    let resolver = DidSovResolver::new(profile.ledger_read);
    let did = format!("did:sov:{}", did_data.did());

    let DidResolutionOutput { did_document, .. } = resolver
        .resolve(&Did::parse(did.clone()).unwrap(), &())
        .await
        .unwrap();
    assert_eq!(did_document.id().to_string(), did);
}
