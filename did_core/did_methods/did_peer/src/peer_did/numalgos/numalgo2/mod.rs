use did_doc::schema::did_doc::{DidDocument, DidDocumentBuilder};
use encoding::{append_encoded_key_segments, append_encoded_service_segment};
use sha256::digest;

use crate::{
    error::DidPeerError,
    peer_did::{
        numalgos::{numalgo2::helpers::didpeer_elements_to_diddoc, numalgo3::Numalgo3, Numalgo},
        FromDidDoc, PeerDid,
    },
    resolver::options::PublicKeyEncoding,
};

mod encoding;
mod helpers;
mod purpose;
mod service_abbreviation;
mod verification_method;

impl FromDidDoc for Numalgo2 {
    fn from_did_doc(did_document: DidDocument) -> Result<PeerDid<Numalgo2>, DidPeerError> {
        let mut did = String::from("did:peer:2");
        did = append_encoded_key_segments(did, &did_document)?;
        did = append_encoded_service_segment(did, &did_document)?;
        PeerDid::<Numalgo2>::parse(did)
    }
}

impl PeerDid<Numalgo2> {
    pub fn to_numalgo3(&self) -> Result<PeerDid<Numalgo3>, DidPeerError> {
        let numalgoless_id = self.did().id().chars().skip(2).collect::<String>();
        let numalgoless_id_hashed = digest(numalgoless_id);
        PeerDid::<Numalgo3>::parse(format!("did:peer:3.{}", numalgoless_id_hashed))
    }

    pub(crate) fn to_did_doc(
        &self,
        public_key_encoding: PublicKeyEncoding,
    ) -> Result<DidDocument, DidPeerError> {
        let mut did_doc_builder: DidDocumentBuilder = DidDocument::builder(self.did().clone());
        did_doc_builder =
            didpeer_elements_to_diddoc(did_doc_builder, self.did(), public_key_encoding)?;
        Ok(did_doc_builder.build())
    }
}

#[derive(Clone, Copy, Default, Debug, PartialEq)]
pub struct Numalgo2;

impl Numalgo for Numalgo2 {
    const NUMALGO_CHAR: char = '2';
}

#[cfg(test)]
mod test {
    use did_doc::schema::did_doc::DidDocument;
    use serde_json::{from_value, json};

    use crate::{
        peer_did::{numalgos::numalgo2::Numalgo2, PeerDid},
        resolver::options::PublicKeyEncoding,
    };

    #[test]
    fn test_peer_did_2_encode_decode() {
        let value = json!({
            "id": "did:peer:2.Ez6MkkukgyKAdBN46UAHvia2nxmioo74F6YdvW1nBT1wfKKha.Vz6MkfoapUdLHHgSMq5PYhdHYCoqGuRku2i17cQ9zAoR5cLSm.SeyJpZCI6IiMwIiwidCI6WyJkaWQtY29tbXVuaWNhdGlvbiJdLCJzIjoiaHR0cDovL2R1bW15dXJsLm9yZy8iLCJhIjpbImRpZGNvbW0vYWlwMjtlbnY9cmZjMTkiXX0",
            "verificationMethod": [
                {
                    "id": "#6MkfoapU",
                    "controller": "did:example:123456789abcdefghi",
                    "type": "Ed25519VerificationKey2020",
                    "publicKeyBase58": "2MKmtP5qx8wtiaYr24KhMiHH5rV3cpkkvPF4LXT4h7fP"
                }
            ],
            "keyAgreement": [
                {
                    "id": "#6Mkkukgy",
                    "controller": "did:example:123456789abcdefghi",
                    "type": "Ed25519VerificationKey2020",
                    "publicKeyBase58": "7TVeP4vBqpZdMfTE314x7gAoyXnPgfPZozsFcjyeQ6vC"
                }
            ],
            "service": [
                {
                    "id": "#0",
                    "type": [
                        "did-communication"
                    ],
                    "serviceEndpoint": "http://dummyurl.org/",
                    "routingKeys": [],
                    "accept": [
                        "didcomm/aip2;env=rfc19"
                    ],
                    "priority": 0,
                    "recipientKeys": [
                        "did:key:z6MkkukgyKAdBN46UAHvia2nxmioo74F6YdvW1nBT1wfKKha"
                    ]
                }
            ]
        });
        let ddo_original: DidDocument = from_value(value).unwrap();
        let did_peer: PeerDid<Numalgo2> = PeerDid::from_did_doc(ddo_original.clone()).unwrap();
        let ddo_decoded: DidDocument = did_peer.to_did_doc(PublicKeyEncoding::Base58).unwrap();
        assert_eq!(ddo_original, ddo_decoded);
    }
}
