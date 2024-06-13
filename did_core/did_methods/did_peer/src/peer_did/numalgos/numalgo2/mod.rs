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
    use did_doc::schema::did_doc::DidDocument;
    use pretty_assertions::assert_eq;
    use serde_json::{from_value, json};

    use crate::{
        peer_did::{numalgos::numalgo2::Numalgo2, PeerDid},
        resolver::options::PublicKeyEncoding,
    };

    #[test]
    fn test_peer_did_2_encode_decode() {
        let expected_did_peer = "did:peer:2.Ez6MkkukgyKAdBN46UAHvia2nxmioo74F6YdvW1nBT1wfKKha.Vz6MkfoapUdLHHgSMq5PYhdHYCoqGuRku2i17cQ9zAoR5cLSm.SeyJpZCI6IiNmb29iYXIiLCJ0IjpbImRpZC1jb21tdW5pY2F0aW9uIl0sInMiOiJodHRwOi8vZHVtbXl1cmwub3JnLyIsInIiOlsiIzZNa2t1a2d5Il0sImEiOlsiZGlkY29tbS9haXAyO2Vudj1yZmMxOSJdfQ";
        let value = json!({
            "id": expected_did_peer,
            "verificationMethod": [
                {
                    "id": "#6MkfoapU",
                    "controller": expected_did_peer,
                    "type": "Ed25519VerificationKey2020",
                    "publicKeyBase58": "2MKmtP5qx8wtiaYr24KhMiHH5rV3cpkkvPF4LXT4h7fP"
                }
            ],
            "keyAgreement": [
                {
                    "id": "#6Mkkukgy",
                    "controller": expected_did_peer,
                    "type": "Ed25519VerificationKey2020",
                    "publicKeyBase58": "7TVeP4vBqpZdMfTE314x7gAoyXnPgfPZozsFcjyeQ6vC"
                }
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
            .to_did_doc_builder(PublicKeyEncoding::Base58)
            .unwrap();
        assert_eq!(ddo_original, ddo_decoded);
    }
}
