use std::fmt::Display;

use public_key::KeyType;
use serde::{Deserialize, Serialize};

use crate::error::DidDocumentBuilderError;

#[derive(Serialize, Deserialize, Clone, Copy, Debug, PartialEq)]
pub enum VerificationMethodType {
    JsonWebKey2020,
    EcdsaSecp256k1VerificationKey2019,
    Ed25519VerificationKey2018,
    Ed25519VerificationKey2020,
    Bls12381G1Key2020,
    Bls12381G2Key2020,
    PgpVerificationKey2021,
    RsaVerificationKey2018,
    X25519KeyAgreementKey2019,
    X25519KeyAgreementKey2020,
    EcdsaSecp256k1RecoveryMethod2020,
    /// https://www.w3.org/TR/vc-data-integrity/#multikey
    Multikey,
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
            VerificationMethodType::RsaVerificationKey2018 => write!(f, "RsaVerificationKey2018"),
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
