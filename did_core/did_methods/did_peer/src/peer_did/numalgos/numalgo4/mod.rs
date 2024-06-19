use did_doc::schema::did_doc::DidDocument;
use did_parser_nom::Did;

use crate::{
    error::DidPeerError,
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
        let serialized = serde_json::to_string(&encoded_document)?;
        let mut prefixed_bytes = Vec::new();
        prefixed_bytes.push(0x02u8); // multi-codec prefix for json is 0x0200, see https://github.com/multiformats/multicodec/blob/master/table.csv
        prefixed_bytes.push(0x00u8);
        prefixed_bytes.extend_from_slice(serialized.as_bytes());
        let encoded_document = multibase::encode(multibase::Base::Base58Btc, prefixed_bytes);
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
            serde_json::from_slice(peer4_did_doc).unwrap();
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
        service::{service_key_kind::ServiceKeyKind, typed::ServiceType, Service},
        types::uri::Uri,
        utils::OneOrList,
        verification_method::{PublicKeyField, VerificationMethodType},
    };
    use did_parser_nom::DidUrl;
    use public_key::KeyType;

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
        let did_expected = "did:peer:4z84UnqbnKs1uK42FKCz3QCr6JFMFYBZk2cPzkhHXETruZZpcdu6PtPyxzRdTYEdq7jWTWtqKykYxbtgmx34DUqyYEnT:zKj4qM7Uj9TKfiRp9DfxJMy1X7vBJEG9p7GnBbLLhA226Yh1SoYrcK942ZmBpAQXuXLLkohiPdJAYjbaatt2fFmhBvSR39zWMsWNqGBqmKW8vuy7uHwXKmuChtngE1WyMCM8r5DvwjubumVYq8uZWaurrdvzkdX57aM5y32kH5oFYjpBFDmugQAvzYP1VkTL8zF3G6wVHKvdAAzp8KracRWF7M6KnPHz3psRJ41Lktdk4NyNTfPZc1ztFV1v95ECazdRKwpFwmyZ9Gs5JhSCx1zYd9Ki4Zongb1VBCgFNVCjiADoUDfkNKoVu4QfzFGd1wCQEJFaYTcZ9N55r5cY215bAHq9fudyUvn2EeGd8FCfPyHPZxHZE5yFsKXmtauTLGJyFuWt31Kk31JN35zRui4nu3HRcnQNmkxADjVYnC7o4cb1D1DYKk2i3xL7mSwsDXUg2NN4mXfnM2JN6WFmGrDe2wKiKW4qc6w4RtmzGPN68LJVDh49dpwsaZM4hdkWc8s1D8mztRBbMDxKtJkDeZE23kpHCMfcfzQQCcPdpoXnpsNMoFaWh1q96cp72J1yNN4bQnSNxzNvMLjfdatmZrrqCkYd8ND4qU5eaE4HibjNsfx87dR2FN6mor8ktrXDirp1JezwU3NepeeHfs5EnY8V4aeMw3LAAEv4qrVTnr4jYmQHdfYVRJnP3qGkWwuHB2x9QhhMy8M4j5xHkNtVdV8Z9CFFuUGrnrPKvBMn2P5W5G3b4MxnWydmDdhdwirQAmHpJwRFoMDW9R55ckGhA6ZVo88BMwwCtKkeWibTHxjcHYdahYjYKzC6ZJGzU3DfHdmcprrN2oLRDcEPDxe6EejPSUu12iTHsCCV9r3txtuC2hvkRtmsgLChVD4Fh6VH22SzetPf513mToJvjfdVSdqaiftxj6";
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

    #[test]
    fn test_resolve_acapy_test_vector1() {
        let peer_did: &str = "did:peer:4zQmcQCH8nWEBBA6BpSEDxHyhPwHdi5CVGcvsZcjhb618zbA:z5CTtVoAxKjH1V1sKizLy5kLvV6AbmACYfcGmfVUDGn4A7BpnVQEESXEYYUG7W479kDHaqLnk7NJuu4w7ftTd9REipB2CQgW9fjzPvmsXyyHzot9o1tgYHNnqFDXgCXwFYJfjkzz3m6mex1WMN4XHWWNM4NB7exDA2maVGis7gJnVAiNrBExaihyeKJ4nBXrB3ArQ1TyuZ39F9qTeCSrBntTTa85wtUtHz5M1oE7Sj1CZeAEQzDnAMToP9idSrSXUo5z8q9Un325d8MtQgxyKGW2a9VYyW189C722GKQbGQSU3dRSwCanVHJwCh9q2G2eNVPeuydAHXmouCUCq3cVHeUkatv73DSoBV17LEJgq8dAYfvSAutG7LFyvrRW5wNjcQMT7WdFHRCqhtzz18zu6fSTQWM4PQPLMVEaKbs51EeYGiGurhu1ChQMjXqnpcRcpCP7RAEgyWSjMER6e3gdCVsBhQSoqGk1UN8NfVah8pxGg2i5Gd1754Ys6aBEhTashFa47Ke7oPoZ6LZiRMETYhUr1cQY65TQhMzyrR6RzLudeRVgcRdKiTTmP2fFi5H8nCHPSGb4wncUxgn3N5CbFaUC";
        let peer_did = PeerDid::<Numalgo4>::parse(peer_did).unwrap();

        let resolved_did_doc = peer_did.resolve_did_doc().unwrap();
        assert_eq!(resolved_did_doc.id().to_string(), "did:peer:4zQmcQCH8nWEBBA6BpSEDxHyhPwHdi5CVGcvsZcjhb618zbA:z5CTtVoAxKjH1V1sKizLy5kLvV6AbmACYfcGmfVUDGn4A7BpnVQEESXEYYUG7W479kDHaqLnk7NJuu4w7ftTd9REipB2CQgW9fjzPvmsXyyHzot9o1tgYHNnqFDXgCXwFYJfjkzz3m6mex1WMN4XHWWNM4NB7exDA2maVGis7gJnVAiNrBExaihyeKJ4nBXrB3ArQ1TyuZ39F9qTeCSrBntTTa85wtUtHz5M1oE7Sj1CZeAEQzDnAMToP9idSrSXUo5z8q9Un325d8MtQgxyKGW2a9VYyW189C722GKQbGQSU3dRSwCanVHJwCh9q2G2eNVPeuydAHXmouCUCq3cVHeUkatv73DSoBV17LEJgq8dAYfvSAutG7LFyvrRW5wNjcQMT7WdFHRCqhtzz18zu6fSTQWM4PQPLMVEaKbs51EeYGiGurhu1ChQMjXqnpcRcpCP7RAEgyWSjMER6e3gdCVsBhQSoqGk1UN8NfVah8pxGg2i5Gd1754Ys6aBEhTashFa47Ke7oPoZ6LZiRMETYhUr1cQY65TQhMzyrR6RzLudeRVgcRdKiTTmP2fFi5H8nCHPSGb4wncUxgn3N5CbFaUC");
        assert_eq!(
            resolved_did_doc.also_known_as()[0].to_string(),
            "did:peer:4zQmcQCH8nWEBBA6BpSEDxHyhPwHdi5CVGcvsZcjhb618zbA"
        );

        // vm/key
        assert_eq!(resolved_did_doc.verification_method().len(), 1);
        let vm = resolved_did_doc.verification_method_by_id("key-0").unwrap();
        assert_eq!(
            vm.verification_method_type(),
            &VerificationMethodType::Multikey
        );
        assert_eq!(
            vm.public_key_field(),
            &PublicKeyField::Multibase {
                public_key_multibase: String::from(
                    "z6MkuNenWjqDeZ4DjkHoqX6WdDYTfUUqcR7ASezo846GHe74"
                )
            }
        );
        let key = vm.public_key().unwrap();
        assert_eq!(
            key.fingerprint(),
            "z6MkuNenWjqDeZ4DjkHoqX6WdDYTfUUqcR7ASezo846GHe74"
        );
        assert_eq!(key.key_type(), &KeyType::Ed25519);

        // servie
        assert_eq!(resolved_did_doc.service().len(), 1);
        let service = resolved_did_doc
            .get_service_by_id(&"#didcomm-0".parse().unwrap())
            .unwrap();
        assert_eq!(
            service.service_type(),
            &OneOrList::One(ServiceType::DIDCommV1)
        );
        assert_eq!(
            service.service_endpoint().to_string(),
            "http://host.docker.internal:9031/"
        );
        let service_recip = service
            .extra_field_recipient_keys()
            .unwrap();
        assert_eq!(service_recip, vec![ServiceKeyKind::Reference("#key-0".parse().unwrap())]);
        log::info!(
            "resolved document: {}",
            serde_json::to_string_pretty(&resolved_did_doc).unwrap()
        );
    }
}
