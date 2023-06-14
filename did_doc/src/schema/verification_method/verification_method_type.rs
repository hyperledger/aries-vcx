use std::fmt::Display;

use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
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
        }
    }
}
