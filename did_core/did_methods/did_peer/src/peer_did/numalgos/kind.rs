use std::fmt::Display;

use crate::{
    error::DidPeerError,
    peer_did::numalgos::{
        numalgo0::Numalgo0, numalgo1::Numalgo1, numalgo2::Numalgo2, numalgo3::Numalgo3,
        numalgo4::Numalgo4, Numalgo,
    },
};

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum NumalgoKind {
    InceptionKeyWithoutDoc(Numalgo0),
    GenesisDoc(Numalgo1),
    MultipleInceptionKeys(Numalgo2),
    DidShortening(Numalgo3),
    DidPeer4(Numalgo4),
}

impl NumalgoKind {
    pub fn to_char(&self) -> char {
        match self {
            NumalgoKind::InceptionKeyWithoutDoc(_) => Numalgo0::NUMALGO_CHAR,
            NumalgoKind::GenesisDoc(_) => Numalgo1::NUMALGO_CHAR,
            NumalgoKind::MultipleInceptionKeys(_) => Numalgo2::NUMALGO_CHAR,
            NumalgoKind::DidShortening(_) => Numalgo3::NUMALGO_CHAR,
            NumalgoKind::DidPeer4(_) => Numalgo4::NUMALGO_CHAR,
        }
    }
}

impl Display for NumalgoKind {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.to_char().fmt(f)
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
            Numalgo4::NUMALGO_CHAR => Ok(NumalgoKind::DidPeer4(Numalgo4)),
            c => Err(DidPeerError::InvalidNumalgoCharacter(c)),
        }
    }
}
