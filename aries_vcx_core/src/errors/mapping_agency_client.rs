use agency_client::errors::error::{AgencyClientError, AgencyClientErrorKind};

use crate::errors::error::{AriesVcxCoreError, AriesVcxCoreErrorKind};

impl From<AgencyClientError> for AriesVcxCoreError {
    fn from(agency_err: AgencyClientError) -> AriesVcxCoreError {
        let vcx_error_kind: AriesVcxCoreErrorKind = agency_err.kind().into();
        AriesVcxCoreError::from_msg(vcx_error_kind, agency_err.to_string())
    }
}

impl From<AgencyClientErrorKind> for AriesVcxCoreErrorKind {
    fn from(agency_err: AgencyClientErrorKind) -> AriesVcxCoreErrorKind {
        match agency_err {
            AgencyClientErrorKind::InvalidState => AriesVcxCoreErrorKind::InvalidState,
            AgencyClientErrorKind::InvalidConfiguration => {
                AriesVcxCoreErrorKind::InvalidConfiguration
            }
            AgencyClientErrorKind::InvalidJson => AriesVcxCoreErrorKind::InvalidJson,
            AgencyClientErrorKind::InvalidOption => AriesVcxCoreErrorKind::InvalidOption,
            AgencyClientErrorKind::InvalidMessagePack => AriesVcxCoreErrorKind::InvalidMessagePack,
            AgencyClientErrorKind::IOError => AriesVcxCoreErrorKind::IOError,
            AgencyClientErrorKind::PostMessageFailed => AriesVcxCoreErrorKind::PostMessageFailed,
            AgencyClientErrorKind::InvalidWalletHandle => {
                AriesVcxCoreErrorKind::InvalidWalletHandle
            }
            AgencyClientErrorKind::UnknownError => AriesVcxCoreErrorKind::UnknownError,
            AgencyClientErrorKind::InvalidDid => AriesVcxCoreErrorKind::InvalidDid,
            AgencyClientErrorKind::InvalidVerkey => AriesVcxCoreErrorKind::InvalidVerkey,
            AgencyClientErrorKind::InvalidUrl => AriesVcxCoreErrorKind::InvalidUrl,
            AgencyClientErrorKind::SerializationError => AriesVcxCoreErrorKind::SerializationError,
            AgencyClientErrorKind::NotBase58 => AriesVcxCoreErrorKind::NotBase58,
            AgencyClientErrorKind::InvalidHttpResponse => {
                AriesVcxCoreErrorKind::InvalidHttpResponse
            }
        }
    }
}
