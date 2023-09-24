use std::fmt::Display;

use crate::error::DidPeerError;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ElementPurpose {
    Assertion,
    Encryption,
    Verification,
    CapabilityInvocation,
    CapabilityDelegation,
    Service,
}

impl Display for ElementPurpose {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let c: char = (*self).into();
        write!(f, "{}", c)
    }
}

impl TryFrom<char> for ElementPurpose {
    type Error = DidPeerError;

    fn try_from(c: char) -> Result<Self, Self::Error> {
        match c {
            'A' => Ok(ElementPurpose::Assertion),
            'E' => Ok(ElementPurpose::Encryption),
            'V' => Ok(ElementPurpose::Verification),
            'I' => Ok(ElementPurpose::CapabilityInvocation),
            'D' => Ok(ElementPurpose::CapabilityDelegation),
            'S' => Ok(ElementPurpose::Service),
            c => Err(DidPeerError::UnsupportedPurpose(c)),
        }
    }
}

impl From<ElementPurpose> for char {
    fn from(purpose: ElementPurpose) -> Self {
        match purpose {
            ElementPurpose::Assertion => 'A',
            ElementPurpose::Encryption => 'E',
            ElementPurpose::Verification => 'V',
            ElementPurpose::CapabilityInvocation => 'I',
            ElementPurpose::CapabilityDelegation => 'D',
            ElementPurpose::Service => 'S',
        }
    }
}
