use crate::error::DidPeerError;

#[derive(Clone, Debug, PartialEq)]
pub enum Transform {
    BASE58BTC,
}

impl TryFrom<char> for Transform {
    type Error = DidPeerError;

    fn try_from(value: char) -> Result<Self, Self::Error> {
        match value {
            'z' => Ok(Transform::BASE58BTC),
            c @ _ => Err(DidPeerError::DidValidationError(format!(
                "Unsupported transform character: {}",
                c
            ))),
        }
    }
}
