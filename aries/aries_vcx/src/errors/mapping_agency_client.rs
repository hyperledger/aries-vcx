use agency_client::errors::error::{AgencyClientError, AgencyClientErrorKind};

use crate::errors::error::{AriesVcxError, AriesVcxErrorKind};

impl From<AgencyClientError> for AriesVcxError {
    fn from(agency_err: AgencyClientError) -> AriesVcxError {
        let vcx_error_kind: AriesVcxErrorKind = agency_err.kind().into();
        AriesVcxError::from_msg(vcx_error_kind, agency_err.to_string())
    }
}

impl From<AgencyClientErrorKind> for AriesVcxErrorKind {
    fn from(agency_err: AgencyClientErrorKind) -> AriesVcxErrorKind {
        match agency_err {
            AgencyClientErrorKind::InvalidState => AriesVcxErrorKind::InvalidState,
            AgencyClientErrorKind::InvalidConfiguration => AriesVcxErrorKind::InvalidConfiguration,
            AgencyClientErrorKind::InvalidJson => AriesVcxErrorKind::InvalidJson,
            AgencyClientErrorKind::InvalidOption => AriesVcxErrorKind::InvalidOption,
            AgencyClientErrorKind::InvalidMessagePack => AriesVcxErrorKind::InvalidMessagePack,
            AgencyClientErrorKind::IOError => AriesVcxErrorKind::IOError,
            AgencyClientErrorKind::PostMessageFailed => AriesVcxErrorKind::PostMessageFailed,
            AgencyClientErrorKind::InvalidWalletHandle => AriesVcxErrorKind::InvalidWalletHandle,
            AgencyClientErrorKind::UnknownError => AriesVcxErrorKind::UnknownError,
            AgencyClientErrorKind::InvalidDid => AriesVcxErrorKind::InvalidDid,
            AgencyClientErrorKind::InvalidVerkey => AriesVcxErrorKind::InvalidVerkey,
            AgencyClientErrorKind::InvalidUrl => AriesVcxErrorKind::InvalidUrl,
            AgencyClientErrorKind::SerializationError => AriesVcxErrorKind::SerializationError,
            AgencyClientErrorKind::NotBase58 => AriesVcxErrorKind::NotBase58,
            AgencyClientErrorKind::InvalidHttpResponse => AriesVcxErrorKind::InvalidHttpResponse,
        }
    }
}
