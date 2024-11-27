use did_doc::schema::did_doc::DidDocument;
use encoding::{append_encoded_key_segments, append_encoded_service_segment};
use sha2::{Digest, Sha256};

use crate::{
    error::DidPeerError,
    helpers::MULTIHASH_SHA2_256,
    peer_did::{
        numalgos::{numalgo2::helpers::diddoc_from_peerdid2_elements, numalgo3::Numalgo3, Numalgo},
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
        let numalgoless_id = self.did().id().chars().skip(1).collect::<String>();
        let numalgoless_id_hashed = {
            let mut hasher = Sha256::new();
            hasher.update(numalgoless_id.as_bytes());
            hasher.finalize()
        };

        let bytes = [MULTIHASH_SHA2_256.as_slice(), &numalgoless_id_hashed[..]].concat();

        let multibase_hash = multibase::encode(multibase::Base::Base58Btc, bytes);
        PeerDid::<Numalgo3>::parse(format!("did:peer:3{}", multibase_hash))
    }

    pub(crate) fn to_did_doc_builder(
        &self,
        public_key_encoding: PublicKeyEncoding,
    ) -> Result<DidDocument, DidPeerError> {
        let mut did_doc_builder: DidDocument = DidDocument::new(self.did().clone());
        did_doc_builder =
            diddoc_from_peerdid2_elements(did_doc_builder, self.did(), public_key_encoding)?;
        Ok(did_doc_builder)
    }
}

#[derive(Clone, Copy, Default, Debug, PartialEq)]
pub struct Numalgo2;

impl Numalgo for Numalgo2 {
    const NUMALGO_CHAR: char = '2';
}

#[cfg(test)]
mod test {
    use did_doc::schema::{
        did_doc::DidDocument, service::service_key_kind::ServiceKeyKind,
        verification_method::PublicKeyField,
    };
    use did_parser_nom::DidUrl;
    use pretty_assertions::assert_eq;
    use serde_json::{from_value, json};

    use crate::{
        peer_did::{numalgos::numalgo2::Numalgo2, PeerDid},
        resolver::options::PublicKeyEncoding,
    };

    #[test]
    fn test_peer_did_2_encode_decode() {
        // NOTE 20/6/24: universal resolver resolves an additional "assertionMethod" key for the "V"
        // key despite the spec not saying to do this.
        let expected_did_peer = "did:peer:2.Ez6MkkukgyKAdBN46UAHvia2nxmioo74F6YdvW1nBT1wfKKha.Vz6MkfoapUdLHHgSMq5PYhdHYCoqGuRku2i17cQ9zAoR5cLSm.SeyJpZCI6IiNmb29iYXIiLCJ0IjpbImRpZC1jb21tdW5pY2F0aW9uIl0sInMiOiJodHRwOi8vZHVtbXl1cmwub3JnLyIsInIiOlsiIzZNa2t1a2d5Il0sImEiOlsiZGlkY29tbS9haXAyO2Vudj1yZmMxOSJdfQ";
        let value = json!({
            "id": expected_did_peer,
            "verificationMethod": [
                {
                    "id": "#key-1",
                    "controller": expected_did_peer,
                    "type": "Ed25519VerificationKey2020",
                    "publicKeyMultibase": "z6MkkukgyKAdBN46UAHvia2nxmioo74F6YdvW1nBT1wfKKha"
                },
                {
                    "id": "#key-2",
                    "controller": expected_did_peer,
                    "type": "Ed25519VerificationKey2020",
                    "publicKeyMultibase": "z6MkfoapUdLHHgSMq5PYhdHYCoqGuRku2i17cQ9zAoR5cLSm"
                }
            ],
            "keyAgreement": [
                "#key-1"
            ],
            "authentication": [
                "#key-2"
            ],
            "service": [
                {
                    "id": "#foobar",
                    "type": [
                        "did-communication"
                    ],
                    "serviceEndpoint": "http://dummyurl.org/",
                    "routingKeys": ["#6Mkkukgy"],
                    "accept": [
                        "didcomm/aip2;env=rfc19"
                    ],
                }
            ]
        });
        let ddo_original: DidDocument = from_value(value).unwrap();
        let did_peer: PeerDid<Numalgo2> = PeerDid::from_did_doc(ddo_original.clone()).unwrap();
        assert_eq!(did_peer.to_string(), expected_did_peer);

        let ddo_decoded: DidDocument = did_peer
            .to_did_doc_builder(PublicKeyEncoding::Multibase)
            .unwrap();
        assert_eq!(ddo_original, ddo_decoded);
    }

    #[test]
    fn test_acapy_did_peer_2() {
        // test vector from AATH testing with acapy 0.12.1
        let did = "did:peer:2.Vz6MkqY3gWxHEp47gCXBmnc5k7sAQChwV76YpZAHZ8erDHatK.SeyJpZCI6IiNkaWRjb21tLTAiLCJ0IjoiZGlkLWNvbW11bmljYXRpb24iLCJwcmlvcml0eSI6MCwicmVjaXBpZW50S2V5cyI6WyIja2V5LTEiXSwiciI6W10sInMiOiJodHRwOi8vaG9zdC5kb2NrZXIuaW50ZXJuYWw6OTAzMSJ9";
        let did = PeerDid::<Numalgo2>::parse(did).unwrap();

        let doc = did
            .to_did_doc_builder(PublicKeyEncoding::Multibase)
            .unwrap();
        assert_eq!(doc.verification_method().len(), 1);
        let vm = doc.verification_method_by_id("key-1").unwrap();
        assert_eq!(
            vm.public_key().unwrap().fingerprint(),
            "z6MkqY3gWxHEp47gCXBmnc5k7sAQChwV76YpZAHZ8erDHatK"
        );
        assert_eq!(
            vm.public_key_field(),
            &PublicKeyField::Multibase {
                public_key_multibase: String::from(
                    "z6MkqY3gWxHEp47gCXBmnc5k7sAQChwV76YpZAHZ8erDHatK"
                )
            }
        );

        assert_eq!(doc.service().len(), 1);
        let service = doc
            .get_service_by_id(&"#didcomm-0".parse().unwrap())
            .unwrap();
        assert_eq!(
            service.service_endpoint().to_string(),
            "http://host.docker.internal:9031/"
        );
        let recips = service.extra_field_recipient_keys().unwrap();
        assert_eq!(recips.len(), 1);
        assert_eq!(
            recips[0],
            ServiceKeyKind::Reference(DidUrl::parse("#key-1".to_string()).unwrap())
        );
    }

    #[test]
    fn test_resolving_spec_defined_example() {
        // https://identity.foundation/peer-did-method-spec/#example-peer-did-2
        // NOTE: excluding the services, as they use a different type of service to the typical
        // service DIDDoc structure
        let did = "did:peer:2.Vz6Mkj3PUd1WjvaDhNZhhhXQdz5UnZXmS7ehtx8bsPpD47kKc.\
                   Ez6LSg8zQom395jKLrGiBNruB9MM6V8PWuf2FpEy4uRFiqQBR";
        let did = PeerDid::<Numalgo2>::parse(did).unwrap();

        let doc = did
            .to_did_doc_builder(PublicKeyEncoding::Multibase)
            .unwrap();
        let expected_doc: DidDocument = serde_json::from_value(json!({
            "id": "did:peer:2.Vz6Mkj3PUd1WjvaDhNZhhhXQdz5UnZXmS7ehtx8bsPpD47kKc.Ez6LSg8zQom395jKLrGiBNruB9MM6V8PWuf2FpEy4uRFiqQBR",
            "verificationMethod": [
              {
                "id": "#key-1",
                "controller": "did:peer:2.Vz6Mkj3PUd1WjvaDhNZhhhXQdz5UnZXmS7ehtx8bsPpD47kKc.Ez6LSg8zQom395jKLrGiBNruB9MM6V8PWuf2FpEy4uRFiqQBR",
                "type": "Ed25519VerificationKey2020",
                "publicKeyMultibase": "z6Mkj3PUd1WjvaDhNZhhhXQdz5UnZXmS7ehtx8bsPpD47kKc"
              },
              {
                "id": "#key-2",
                "controller": "did:peer:2.Vz6Mkj3PUd1WjvaDhNZhhhXQdz5UnZXmS7ehtx8bsPpD47kKc.Ez6LSg8zQom395jKLrGiBNruB9MM6V8PWuf2FpEy4uRFiqQBR",
                "type": "X25519KeyAgreementKey2020",
                "publicKeyMultibase": "z6LSg8zQom395jKLrGiBNruB9MM6V8PWuf2FpEy4uRFiqQBR"
              }
            ],
            "authentication": [
              "#key-1"
            ],
            "keyAgreement": [
              "#key-2"
            ]
        })).unwrap();
        assert_eq!(doc, expected_doc);
    }
}
