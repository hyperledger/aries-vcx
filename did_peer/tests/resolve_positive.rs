mod fixtures;

use did_doc::schema::did_doc::DidDocument;
use did_doc_sov::extra_fields::ExtraFieldsSov;
use did_peer::peer_did_resolver::{
    options::{ExtraFieldsOptions, PublicKeyEncoding},
    resolver::PeerDidResolver,
};
use did_resolver::traits::resolvable::{resolution_options::DidResolutionOptions, DidResolvable};
use tokio::test;

use crate::fixtures::{
    basic::{DID_DOC_BASIC, PEER_DID_NUMALGO_2_BASIC},
    multiple_services::{DID_DOC_MULTIPLE_SERVICES, PEER_DID_NUMALGO_2_MULTIPLE_SERVICES},
    no_routing_keys::{DID_DOC_NO_ROUTING_KEYS, PEER_DID_NUMALGO_2_NO_ROUTING_KEYS},
    no_services::{DID_DOC_NO_SERVICES, PEER_DID_NUMALGO_2_NO_SERVICES},
};

macro_rules! resolve_positive_test {
    ($test_name:ident, $did_doc:expr, $peer_did:expr, $encoding:expr) => {
        #[test]
        async fn $test_name() {
            let resolver = PeerDidResolver::new();
            let options =
                DidResolutionOptions::new().set_extra(ExtraFieldsOptions::new().set_public_key_encoding($encoding));
            let did_document_expected = serde_json::from_str::<DidDocument<ExtraFieldsSov>>($did_doc).unwrap();
            let ddo = resolver
                .resolve(&$peer_did.parse().unwrap(), &options)
                .await
                .unwrap();
            let did_document_actual = ddo.did_document().clone();
            assert_eq!(did_document_actual, did_document_expected);
        }
    };
}

resolve_positive_test!(
    test_resolve_numalgo2_basic,
    DID_DOC_BASIC,
    PEER_DID_NUMALGO_2_BASIC,
    PublicKeyEncoding::Base58
);

resolve_positive_test!(
    test_resolve_numalgo2_multiple_services,
    DID_DOC_MULTIPLE_SERVICES,
    PEER_DID_NUMALGO_2_MULTIPLE_SERVICES,
    PublicKeyEncoding::Multibase
);

resolve_positive_test!(
    test_resolve_numalgo2_no_routing_keys,
    DID_DOC_NO_ROUTING_KEYS,
    PEER_DID_NUMALGO_2_NO_ROUTING_KEYS,
    PublicKeyEncoding::Multibase
);

resolve_positive_test!(
    test_resolve_numalgo2_no_services,
    DID_DOC_NO_SERVICES,
    PEER_DID_NUMALGO_2_NO_SERVICES,
    PublicKeyEncoding::Multibase
);
