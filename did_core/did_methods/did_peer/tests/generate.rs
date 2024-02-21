mod fixtures;

use did_doc::schema::did_doc::DidDocument;
use did_peer::peer_did::{
    numalgos::{numalgo2::Numalgo2, numalgo3::Numalgo3},
    PeerDid,
};
use pretty_assertions::assert_eq;

use crate::fixtures::{
    basic::{DID_DOC_BASIC, PEER_DID_NUMALGO_2_BASIC, PEER_DID_NUMALGO_3_BASIC},
    no_routing_keys::{
        DID_DOC_NO_ROUTING_KEYS, PEER_DID_NUMALGO_2_NO_ROUTING_KEYS,
        PEER_DID_NUMALGO_3_NO_ROUTING_KEYS,
    },
    no_services::{
        DID_DOC_NO_SERVICES, PEER_DID_NUMALGO_2_NO_SERVICES, PEER_DID_NUMALGO_3_NO_SERVICES,
    },
};

fn test_numalgo2(did_doc: &str, expected_peer_did: &str) {
    let did_document = serde_json::from_str::<DidDocument>(did_doc).unwrap();
    assert_eq!(
        PeerDid::<Numalgo2>::parse(expected_peer_did.to_string()).unwrap(),
        PeerDid::<Numalgo2>::from_did_doc(did_document).unwrap()
    );
}

fn test_numalgo3(did_doc: &str, expected_peer_did: &str) {
    let did_document = serde_json::from_str::<DidDocument>(did_doc).unwrap();
    assert_eq!(
        PeerDid::<Numalgo3>::parse(expected_peer_did.to_string()).unwrap(),
        PeerDid::<Numalgo3>::from_did_doc(did_document).unwrap()
    );
}

#[test]
fn test_generate_numalgo2_basic() {
    test_numalgo2(DID_DOC_BASIC, PEER_DID_NUMALGO_2_BASIC);
}

#[test]
fn test_generate_numalgo2_no_services() {
    test_numalgo2(DID_DOC_NO_SERVICES, PEER_DID_NUMALGO_2_NO_SERVICES);
}

#[test]
fn test_generate_numalgo2_no_routing_keys() {
    test_numalgo2(DID_DOC_NO_ROUTING_KEYS, PEER_DID_NUMALGO_2_NO_ROUTING_KEYS);
}

#[test]
fn test_generate_numalgo3_basic() {
    test_numalgo3(DID_DOC_BASIC, PEER_DID_NUMALGO_3_BASIC);
}

#[test]
fn test_generate_numalgo3_no_services() {
    test_numalgo3(DID_DOC_NO_SERVICES, PEER_DID_NUMALGO_3_NO_SERVICES);
}

#[test]
fn test_generate_numalgo3_no_routing_keys() {
    test_numalgo3(DID_DOC_NO_ROUTING_KEYS, PEER_DID_NUMALGO_3_NO_ROUTING_KEYS);
}
