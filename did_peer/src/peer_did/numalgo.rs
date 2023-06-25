use std::fmt::Display;

use crate::error::DidPeerError;

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum Numalgo {
    InceptionKeyWithoutDoc,
    GenesisDoc,
    MultipleInceptionKeys,
    DidShortening,
}

impl Display for Numalgo {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Numalgo::InceptionKeyWithoutDoc => write!(f, "0"),
            Numalgo::GenesisDoc => write!(f, "1"),
            Numalgo::MultipleInceptionKeys => write!(f, "2"),
            Numalgo::DidShortening => write!(f, "3"),
        }
    }
}

impl TryFrom<char> for Numalgo {
    type Error = DidPeerError;

    fn try_from(value: char) -> Result<Self, Self::Error> {
        match value {
            '0' => Ok(Numalgo::InceptionKeyWithoutDoc),
            '1' => Ok(Numalgo::GenesisDoc),
            '2' => Ok(Numalgo::MultipleInceptionKeys),
            '3' => Ok(Numalgo::DidShortening),
            c @ _ => Err(DidPeerError::InvalidNumalgoCharacter(c)),
        }
    }
}
