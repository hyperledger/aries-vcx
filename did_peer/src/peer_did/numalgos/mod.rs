pub mod numalgo0;
pub mod numalgo1;
pub mod numalgo2;
pub mod numalgo3;

pub(super) mod traits;

use std::fmt::Display;

use numalgo0::Numalgo0;
use numalgo1::Numalgo1;
use numalgo2::Numalgo2;
use numalgo3::Numalgo3;

use self::traits::Numalgo;
use crate::error::DidPeerError;

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum NumalgoKind {
    InceptionKeyWithoutDoc(Numalgo0),
    GenesisDoc(Numalgo1),
    MultipleInceptionKeys(Numalgo2),
    DidShortening(Numalgo3),
}

impl Display for NumalgoKind {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            NumalgoKind::InceptionKeyWithoutDoc(_) => Numalgo0::NUMALGO_CHAR.fmt(f),
            NumalgoKind::GenesisDoc(_) => Numalgo1::NUMALGO_CHAR.fmt(f),
            NumalgoKind::MultipleInceptionKeys(_) => Numalgo2::NUMALGO_CHAR.fmt(f),
            NumalgoKind::DidShortening(_) => Numalgo3::NUMALGO_CHAR.fmt(f),
        }
    }
}

impl TryFrom<char> for NumalgoKind {
    type Error = DidPeerError;

    fn try_from(value: char) -> Result<Self, Self::Error> {
        match value {
            Numalgo0::NUMALGO_CHAR => Ok(NumalgoKind::InceptionKeyWithoutDoc(Numalgo0)),
            Numalgo1::NUMALGO_CHAR => Ok(NumalgoKind::GenesisDoc(Numalgo1)),
            Numalgo2::NUMALGO_CHAR => Ok(NumalgoKind::MultipleInceptionKeys(Numalgo2)),
            Numalgo3::NUMALGO_CHAR => Ok(NumalgoKind::DidShortening(Numalgo3)),
            c => Err(DidPeerError::InvalidNumalgoCharacter(c)),
        }
    }
}
