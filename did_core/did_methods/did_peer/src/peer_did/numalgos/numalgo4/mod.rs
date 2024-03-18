// use did_doc::schema::did_doc::{DidDocument, DidDocumentBuilder};
// use sha256::digest;
//
// use crate::{
//     error::DidPeerError,
//     peer_did::{
//         numalgos::{numalgo3::Numalgo3, Numalgo},
//         FromDidDoc, PeerDid,
//     },
//     resolver::options::PublicKeyEncoding,
// };
// use crate::peer_did::numalgos::numalgo4::encoded_document::EncodedDocument;

use did_parser_nom::Did;

use crate::{
    error::DidPeerError,
    peer_did::{
        numalgos::{numalgo4::encoded_document::DidPeer4EncodedDocument, Numalgo},
        PeerDid,
    },
};

pub mod encoded_document;
mod encoding;
mod helpers;
mod verification_method;

// // The document MUST NOT include an id at the root. For DID Documents, this is populated with the
// DID itself. Since we are in the process of generating a DID, we do not yet know the value of the
// DID. When the DID is resolved later, this value will be correctly filled in. // All identifiers
// within this document MUST be relative. For example, the id of a verificationMethod might be
// #key-1 instead of something like did:example:abc123#key-1. // All references pointing to
// resources within this document MUST be relative. For example, a verification method reference in
// a verification relationship such as authentication might be #key-1 instead of something like
// did:example:abc123#key-1. // For verification methods, the controller MUST be omitted if the
// controller is the document owner. If it is controlled by a DID other than the owner of the
// document, it MUST be included. fn validate_did_document(did_document: EncodedDocument) ->
// Result<(), DidPeerError> { //     did_document.verification_method()
//
// }

#[derive(Clone, Copy, Default, Debug, PartialEq)]
pub struct Numalgo4;

impl Numalgo for Numalgo4 {
    const NUMALGO_CHAR: char = '4';
}

impl PeerDid<Numalgo4> {
    pub fn new(encoded_document: DidPeer4EncodedDocument) -> Result<Self, DidPeerError> {
        let serialized = serde_json::to_string(&encoded_document)?;
        let mut prefixed_bytes = Vec::new();
        prefixed_bytes.push(0x02u8); // multi-codec prefix for json is 0x0200, see https://github.com/multiformats/multicodec/blob/master/table.csv
        prefixed_bytes.push(0x00u8);
        prefixed_bytes.extend_from_slice(serialized.as_bytes());
        let encoded_document = multibase::encode(multibase::Base::Base58Btc, prefixed_bytes);
        // Take SHA2-256 digest of the encoded document (encode the bytes as utf-8)
        // Prefix these bytes with the multihash prefix for SHA2-256 and the hash length (varint
        // 0x12 for prefix, varint 0x20 for 32 bytes in length) Multibase encode the bytes
        // as base58btc (base58 encode the value and prefix with a z) Consider this value
        // the hash
        let hash_raw = sha256::digest(&encoded_document);
        let prefix = vec![0x12u8, 0x20u8];
        let hash = multibase::encode(
            multibase::Base::Base58Btc,
            [prefix.as_slice(), hash_raw.as_bytes()].concat(),
        );
        let did = Did::parse(format!("did:peer:4{}:{}", hash, encoded_document))?;
        Ok(Self {
            did,
            numalgo: Numalgo4,
        })
    }
}

#[cfg(test)]
mod tests {
    use std::collections::HashMap;

    use did_doc::schema::{
        service::{typed::ServiceType, Service},
        types::uri::Uri,
        utils::OneOrList,
    };

    use crate::peer_did::{
        numalgos::numalgo4::{encoded_document::DidPeer4EncodedDocumentBuilder, Numalgo4},
        PeerDid,
    };

    #[test]
    fn test_create_did_peer_4() {
        let service = Service::new(
            Uri::new("#service-0").unwrap(),
            "https://example.com/endpoint".parse().unwrap(),
            OneOrList::One(ServiceType::DIDCommV2),
            HashMap::default(),
        );
        let encoded_document = DidPeer4EncodedDocumentBuilder::default()
            .service(vec![service])
            .build()
            .unwrap();
        let did = PeerDid::<Numalgo4>::new(encoded_document).unwrap();
        assert_eq!(did.to_string(), "did:peer:4<hash>:Ez6MkkukgyKAdBN46UAHvia2nxmioo74F6YdvW1nBT1wfKKha.Vz6MkfoapUdLHHgSMq5PYhdHYCoqGuRku2i17cQ9zAoR5cLSm.SeyJpZCI6IiNmb29iYXIiLCJ0IjpbImRpZC1jb21tdW5pY2F0aW9uIl0sInMiOiJodHRwOi8vZHVtbXl1cmwub3JnLyIsInIiOlsiIzZNa2t1a2d5Il0sImEiOlsiZGlkY29tbS9haXAyO2Vudj1yZmMxOSJdfQ");
    }
}

//
// #[derive(Clone, Copy, Default, Debug, PartialEq)]
// pub struct Numalgo4;
//
// impl Numalgo for Numalgo4 {
//     const NUMALGO_CHAR: char = '2';
// }

// #[cfg(test)]
// mod test {
//     use did_doc::schema::did_doc::DidDocument;
//     use pretty_assertions::assert_eq;
//     use serde_json::{from_value, json};
//
//     use crate::{
//         peer_did::{numalgos::numalgo2::Numalgo2, PeerDid},
//         resolver::options::PublicKeyEncoding,
//     };
//
//     #[test]
//     fn test_peer_did_2_encode_decode() {
//         let expected_did_peer =
// "did:peer:2.Ez6MkkukgyKAdBN46UAHvia2nxmioo74F6YdvW1nBT1wfKKha.
// Vz6MkfoapUdLHHgSMq5PYhdHYCoqGuRku2i17cQ9zAoR5cLSm.
// SeyJpZCI6IiNmb29iYXIiLCJ0IjpbImRpZC1jb21tdW5pY2F0aW9uIl0sInMiOiJodHRwOi8vZHVtbXl1cmwub3JnLyIsInIiOlsiIzZNa2t1a2d5Il0sImEiOlsiZGlkY29tbS9haXAyO2Vudj1yZmMxOSJdfQ"
// ;         let value = json!({
//             "id": expected_did_peer,
//             "verificationMethod": [
//                 {
//                     "id": "#6MkfoapU",
//                     "controller": expected_did_peer,
//                     "type": "Ed25519VerificationKey2020",
//                     "publicKeyBase58": "2MKmtP5qx8wtiaYr24KhMiHH5rV3cpkkvPF4LXT4h7fP"
//                 }
//             ],
//             "keyAgreement": [
//                 {
//                     "id": "#6Mkkukgy",
//                     "controller": expected_did_peer,
//                     "type": "Ed25519VerificationKey2020",
//                     "publicKeyBase58": "7TVeP4vBqpZdMfTE314x7gAoyXnPgfPZozsFcjyeQ6vC"
//                 }
//             ],
//             "service": [
//                 {
//                     "id": "#foobar",
//                     "type": [
//                         "did-communication"
//                     ],
//                     "serviceEndpoint": "http://dummyurl.org/",
//                     "routingKeys": ["#6Mkkukgy"],
//                     "accept": [
//                         "didcomm/aip2;env=rfc19"
//                     ],
//                 }
//             ]
//         });
//         let ddo_original: DidDocument = from_value(value).unwrap();
//         let did_peer: PeerDid<Numalgo2> = PeerDid::from_did_doc(ddo_original.clone()).unwrap();
//         assert_eq!(did_peer.to_string(), expected_did_peer);
//
//         let ddo_decoded: DidDocument = did_peer
//             .to_did_doc_builder(PublicKeyEncoding::Base58)
//             .unwrap()
//             .build();
//         assert_eq!(ddo_original, ddo_decoded);
//     }
// }
