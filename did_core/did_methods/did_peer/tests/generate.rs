mod fixtures;

use did_doc::schema::did_doc::DidDocument;
use did_doc_sov::extra_fields::ExtraFieldsSov;
use did_peer::peer_did::{
    numalgos::{numalgo2::Numalgo2, numalgo3::Numalgo3},
    PeerDid,
};

use crate::fixtures::{
    basic::{DID_DOC_BASIC, PEER_DID_NUMALGO_2_BASIC, PEER_DID_NUMALGO_3_BASIC},
    multiple_services::{
        DID_DOC_MULTIPLE_SERVICES, PEER_DID_NUMALGO_2_MULTIPLE_SERVICES,
        PEER_DID_NUMALGO_3_MULTIPLE_SERVICES,
    },
    no_routing_keys::{
        DID_DOC_NO_ROUTING_KEYS, PEER_DID_NUMALGO_2_NO_ROUTING_KEYS,
        PEER_DID_NUMALGO_3_NO_ROUTING_KEYS,
    },
    no_services::{
        DID_DOC_NO_SERVICES, PEER_DID_NUMALGO_2_NO_SERVICES, PEER_DID_NUMALGO_3_NO_SERVICES,
    },
};

macro_rules! generate_test_numalgo2 {
    ($test_name:ident, $did_doc:expr, $peer_did:expr) => {
        #[test]
        fn $test_name() {
            let did_document =
                serde_json::from_str::<DidDocument>($did_doc).unwrap();
            assert_eq!(
                PeerDid::<Numalgo2>::parse($peer_did.to_string()).unwrap(),
                PeerDid::<Numalgo2>::from_did_doc(did_document).unwrap()
            );
        }
    };
}

macro_rules! generate_test_numalgo3 {
    ($test_name:ident, $did_doc:expr, $peer_did:expr) => {
        #[test]
        fn $test_name() {
            let did_document =
                serde_json::from_str::<DidDocument>($did_doc).unwrap();
            assert_eq!(
                PeerDid::<Numalgo3>::parse($peer_did.to_string()).unwrap(),
                PeerDid::<Numalgo3>::from_did_doc(did_document).unwrap()
            );
        }
    };
}

generate_test_numalgo2!(
    test_generate_numalgo2_basic,
    DID_DOC_BASIC,
    PEER_DID_NUMALGO_2_BASIC
);

generate_test_numalgo2!(
    test_generate_numalgo2_multiple_services,
    DID_DOC_MULTIPLE_SERVICES,
    PEER_DID_NUMALGO_2_MULTIPLE_SERVICES
);

generate_test_numalgo2!(
    test_generate_numalgo2_no_services,
    DID_DOC_NO_SERVICES,
    PEER_DID_NUMALGO_2_NO_SERVICES
);

generate_test_numalgo2!(
    test_generate_numalgo2_no_routing_keys,
    DID_DOC_NO_ROUTING_KEYS,
    PEER_DID_NUMALGO_2_NO_ROUTING_KEYS
);

generate_test_numalgo3!(
    test_generate_numalgo3_basic,
    DID_DOC_BASIC,
    PEER_DID_NUMALGO_3_BASIC
);

generate_test_numalgo3!(
    test_generate_numalgo3_multiple_services,
    DID_DOC_MULTIPLE_SERVICES,
    PEER_DID_NUMALGO_3_MULTIPLE_SERVICES
);

generate_test_numalgo3!(
    test_generate_numalgo3_no_services,
    DID_DOC_NO_SERVICES,
    PEER_DID_NUMALGO_3_NO_SERVICES
);

generate_test_numalgo3!(
    test_generate_numalgo3_no_routing_keys,
    DID_DOC_NO_ROUTING_KEYS,
    PEER_DID_NUMALGO_3_NO_ROUTING_KEYS
);
