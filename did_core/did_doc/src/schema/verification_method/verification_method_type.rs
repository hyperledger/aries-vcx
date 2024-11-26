use std::fmt::Display;

use public_key::KeyType;
use serde::{Deserialize, Serialize};

use crate::{error::DidDocumentBuilderError, schema::contexts};

#[derive(Serialize, Deserialize, Clone, Copy, Debug, PartialEq)]
pub enum VerificationMethodType {
    /// https://w3id.org/security/suites/jws-2020/v1
    JsonWebKey2020,
    /// https://w3id.org/security/suites/secp256k1-2019/v1
    EcdsaSecp256k1VerificationKey2019,
    /// https://w3id.org/security/suites/ed25519-2018/v1
    Ed25519VerificationKey2018,
    /// https://w3id.org/security/suites/ed25519-2020/v1
    Ed25519VerificationKey2020,
    /// https://w3id.org/security/bbs/v1
    Bls12381G1Key2020,
    /// https://w3id.org/security/bbs/v1
    Bls12381G2Key2020,
    /// https://w3id.org/pgp/v1
    PgpVerificationKey2021,
    /// https://w3id.org/security/suites/x25519-2019/v1
    X25519KeyAgreementKey2019,
    /// https://w3id.org/security/suites/x25519-2020/v1
    X25519KeyAgreementKey2020,
    /// https://identity.foundation/EcdsaSecp256k1RecoverySignature2020/lds-ecdsa-secp256k1-recovery2020-0.0.jsonld
    EcdsaSecp256k1RecoveryMethod2020,
    /// https://www.w3.org/TR/vc-data-integrity/#multikey
    /// https://w3id.org/security/multikey/v1
    Multikey,
}

impl VerificationMethodType {
    /// Return the JSON-LD context URL for which this type comes from
    pub fn context_for_type(&self) -> &str {
        match self {
            VerificationMethodType::JsonWebKey2020 => contexts::W3C_SUITE_JWS_2020,
            VerificationMethodType::EcdsaSecp256k1VerificationKey2019 => {
                contexts::W3C_SUITE_SECP256K1_2019
            }
            VerificationMethodType::Ed25519VerificationKey2018 => contexts::W3C_SUITE_ED25519_2018,
            VerificationMethodType::Ed25519VerificationKey2020 => contexts::W3C_SUITE_ED25519_2020,
            VerificationMethodType::Bls12381G1Key2020 => contexts::W3C_BBS_V1,
            VerificationMethodType::Bls12381G2Key2020 => contexts::W3C_BBS_V1,
            VerificationMethodType::PgpVerificationKey2021 => contexts::W3C_PGP_V1,
            VerificationMethodType::X25519KeyAgreementKey2019 => contexts::W3C_SUITE_X25519_2019,
            VerificationMethodType::X25519KeyAgreementKey2020 => contexts::W3C_SUITE_X25519_2020,
            VerificationMethodType::EcdsaSecp256k1RecoveryMethod2020 => {
                contexts::W3C_SUITE_SECP259K1_RECOVERY_2020
            }
            VerificationMethodType::Multikey => contexts::W3C_MULTIKEY_V1,
        }
    }
}

impl Display for VerificationMethodType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            VerificationMethodType::JsonWebKey2020 => write!(f, "JsonWebKey2020"),
            VerificationMethodType::EcdsaSecp256k1VerificationKey2019 => {
                write!(f, "EcdsaSecp256k1VerificationKey2019")
            }
            VerificationMethodType::Ed25519VerificationKey2018 => {
                write!(f, "Ed25519VerificationKey2018")
            }
            VerificationMethodType::Ed25519VerificationKey2020 => {
                write!(f, "Ed25519VerificationKey2020")
            }
            VerificationMethodType::Bls12381G1Key2020 => write!(f, "Bls12381G1Key2020"),
            VerificationMethodType::Bls12381G2Key2020 => write!(f, "Bls12381G2Key2020"),
            VerificationMethodType::PgpVerificationKey2021 => write!(f, "PgpVerificationKey2021"),
            VerificationMethodType::X25519KeyAgreementKey2019 => {
                write!(f, "X25519KeyAgreementKey2019")
            }
            VerificationMethodType::X25519KeyAgreementKey2020 => {
                write!(f, "X25519KeyAgreementKey2020")
            }
            VerificationMethodType::EcdsaSecp256k1RecoveryMethod2020 => {
                write!(f, "EcdsaSecp256k1RecoveryMethod2020")
            }
            VerificationMethodType::Multikey => {
                write!(f, "Multikey")
            }
        }
    }
}

impl TryFrom<VerificationMethodType> for KeyType {
    type Error = DidDocumentBuilderError;

    fn try_from(value: VerificationMethodType) -> Result<Self, Self::Error> {
        match value {
            VerificationMethodType::Ed25519VerificationKey2018
            | VerificationMethodType::Ed25519VerificationKey2020 => Ok(KeyType::Ed25519),
            VerificationMethodType::Bls12381G1Key2020 => Ok(KeyType::Bls12381g1),
            VerificationMethodType::Bls12381G2Key2020 => Ok(KeyType::Bls12381g2),
            VerificationMethodType::X25519KeyAgreementKey2019
            | VerificationMethodType::X25519KeyAgreementKey2020 => Ok(KeyType::X25519),
            // The verification method type does not map directly to a key type.
            // This may occur when the VM type is a multikey (JsonWebKey, Multikey, etc)
            _ => Err(DidDocumentBuilderError::UnsupportedVerificationMethodType(
                value,
            )),
        }
    }
}
