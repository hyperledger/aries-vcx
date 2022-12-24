use indy_credx::ursa::errors::{UrsaCryptoError, UrsaCryptoErrorKind};
use crate::errors::error::{AriesVcxError, AriesVcxErrorKind};

impl From<UrsaCryptoError> for AriesVcxError {
    fn from(err: UrsaCryptoError) -> Self {
        match err.kind() {
            UrsaCryptoErrorKind::InvalidState => {
                AriesVcxError::from_msg(AriesVcxErrorKind::InvalidState, err)
            }
            UrsaCryptoErrorKind::InvalidStructure => {
                AriesVcxError::from_msg(AriesVcxErrorKind::InvalidInput, err)
            }
            UrsaCryptoErrorKind::InvalidParam(_) => {
                AriesVcxError::from_msg(AriesVcxErrorKind::InvalidInput, err)
            }
            UrsaCryptoErrorKind::IOError => AriesVcxError::from_msg(AriesVcxErrorKind::IOError, err),
            UrsaCryptoErrorKind::ProofRejected => {
                AriesVcxError::from_msg(AriesVcxErrorKind::InvalidState, err)
            }
            UrsaCryptoErrorKind::RevocationAccumulatorIsFull => {
                AriesVcxError::from_msg(AriesVcxErrorKind::InvalidState, err)
            }
            UrsaCryptoErrorKind::InvalidRevocationAccumulatorIndex => {
                AriesVcxError::from_msg(AriesVcxErrorKind::InvalidInput, err)
            }
            UrsaCryptoErrorKind::CredentialRevoked => {
                AriesVcxError::from_msg(AriesVcxErrorKind::InvalidState, err)
            }
        }
    }
}
