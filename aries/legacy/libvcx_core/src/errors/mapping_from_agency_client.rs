use aries_vcx::agency_client::errors::error::{AgencyClientError, AgencyClientErrorKind};

use crate::errors::error::{LibvcxError, LibvcxErrorKind};

impl From<AgencyClientError> for LibvcxError {
    fn from(agency_err: AgencyClientError) -> LibvcxError {
        let vcx_error_kind: LibvcxErrorKind = agency_err.kind().into();
        LibvcxError::from_msg(vcx_error_kind, agency_err.to_string())
    }
}

impl From<AgencyClientErrorKind> for LibvcxErrorKind {
    fn from(agency_err: AgencyClientErrorKind) -> LibvcxErrorKind {
        match agency_err {
            AgencyClientErrorKind::InvalidState => LibvcxErrorKind::InvalidState,
            AgencyClientErrorKind::InvalidConfiguration => LibvcxErrorKind::InvalidConfiguration,
            AgencyClientErrorKind::InvalidJson => LibvcxErrorKind::InvalidJson,
            AgencyClientErrorKind::InvalidOption => LibvcxErrorKind::InvalidOption,
            AgencyClientErrorKind::InvalidMessagePack => LibvcxErrorKind::InvalidMessagePack,
            AgencyClientErrorKind::IOError => LibvcxErrorKind::IOError,
            AgencyClientErrorKind::PostMessageFailed => LibvcxErrorKind::PostMessageFailed,
            AgencyClientErrorKind::InvalidWalletHandle => LibvcxErrorKind::InvalidWalletHandle,
            AgencyClientErrorKind::UnknownError => LibvcxErrorKind::UnknownError,
            AgencyClientErrorKind::InvalidDid => LibvcxErrorKind::InvalidDid,
            AgencyClientErrorKind::InvalidVerkey => LibvcxErrorKind::InvalidVerkey,
            AgencyClientErrorKind::InvalidUrl => LibvcxErrorKind::InvalidUrl,
            AgencyClientErrorKind::SerializationError => LibvcxErrorKind::SerializationError,
            AgencyClientErrorKind::NotBase58 => LibvcxErrorKind::NotBase58,
            AgencyClientErrorKind::InvalidHttpResponse => LibvcxErrorKind::InvalidHttpResponse,
        }
    }
}
