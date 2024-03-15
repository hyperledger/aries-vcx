use did_doc::schema::verification_method::{
    IncompleteVerificationMethodBuilder, VerificationMethod, VerificationMethodType,
};
use did_parser::{Did, DidUrl};
use public_key::{Key, KeyType};

use crate::{error::DidPeerError, resolver::options::PublicKeyEncoding};

pub fn get_verification_methods_by_key(
    key: &Key,
    did: &Did,
    public_key_encoding: PublicKeyEncoding,
) -> Result<Vec<VerificationMethod>, DidPeerError> {
    let id = to_did_url_reference(key)?;
    let vm_type = match key.key_type() {
        KeyType::Ed25519 => VerificationMethodType::Ed25519VerificationKey2020,
        KeyType::Bls12381g1 => VerificationMethodType::Bls12381G1Key2020,
        KeyType::Bls12381g2 => VerificationMethodType::Bls12381G2Key2020,
        KeyType::X25519 => VerificationMethodType::X25519KeyAgreementKey2020,
        KeyType::P256 => VerificationMethodType::JsonWebKey2020,
        KeyType::P384 => VerificationMethodType::JsonWebKey2020,
        KeyType::P521 => VerificationMethodType::JsonWebKey2020,
        KeyType::Bls12381g1g2 => {
            return Ok(build_verification_methods_from_bls_multikey(
                &Key::new(key.key()[..48].to_vec(), KeyType::Bls12381g1)?,
                &Key::new(key.key()[48..].to_vec(), KeyType::Bls12381g2)?,
                did.to_owned(),
                public_key_encoding,
            ));
        }
    };
    Ok(build_verification_methods_from_type_and_key(
        vm_type,
        key,
        id,
        did.to_owned(),
        public_key_encoding,
    ))
}

pub fn get_key_by_verification_method(vm: &VerificationMethod) -> Result<Key, DidPeerError> {
    let key_type = match vm.verification_method_type() {
        VerificationMethodType::Ed25519VerificationKey2018
        | VerificationMethodType::Ed25519VerificationKey2020 => KeyType::Ed25519,
        VerificationMethodType::Bls12381G1Key2020 => KeyType::Bls12381g1,
        VerificationMethodType::Bls12381G2Key2020 => KeyType::Bls12381g2,
        VerificationMethodType::X25519KeyAgreementKey2019
        | VerificationMethodType::X25519KeyAgreementKey2020 => KeyType::X25519,
        t => {
            return Err(DidPeerError::UnsupportedVerificationMethodType(
                t.to_owned(),
            ));
        }
    };
    Ok(Key::new(vm.public_key_field().key_decoded()?, key_type)?)
}

fn build_verification_methods_from_type_and_key(
    vm_type: VerificationMethodType,
    key: &Key,
    id: DidUrl,
    did: Did,
    public_key_encoding: PublicKeyEncoding,
) -> Vec<VerificationMethod> {
    vec![add_public_key_to_builder(
        VerificationMethod::builder(id, did, vm_type),
        key,
        public_key_encoding,
    )]
}

fn build_verification_methods_from_bls_multikey(
    g1_key: &Key,
    g2_key: &Key,
    did: Did,
    public_key_encoding: PublicKeyEncoding,
) -> Vec<VerificationMethod> {
    let id1 = to_did_url_reference(g1_key).unwrap();
    let id2 = to_did_url_reference(g2_key).unwrap();

    let vm1 = add_public_key_to_builder(
        VerificationMethod::builder(
            id1,
            did.to_owned(),
            VerificationMethodType::Bls12381G1Key2020,
        ),
        g1_key,
        public_key_encoding,
    );
    let vm2 = add_public_key_to_builder(
        VerificationMethod::builder(id2, did, VerificationMethodType::Bls12381G2Key2020),
        g2_key,
        public_key_encoding,
    );
    vec![vm1, vm2]
}

fn add_public_key_to_builder(
    builder: IncompleteVerificationMethodBuilder,
    key: &Key,
    public_key_encoding: PublicKeyEncoding,
) -> VerificationMethod {
    match public_key_encoding {
        PublicKeyEncoding::Base58 => builder.add_public_key_base58(key.base58()).build(),
        PublicKeyEncoding::Multibase => builder.add_public_key_multibase(key.fingerprint()).build(),
    }
}

fn to_did_url_reference(key: &Key) -> Result<DidUrl, DidPeerError> {
    DidUrl::from_fragment(
        key.prefixless_fingerprint()
            .chars()
            .take(8)
            .collect::<String>(),
    )
    .map_err(Into::into)
}

#[cfg(test)]
mod tests {
    use did_doc::schema::verification_method::{VerificationMethod, VerificationMethodType};
    use did_parser::Did;
    use public_key::Key;

    fn did() -> Did {
        "did:peer:2.Ez6LSbysY2xFMRpGMhb7tFTLMpeuPRaqaWM1yECx2AtzE3KCc.\
         Vz6MkqRYqQiSgvZQdnBytw86Qbs2ZWUkGv22od935YF4s8M7V.\
         Vz6MkgoLTnTypo3tDRwCkZXSccTPHRLhF4ZnjhueYAFpEX6vg"
            .parse()
            .unwrap()
    }

    fn key_0() -> Key {
        Key::from_fingerprint("z6LSbysY2xFMRpGMhb7tFTLMpeuPRaqaWM1yECx2AtzE3KCc").unwrap()
    }

    fn key_1() -> Key {
        Key::from_fingerprint("z6MkqRYqQiSgvZQdnBytw86Qbs2ZWUkGv22od935YF4s8M7V").unwrap()
    }

    fn key_2() -> Key {
        Key::from_fingerprint("z6MkgoLTnTypo3tDRwCkZXSccTPHRLhF4ZnjhueYAFpEX6vg").unwrap()
    }

    fn verification_method_0() -> VerificationMethod {
        VerificationMethod::builder(
            did().into(),
            did(),
            VerificationMethodType::X25519KeyAgreementKey2020,
        )
        .add_public_key_multibase(key_0().fingerprint())
        .build()
    }

    fn verification_method_1() -> VerificationMethod {
        VerificationMethod::builder(
            did().into(),
            did(),
            VerificationMethodType::Ed25519VerificationKey2020,
        )
        .add_public_key_multibase(key_1().fingerprint())
        .build()
    }

    fn verification_method_2() -> VerificationMethod {
        VerificationMethod::builder(
            did().into(),
            did(),
            VerificationMethodType::Ed25519VerificationKey2020,
        )
        .add_public_key_multibase(key_2().fingerprint())
        .build()
    }

    mod get_verification_methods_by_key {
        use super::*;
        use crate::{
            peer_did::numalgos::numalgo2::verification_method, resolver::options::PublicKeyEncoding,
        };

        // Multibase encoded keys are multicodec-prefixed by their encoding type ...
        fn test_get_verification_methods_by_key_multibase(key: &Key) {
            let vms = verification_method::get_verification_methods_by_key(
                key,
                &did(),
                PublicKeyEncoding::Multibase,
            )
            .unwrap();
            assert_eq!(vms.len(), 1);
            assert_eq!(
                vms[0].public_key_field().key_decoded().unwrap(),
                key.multicodec_prefixed_key()
            );
            assert_ne!(vms[0].public_key_field().key_decoded().unwrap(), key.key());
        }

        // ... and base58 encoded keys are not
        fn test_get_verification_methods_by_key_base58(key: &Key) {
            let vms = verification_method::get_verification_methods_by_key(
                key,
                &did(),
                PublicKeyEncoding::Base58,
            )
            .unwrap();
            assert_eq!(vms.len(), 1);
            assert_ne!(
                vms[0].public_key_field().key_decoded().unwrap(),
                key.multicodec_prefixed_key()
            );
            assert_eq!(vms[0].public_key_field().key_decoded().unwrap(), key.key());
        }

        #[test]
        fn test_get_verification_methods_by_key_multibase_0() {
            test_get_verification_methods_by_key_multibase(&key_0());
        }

        #[test]
        fn test_get_verification_methods_by_key_multibase_1() {
            test_get_verification_methods_by_key_multibase(&key_1());
        }

        #[test]
        fn test_get_verification_methods_by_key_multibase_2() {
            test_get_verification_methods_by_key_multibase(&key_2());
        }

        #[test]
        fn test_get_verification_methods_by_key_base58_0() {
            test_get_verification_methods_by_key_base58(&key_0());
        }

        #[test]
        fn test_get_verification_methods_by_key_base58_1() {
            test_get_verification_methods_by_key_base58(&key_1());
        }

        #[test]
        fn test_get_verification_methods_by_key_base58_2() {
            test_get_verification_methods_by_key_base58(&key_2());
        }
    }

    mod get_key_by_verification_method {
        use super::*;
        use crate::peer_did::numalgos::numalgo2::verification_method;

        #[test]
        fn test_get_key_by_verification_method_0() {
            assert_eq!(
                verification_method::get_key_by_verification_method(&verification_method_0())
                    .unwrap(),
                key_0()
            );
        }

        #[test]
        fn test_get_key_by_verification_method_1() {
            assert_eq!(
                verification_method::get_key_by_verification_method(&verification_method_1())
                    .unwrap(),
                key_1()
            );
        }

        #[test]
        fn test_get_key_by_verification_method_2() {
            assert_eq!(
                verification_method::get_key_by_verification_method(&verification_method_2())
                    .unwrap(),
                key_2()
            );
        }
    }
}
