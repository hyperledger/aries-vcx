use shared_vcx::errors::error::{SharedVcxError, SharedVcxErrorKind};

use crate::errors::error::{AgencyClientError, AgencyClientErrorKind};

impl From<SharedVcxErrorKind> for AgencyClientErrorKind {
    fn from(error: SharedVcxErrorKind) -> Self {
        match error{
            SharedVcxErrorKind::InvalidDid => AgencyClientErrorKind::InvalidDid,
            SharedVcxErrorKind::InvalidVerkey => AgencyClientErrorKind::InvalidVerkey,
            SharedVcxErrorKind::NotBase58 => AgencyClientErrorKind::NotBase58
        }
    }
}

impl From<SharedVcxError> for AgencyClientError {
    fn from(error: SharedVcxError) -> Self {
        let kind: AgencyClientErrorKind = error.kind().into();
        AgencyClientError::from_msg(kind, error.to_string())
    }
}
