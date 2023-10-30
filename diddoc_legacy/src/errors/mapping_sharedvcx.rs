use shared_vcx::errors::validation::{ValidationError, ValidationErrorKind};

use crate::errors::error::{DiddocError, DiddocErrorKind};

impl From<ValidationErrorKind> for DiddocErrorKind {
    fn from(error: ValidationErrorKind) -> Self {
        match error {
            ValidationErrorKind::InvalidDid => DiddocErrorKind::InvalidDid,
            ValidationErrorKind::InvalidVerkey => DiddocErrorKind::InvalidVerkey,
            ValidationErrorKind::NotBase58 => DiddocErrorKind::NotBase58,
        }
    }
}

impl From<ValidationError> for DiddocError {
    fn from(error: ValidationError) -> Self {
        let kind: DiddocErrorKind = error.kind().into();
        DiddocError::from_msg(kind, error.to_string())
    }
}
