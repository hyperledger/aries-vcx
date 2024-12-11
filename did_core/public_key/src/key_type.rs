use std::fmt::Display;

use serde::{Deserialize, Serialize};

use crate::error::PublicKeyError;

#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub enum KeyType {
    Ed25519,
    Bls12381g1g2,
    Bls12381g1,
    Bls12381g2,
    X25519,
    P256,
    P384,
    P521,
}

impl KeyType {
    const C_BLS12381G1: u64 = 234;
    const C_BLS12381G2: u64 = 235;
    const C_X25519: u64 = 236;
    const C_ED25519: u64 = 237;
    const C_BLS12381G1G2: u64 = 238;
    const C_P256: u64 = 4608;
    const C_P384: u64 = 4609;
    const C_P521: u64 = 4610;
}

// https://github.com/multiformats/multicodec/blob/master/table.csv
impl From<&KeyType> for u64 {
    fn from(key_type: &KeyType) -> Self {
        match key_type {
            KeyType::Bls12381g1 => KeyType::C_BLS12381G1,
            KeyType::Bls12381g2 => KeyType::C_BLS12381G2,
            KeyType::X25519 => KeyType::C_X25519,
            KeyType::Ed25519 => KeyType::C_ED25519,
            KeyType::Bls12381g1g2 => KeyType::C_BLS12381G1G2,
            KeyType::P256 => KeyType::C_P256,
            KeyType::P384 => KeyType::C_P384,
            KeyType::P521 => KeyType::C_P521,
        }
    }
}

impl TryFrom<u64> for KeyType {
    type Error = PublicKeyError;

    fn try_from(value: u64) -> Result<Self, Self::Error> {
        match value {
            KeyType::C_BLS12381G1 => Ok(KeyType::Bls12381g1),
            KeyType::C_BLS12381G2 => Ok(KeyType::Bls12381g2),
            KeyType::C_X25519 => Ok(KeyType::X25519),
            KeyType::C_ED25519 => Ok(KeyType::Ed25519),
            KeyType::C_BLS12381G1G2 => Ok(KeyType::Bls12381g1g2),
            KeyType::C_P256 => Ok(KeyType::P256),
            KeyType::C_P384 => Ok(KeyType::P384),
            KeyType::C_P521 => Ok(KeyType::P521),
            p => Err(PublicKeyError::UnsupportedMulticodecDescriptor(p)),
        }
    }
}

impl Display for KeyType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            KeyType::Bls12381g1 => write!(f, "Bls12381g1"),
            KeyType::Bls12381g2 => write!(f, "Bls12381g2"),
            KeyType::X25519 => write!(f, "X25519"),
            KeyType::Ed25519 => write!(f, "Ed25519"),
            KeyType::Bls12381g1g2 => write!(f, "Bls12381g1g2"),
            KeyType::P256 => write!(f, "P256"),
            KeyType::P384 => write!(f, "P384"),
            KeyType::P521 => write!(f, "P521"),
        }
    }
}
