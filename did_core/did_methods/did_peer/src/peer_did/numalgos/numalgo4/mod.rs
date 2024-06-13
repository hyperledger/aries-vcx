use did_doc::schema::did_doc::DidDocument;
use did_parser_nom::Did;
use sha2::{Digest, Sha256};

use crate::{
    error::DidPeerError,
    helpers::{MULTICODEC_JSON_VARINT, MULTIHASH_SHA2_256},
    peer_did::{
        numalgos::{numalgo4::construction_did_doc::DidPeer4ConstructionDidDocument, Numalgo},
        PeerDid,
    },
};

pub mod construction_did_doc;

#[derive(Clone, Copy, Default, Debug, PartialEq)]
pub struct Numalgo4;

impl Numalgo for Numalgo4 {
    const NUMALGO_CHAR: char = '4';
}

impl PeerDid<Numalgo4> {
    /// Implementation of did:peer:4 creation spec:
    /// https://identity.foundation/peer-did-method-spec/#creating-a-did
    pub fn new(encoded_document: DidPeer4ConstructionDidDocument) -> Result<Self, DidPeerError> {
        let serialized = serde_json::to_vec(&encoded_document)?;
        let encoded_document = multibase::encode(
            multibase::Base::Base58Btc,
            [MULTICODEC_JSON_VARINT.as_slice(), &serialized].concat(),
        );

        let encoded_doc_digest = {
            let mut hasher = Sha256::new();
            hasher.update(encoded_document.as_bytes());
            hasher.finalize()
        };
        let hash = multibase::encode(
            multibase::Base::Base58Btc,
            [MULTIHASH_SHA2_256.as_slice(), &encoded_doc_digest].concat(),
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

    pub fn short_form(&self) -> Did {
        let parts = self.did().id().split(':').collect::<Vec<&str>>();
        let short_form_id = match parts.first() {
            None => {
                return self.did().clone(); // the DID was short form already
            }
            Some(hash_part) => hash_part,
        };
        let short_form_did = format!("did:peer:{}", short_form_id);
        let parse_result = Did::parse(short_form_did).map_err(|e| {
            DidPeerError::GeneralError(format!("Failed to parse short form of PeerDid: {}", e))
        });
        // ** safety note (panic) **
        // This should only panic if the parser is inherently buggy. We rely on following
        // assumptions:
        //   - `did:peer:` is a valid DID prefix
        //   - `short_form_did` is substring/prefix of `self.id()`, without colons, and therefore
        //     valid DID ID
        //   - every peer-did includes hash component followed prefix "did:peer:"
        parse_result.expect("Failed to parse short form of PeerDid")
    }

    pub fn hash(&self) -> Result<String, DidPeerError> {
        let short_form_did = self.short_form();
        let hash = short_form_did.id()[1..].to_string(); // the first character of id did:peer:4 ID is always "4", followed by hash
        Ok(hash)
    }

    fn encoded_did_peer_4_document(&self) -> Option<&str> {
        let did = self.did();
        did.id().split(':').collect::<Vec<_>>().get(1).copied()
    }

    fn to_did_peer_4_encoded_diddoc(
        &self,
    ) -> Result<DidPeer4ConstructionDidDocument, DidPeerError> {
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
        let encoded_document: DidPeer4ConstructionDidDocument =
            serde_json::from_slice(peer4_did_doc).map_err(|e| {
                DidPeerError::GeneralError(format!("Failed to decode the encoded did doc: {}", e))
            })?;
        Ok(encoded_document)
    }

    pub fn resolve_did_doc(&self) -> Result<DidDocument, DidPeerError> {
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
            construction_did_doc::{DidPeer4ConstructionDidDocument, DidPeer4VerificationMethod},
            Numalgo4,
        },
        PeerDid,
    };

    fn prepare_verification_method(key_id: &str) -> DidPeer4VerificationMethod {
        DidPeer4VerificationMethod::builder()
            .id(DidUrl::parse(key_id.to_string()).unwrap())
            .verification_method_type(VerificationMethodType::Ed25519VerificationKey2020)
            .public_key(PublicKeyField::Base58 {
                public_key_base58: "z27uFkiq".to_string(),
            })
            .build()
    }

    #[test]
    fn test_create_did_peer_4() {
        let service = Service::new(
            Uri::new("#service-0").unwrap(),
            "https://example.com/endpoint".parse().unwrap(),
            OneOrList::One(ServiceType::DIDCommV2),
            HashMap::default(),
        );
        let vm = prepare_verification_method("#shared-key-1");
        let vm_ka = prepare_verification_method("#key_agreement-1");
        let vm_auth = prepare_verification_method("#key-authentication-1");
        let vm_deleg = prepare_verification_method("#key-delegation-1");
        let vm_invoc = prepare_verification_method("#key-invocation-1");

        let mut construction_did_doc = DidPeer4ConstructionDidDocument::new();
        construction_did_doc.add_service(service);
        construction_did_doc.add_verification_method(vm);
        construction_did_doc.add_key_agreement(vm_ka);
        construction_did_doc.add_authentication(vm_auth);
        construction_did_doc.add_capability_delegation(vm_deleg);
        construction_did_doc.add_capability_invocation(vm_invoc);

        let did = PeerDid::<Numalgo4>::new(construction_did_doc).unwrap();
        let did_expected = "did:peer:4zQmcaWgiE7Q6ERzDovHKrQXEPC1bd7X4YAv9Hb9Pw1Qhjtm:zMeTVLzkiLyX6Wj4CLuZ7WoX3ZSxFQVFBYQy5vfZmgZUCuFcsVeuWMrvhznUxei6NqGDqoE4rYF88ptgdQxFmrCj7fcqxdvzBjncMDsyYXYLzucXydU2N1cXSZH8rN5srWEftBUg4SsVrF8upEMJ81Yts9fBTitgrCzQGdpPGnBtWKh6C21uVBM1wShxrG5FGcisduzRnGDKLCEGxKDZkBrSaTsWk7a2kTMi45nouL8VnXY9DkfcyJTYwvdNC67BXpdp44ksvvtveYH6WqmvCNhPi7RSEdYoD7ZZYDBboRvbuAQANQSMPqpmj9L8Zz62oDxYJffFqK8yjdhBfanyqtQMCnxm7w1rDhExZdUK6BSQL3utQ1LeeZmHLZmXdsQp3nkTLrW4AwWnoTVEGSgnSDsgRiZkma9YaZgnZVhnQeNj4fjes6aZKwEjqLChabXfrXU6d6EJ9v4tRrHxatxdEdM5wmgCMxEVFhpSjcLQu7bpULkj63GUQMVWyWcXyd8UsP56rMWgyutCRtfA4Mm7sVKgcp7324fzAB6VLXLoHy6dPdhrZ2eYcB4Qgke12PpF37xFViScqEdQovUDPWiH41Kz898T3chUvDyFCP6ugXmwJTXGgSTxLT5gpBEjvbshbrV6jAFy4wWnBeaQVvDAG7DgDa7jhGqj3Grg4NHzPLFne37GKrWW1dxtfL2D8XKyLb6tMsny6bAqdrtt5LhRazuaXgGmWCtksLZeaPrjdFKa4Gevj6BiJ67RC15HoecGsZYUtQdXzBPkVvyvWbzbh3UaEYppCP6yq8ZGYc7AeGX8YnatwmhoECdoV1GdZAdRAoqj4pQmn4mTjHUfZ3ZmPQxYqhiAxrdzG51MDVc182YLWhMBbShxxUXs11b1UT4JVXizwzoArZNzg7PQekbRcauHHaRLRcQ";
        assert_eq!(did.to_string(), did_expected);

        let resolved_did_doc = did.resolve_did_doc().unwrap();
        println!(
            "resolved document: {}",
            serde_json::to_string_pretty(&resolved_did_doc).unwrap()
        );
        assert_eq!(resolved_did_doc.id().to_string(), did.did().to_string());
        assert!(resolved_did_doc
            .verification_method_by_id("shared-key-1")
            .is_some());
        assert!(resolved_did_doc
            .key_agreement_by_id("key_agreement-1")
            .is_some());
        assert!(resolved_did_doc
            .authentication_by_id("key-authentication-1")
            .is_some());
        assert!(resolved_did_doc
            .capability_delegation_by_id("key-delegation-1")
            .is_some());
        assert!(resolved_did_doc
            .capability_invocation_by_id("key-invocation-1")
            .is_some());
        log::info!(
            "resolved document: {}",
            serde_json::to_string_pretty(&resolved_did_doc).unwrap()
        );
    }

    #[test]
    fn long_form_to_short_form() {
        let peer_did = "did:peer:4z84UjLJ6ugExV8TJ5gJUtZap5q67uD34LU26m1Ljo2u9PZ4xHa9XnknHLc3YMST5orPXh3LKi6qEYSHdNSgRMvassKP:z27uFkiqJVwvvn2ke5M19UCvByS79r5NppqwjiGAJzkj1EM4sf2JmiUySkANKy4YNu8M7yKjSmvPJTqbcyhPrJs9TASzDs2fWE1vFegmaRJxHRF5M9wGTPwGR1NbPkLGsvcnXum7aN2f8kX3BnhWWWp";
        let peer_did = PeerDid::<Numalgo4>::parse(peer_did).unwrap();
        assert_eq!(peer_did.short_form().to_string(), "did:peer:4z84UjLJ6ugExV8TJ5gJUtZap5q67uD34LU26m1Ljo2u9PZ4xHa9XnknHLc3YMST5orPXh3LKi6qEYSHdNSgRMvassKP".to_string());
    }

    #[test]
    fn short_form_to_short_form() {
        let peer_did = "did:peer:4z84UjLJ6ugExV8TJ5gJUtZap5q67uD34LU26m1Ljo2u9PZ4xHa9XnknHLc3YMST5orPXh3LKi6qEYSHdNSgRMvassKP";
        let peer_did = PeerDid::<Numalgo4>::parse(peer_did).unwrap();
        assert_eq!(peer_did.short_form().to_string(), "did:peer:4z84UjLJ6ugExV8TJ5gJUtZap5q67uD34LU26m1Ljo2u9PZ4xHa9XnknHLc3YMST5orPXh3LKi6qEYSHdNSgRMvassKP".to_string());
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
