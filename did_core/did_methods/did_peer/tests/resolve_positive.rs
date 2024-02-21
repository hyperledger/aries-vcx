mod fixtures;

use did_doc::schema::did_doc::DidDocument;
use did_peer::resolver::{options::PublicKeyEncoding, PeerDidResolutionOptions, PeerDidResolver};
use did_resolver::traits::resolvable::DidResolvable;
use pretty_assertions::assert_eq;
use tokio::test;

use crate::fixtures::{
    basic::{DID_DOC_BASIC, PEER_DID_NUMALGO_2_BASIC},
    no_routing_keys::{DID_DOC_NO_ROUTING_KEYS, PEER_DID_NUMALGO_2_NO_ROUTING_KEYS},
    no_services::{DID_DOC_NO_SERVICES, PEER_DID_NUMALGO_2_NO_SERVICES},
};

async fn resolve_positive_test(did_doc: &str, peer_did: &str, options: PeerDidResolutionOptions) {
    let did_document_expected = serde_json::from_str::<DidDocument>(did_doc).unwrap();
    let resolution = PeerDidResolver
        .resolve(&peer_did.parse().unwrap(), &options)
        .await
        .unwrap();
    assert_eq!(resolution.did_document, did_document_expected);
}

#[test]
async fn test_resolve_numalgo2_basic() {
    let options = PeerDidResolutionOptions {
        encoding: Some(PublicKeyEncoding::Base58),
    };
    resolve_positive_test(DID_DOC_BASIC, PEER_DID_NUMALGO_2_BASIC, options).await;
}

#[test]
async fn test_resolve_numalgo2_no_routing_keys() {
    let options = PeerDidResolutionOptions {
        encoding: Some(PublicKeyEncoding::Multibase),
    };
    resolve_positive_test(
        DID_DOC_NO_ROUTING_KEYS,
        PEER_DID_NUMALGO_2_NO_ROUTING_KEYS,
        options,
    )
    .await;
}

#[test]
async fn test_resolve_numalgo2_no_services() {
    let options = PeerDidResolutionOptions {
        encoding: Some(PublicKeyEncoding::Multibase),
    };
    resolve_positive_test(DID_DOC_NO_SERVICES, PEER_DID_NUMALGO_2_NO_SERVICES, options).await;
}
