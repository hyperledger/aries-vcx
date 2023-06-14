use crate::error::DidPeerError;

#[derive(Debug, Clone, PartialEq)]
pub enum SupportedKeyType {
    Ed25519,
    Bls12381g1g2,
    Bls12381g1,
    Bls12381g2,
    X25519,
    P256,
    P384,
    P521,
}

impl From<&SupportedKeyType> for u64 {
    fn from(key_type: &SupportedKeyType) -> Self {
        match key_type {
            SupportedKeyType::Ed25519 => 237,
            SupportedKeyType::Bls12381g1 => 234,
            SupportedKeyType::Bls12381g2 => 235,
            SupportedKeyType::X25519 => 236,
            SupportedKeyType::P256 => 4608,
            SupportedKeyType::P384 => 4609,
            SupportedKeyType::P521 => 4610,
            SupportedKeyType::Bls12381g1g2 => 238,
        }
    }
}

impl TryFrom<u64> for SupportedKeyType {
    type Error = DidPeerError;

    fn try_from(value: u64) -> Result<Self, Self::Error> {
        match value {
            234 => Ok(SupportedKeyType::Bls12381g1),
            235 => Ok(SupportedKeyType::Bls12381g2),
            236 => Ok(SupportedKeyType::X25519),
            237 => Ok(SupportedKeyType::Ed25519),
            238 => Ok(SupportedKeyType::Bls12381g1g2),
            4608 => Ok(SupportedKeyType::P256),
            4609 => Ok(SupportedKeyType::P384),
            4610 => Ok(SupportedKeyType::P521),
            p @ _ => Err(DidPeerError::UnsupportedMulticodecDescriptor(p)),
        }
    }
}
