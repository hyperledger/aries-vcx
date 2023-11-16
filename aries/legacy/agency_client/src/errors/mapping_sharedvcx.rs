use shared::errors::validation::{ValidationError, ValidationErrorKind};

use crate::errors::error::{AgencyClientError, AgencyClientErrorKind};

impl From<ValidationErrorKind> for AgencyClientErrorKind {
    fn from(error: ValidationErrorKind) -> Self {
        match error {
            ValidationErrorKind::InvalidDid => AgencyClientErrorKind::InvalidDid,
            ValidationErrorKind::InvalidVerkey => AgencyClientErrorKind::InvalidVerkey,
            ValidationErrorKind::NotBase58 => AgencyClientErrorKind::NotBase58,
        }
    }
}

impl From<ValidationError> for AgencyClientError {
    fn from(error: ValidationError) -> Self {
        let kind: AgencyClientErrorKind = error.kind().into();
        AgencyClientError::from_msg(kind, error.to_string())
    }
}
