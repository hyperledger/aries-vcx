use agency_client::error::{AgencyClientError, AgencyClientErrorKind};
use crate::errors::error::{VcxError, VcxErrorKind};

impl From<AgencyClientError> for VcxError {
    fn from(agency_err: AgencyClientError) -> VcxError {
        let vcx_error_kind: VcxErrorKind = agency_err.kind().into();
        VcxError::from_msg(vcx_error_kind, agency_err.to_string())
    }
}

impl From<AgencyClientErrorKind> for VcxErrorKind {
    fn from(agency_err: AgencyClientErrorKind) -> VcxErrorKind {
        match agency_err {
            AgencyClientErrorKind::InvalidState => VcxErrorKind::InvalidState,
            AgencyClientErrorKind::InvalidConfiguration => VcxErrorKind::InvalidConfiguration,
            AgencyClientErrorKind::InvalidJson => VcxErrorKind::InvalidJson,
            AgencyClientErrorKind::InvalidOption => VcxErrorKind::InvalidOption,
            AgencyClientErrorKind::InvalidMessagePack => VcxErrorKind::InvalidMessagePack,
            AgencyClientErrorKind::IOError => VcxErrorKind::IOError,
            AgencyClientErrorKind::PostMessageFailed => VcxErrorKind::PostMessageFailed,
            AgencyClientErrorKind::InvalidWalletHandle => VcxErrorKind::InvalidWalletHandle,
            AgencyClientErrorKind::UnknownError => VcxErrorKind::UnknownError,
            AgencyClientErrorKind::InvalidDid => VcxErrorKind::InvalidDid,
            AgencyClientErrorKind::InvalidVerkey => VcxErrorKind::InvalidVerkey,
            AgencyClientErrorKind::InvalidUrl => VcxErrorKind::InvalidUrl,
            AgencyClientErrorKind::SerializationError => VcxErrorKind::SerializationError,
            AgencyClientErrorKind::NotBase58 => VcxErrorKind::NotBase58,
            AgencyClientErrorKind::InvalidHttpResponse => VcxErrorKind::InvalidHttpResponse,
        }
    }
}
