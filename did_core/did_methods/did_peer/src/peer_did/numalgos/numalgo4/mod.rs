use did_doc::schema::did_doc::DidDocument;
use did_parser_nom::Did;

use crate::{
    error::DidPeerError,
    peer_did::{
        numalgos::{numalgo4::encoded_document::DidPeer4EncodedDocument, Numalgo},
        PeerDid,
    },
};

pub mod encoded_document;

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

    pub fn long_form(&self) -> Result<Did, DidPeerError> {
        self.encoded_did_peer_4_document()
            .ok_or(DidPeerError::GeneralError(format!(
                "Long form is not available for peer did: {}",
                self.did
            )))?;
        Ok(self.did().clone())
    }

    pub fn short_form(&self) -> Result<Did, DidPeerError> {
        let short_id = self
            .did()
            .id()
            .to_string()
            .split(':')
            .collect::<Vec<&str>>()[0]
            .to_string();
        Did::parse(format!("did:peer:{}", short_id)).map_err(|e| {
            DidPeerError::GeneralError(format!("Failed to parse short form of PeerDid: {}", e))
        })
    }

    pub fn hash(&self) -> Result<String, DidPeerError> {
        let short_form_did = self.short_form()?;
        let hash = short_form_did.id()[1..].to_string(); // the first character of id did:peer:4 ID is always "4", followed by hash
        Ok(hash)
    }

    fn encoded_did_peer_4_document(&self) -> Option<&str> {
        let did = self.did();
        did.id().split(':').collect::<Vec<_>>().get(1).copied()
    }

    fn to_did_peer_4_encoded_diddoc(&self) -> Result<DidPeer4EncodedDocument, DidPeerError> {
        let encoded_did_doc =
            self.encoded_did_peer_4_document()
                .ok_or(DidPeerError::GeneralError(format!(
                    "to_did_peer_4_encoded_diddoc >> Long form is not available for peer did: {}",
                    self.did
                )))?;
        let (_base, diddoc_with_multibase_prefix) =
            multibase::decode(encoded_did_doc).map_err(|e| {
                DidPeerError::GeneralError(format!(
                    "Failed to decode multibase prefix from encoded did doc: {}",
                    e
                ))
            })?;
        // without first 2 bytes
        let peer4_did_doc: &[u8] = &diddoc_with_multibase_prefix[2..];
        let encoded_document: DidPeer4EncodedDocument =
            serde_json::from_slice(peer4_did_doc).unwrap();
        Ok(encoded_document)
    }

    pub fn decode_did_doc(&self) -> Result<DidDocument, DidPeerError> {
        let did_doc_peer4_encoded = self.to_did_peer_4_encoded_diddoc()?;
        Ok(did_doc_peer4_encoded.contextualize_to_did_doc(self))
    }
}

#[cfg(test)]
mod tests {
    use std::collections::HashMap;

    use did_doc::schema::{
        service::{typed::ServiceType, Service},
        types::uri::Uri,
        utils::OneOrList,
        verification_method::{PublicKeyField, VerificationMethodType},
    };
    use did_parser_nom::DidUrl;

    use crate::peer_did::{
        numalgos::numalgo4::{
            encoded_document::{DidPeer4EncodedDocumentBuilder, DidPeer4VerificationMethod},
            Numalgo4,
        },
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
        let vm = DidPeer4VerificationMethod {
            id: DidUrl::parse("#key-1".to_string()).unwrap(),
            verification_method_type: VerificationMethodType::Ed25519VerificationKey2020,
            public_key: PublicKeyField::Base58 {
                public_key_base58: "z27uFkiq".to_string(),
            },
        };
        let encoded_document = DidPeer4EncodedDocumentBuilder::default()
            .service(vec![service])
            .verification_method(vec![vm])
            .build()
            .unwrap();
        println!(
            "original didpeer4 document: {}",
            serde_json::to_string_pretty(&encoded_document).unwrap()
        );
        let did = PeerDid::<Numalgo4>::new(encoded_document).unwrap();
        assert_eq!(did.to_string(), "did:peer:4z84Vmeih9kTUrnxVanw9DhiVX9JNuW5cEz1RJx9dwrKcqh4bq96Z6zuc9m6oPV4gc6tafguyzd8dYih4N153Gh3XmWK:z2FrKwFgfDgrV5fdpSvPvBThURtNvDa3RWfoueUsEVQQmzJpMxXhAiutkPRRbuvVVeJDMZd2wdjeeNsRPx1csnDyQsoyhQWviaBd2LRen8fp9vZSkzmFmP1sgoKDXztkREhiUnKbXCiArA6t2nKed2NoGALYXFw1D72NbSgEhcMVzLL2wwgovV4D1HhEcvzXJQDKXwqUDaW1B3YgCMBKeEvy4vsaYhxf7JFcZzS5Ga8mSSUk3nAC9nXMWG3GT8XxzviQWxdfB2fwyKoy3bC3ihxwwjkpxVNuB72mJ");

        let resolved_did_doc = did.decode_did_doc().unwrap();
        assert_eq!(resolved_did_doc.id().to_string(), did.did().to_string());
        println!(
            "resolved document: {}",
            serde_json::to_string_pretty(&resolved_did_doc).unwrap()
        );
    }

    #[test]
    fn long_form_to_short_form() {
        let peer_did = "did:peer:4z84UjLJ6ugExV8TJ5gJUtZap5q67uD34LU26m1Ljo2u9PZ4xHa9XnknHLc3YMST5orPXh3LKi6qEYSHdNSgRMvassKP:z27uFkiqJVwvvn2ke5M19UCvByS79r5NppqwjiGAJzkj1EM4sf2JmiUySkANKy4YNu8M7yKjSmvPJTqbcyhPrJs9TASzDs2fWE1vFegmaRJxHRF5M9wGTPwGR1NbPkLGsvcnXum7aN2f8kX3BnhWWWp";
        let peer_did = PeerDid::<Numalgo4>::parse(peer_did).unwrap();
        assert_eq!(peer_did.short_form().unwrap().to_string(), "did:peer:4z84UjLJ6ugExV8TJ5gJUtZap5q67uD34LU26m1Ljo2u9PZ4xHa9XnknHLc3YMST5orPXh3LKi6qEYSHdNSgRMvassKP".to_string());
    }

    #[test]
    fn short_form_to_short_form() {
        let peer_did = "did:peer:4z84UjLJ6ugExV8TJ5gJUtZap5q67uD34LU26m1Ljo2u9PZ4xHa9XnknHLc3YMST5orPXh3LKi6qEYSHdNSgRMvassKP";
        let peer_did = PeerDid::<Numalgo4>::parse(peer_did).unwrap();
        assert_eq!(peer_did.short_form().unwrap().to_string(), "did:peer:4z84UjLJ6ugExV8TJ5gJUtZap5q67uD34LU26m1Ljo2u9PZ4xHa9XnknHLc3YMST5orPXh3LKi6qEYSHdNSgRMvassKP".to_string());
    }

    #[test]
    fn long_form_to_long_form() {
        let peer_did = "did:peer:4z84UjLJ6ugExV8TJ5gJUtZap5q67uD34LU26m1Ljo2u9PZ4xHa9XnknHLc3YMST5orPXh3LKi6qEYSHdNSgRMvassKP:z27uFkiqJVwvvn2ke5M19UCvByS79r5NppqwjiGAJzkj1EM4sf2JmiUySkANKy4YNu8M7yKjSmvPJTqbcyhPrJs9TASzDs2fWE1vFegmaRJxHRF5M9wGTPwGR1NbPkLGsvcnXum7aN2f8kX3BnhWWWp";
        let peer_did = PeerDid::<Numalgo4>::parse(peer_did).unwrap();
        assert_eq!(peer_did.long_form().unwrap().to_string(), "did:peer:4z84UjLJ6ugExV8TJ5gJUtZap5q67uD34LU26m1Ljo2u9PZ4xHa9XnknHLc3YMST5orPXh3LKi6qEYSHdNSgRMvassKP:z27uFkiqJVwvvn2ke5M19UCvByS79r5NppqwjiGAJzkj1EM4sf2JmiUySkANKy4YNu8M7yKjSmvPJTqbcyhPrJs9TASzDs2fWE1vFegmaRJxHRF5M9wGTPwGR1NbPkLGsvcnXum7aN2f8kX3BnhWWWp".to_string());
    }

    #[test]
    fn short_form_to_long_form_fails() {
        let peer_did = "did:peer:4z84UjLJ6ugExV8TJ5gJUtZap5q67uD34LU26m1Ljo2u9PZ4xHa9XnknHLc3YMST5orPXh3LKi6qEYSHdNSgRMvassKP";
        let peer_did = PeerDid::<Numalgo4>::parse(peer_did).unwrap();
        peer_did.long_form().unwrap_err();
    }
}
