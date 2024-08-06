use did_doc::schema::verification_method::{
    PublicKeyField, VerificationMethod, VerificationMethodType,
};
use did_parser_nom::{Did, DidUrl};
use public_key::{Key, KeyType};

use crate::{error::DidPeerError, resolver::options::PublicKeyEncoding};

pub fn get_verification_methods_by_key(
    key: &Key,
    did: &Did,
    public_key_encoding: PublicKeyEncoding,
    vm_index: &mut usize,
) -> Result<Vec<VerificationMethod>, DidPeerError> {
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
                vm_index,
            ));
        }
    };

    build_verification_methods_from_type_and_key(
        vm_type,
        key,
        did.to_owned(),
        public_key_encoding,
        vm_index,
    )
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
    did: Did,
    public_key_encoding: PublicKeyEncoding,
    vm_index: &mut usize,
) -> Result<Vec<VerificationMethod>, DidPeerError> {
    let id = nth_key_did_url_reference(*vm_index)?;
    *vm_index += 1;

    let vm = VerificationMethod::builder()
        .id(id)
        .controller(did)
        .verification_method_type(vm_type)
        .public_key(key_to_key_field(key, public_key_encoding))
        .build();
    Ok(vec![vm])
}

fn build_verification_methods_from_bls_multikey(
    g1_key: &Key,
    g2_key: &Key,
    did: Did,
    public_key_encoding: PublicKeyEncoding,
    vm_index: &mut usize,
) -> Vec<VerificationMethod> {
    let id1 = nth_key_did_url_reference(*vm_index).unwrap();
    *vm_index += 1;
    let id2 = nth_key_did_url_reference(*vm_index).unwrap();
    *vm_index += 1;
    let vm1 = VerificationMethod::builder()
        .id(id1)
        .controller(did.to_owned())
        .verification_method_type(VerificationMethodType::Bls12381G1Key2020)
        .public_key(key_to_key_field(g1_key, public_key_encoding))
        .build();
    let vm2 = VerificationMethod::builder()
        .id(id2)
        .controller(did.to_owned())
        .verification_method_type(VerificationMethodType::Bls12381G2Key2020)
        .public_key(key_to_key_field(g2_key, public_key_encoding))
        .build();
    vec![vm1, vm2]
}

fn key_to_key_field(key: &Key, public_key_encoding: PublicKeyEncoding) -> PublicKeyField {
    match public_key_encoding {
        PublicKeyEncoding::Base58 => PublicKeyField::Base58 {
            public_key_base58: key.base58(),
        },
        PublicKeyEncoding::Multibase => PublicKeyField::Multibase {
            public_key_multibase: key.fingerprint(),
        },
    }
}

fn nth_key_did_url_reference(n: usize) -> Result<DidUrl, DidPeerError> {
    DidUrl::from_fragment(format!("key-{n}")).map_err(Into::into)
}

#[cfg(test)]
mod tests {
    use did_doc::schema::verification_method::{
        PublicKeyField, VerificationMethod, VerificationMethodType,
    };
    use did_parser_nom::Did;
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
        VerificationMethod::builder()
            .id(did().into())
            .controller(did())
            .verification_method_type(VerificationMethodType::X25519KeyAgreementKey2020)
            .public_key(PublicKeyField::Multibase {
                public_key_multibase: key_0().fingerprint(),
            })
            .build()
    }

    fn verification_method_1() -> VerificationMethod {
        VerificationMethod::builder()
            .id(did().into())
            .controller(did())
            .verification_method_type(VerificationMethodType::Ed25519VerificationKey2020)
            .public_key(PublicKeyField::Multibase {
                public_key_multibase: key_1().fingerprint(),
            })
            .build()
    }

    fn verification_method_2() -> VerificationMethod {
        VerificationMethod::builder()
            .id(did().into())
            .controller(did())
            .verification_method_type(VerificationMethodType::Ed25519VerificationKey2020)
            .public_key(PublicKeyField::Multibase {
                public_key_multibase: key_2().fingerprint(),
            })
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
                &mut 0,
            )
            .unwrap();
            assert_eq!(vms.len(), 1);
            let vm = &vms[0];
            assert!(matches!(
                vm.public_key_field(),
                PublicKeyField::Multibase { .. }
            ));
            assert_eq!(vm.public_key_field().key_decoded().unwrap(), key.key());
        }

        // ... and base58 encoded keys are not
        fn test_get_verification_methods_by_key_base58(key: &Key) {
            let vms = verification_method::get_verification_methods_by_key(
                key,
                &did(),
                PublicKeyEncoding::Base58,
                &mut 0,
            )
            .unwrap();
            assert_eq!(vms.len(), 1);
            let vm = &vms[0];
            assert!(matches!(
                vm.public_key_field(),
                PublicKeyField::Base58 { .. }
            ));
            assert_eq!(vm.public_key_field().key_decoded().unwrap(), key.key());
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
