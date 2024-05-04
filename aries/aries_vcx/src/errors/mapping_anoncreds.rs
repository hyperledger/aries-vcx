use aries_vcx_anoncreds::errors::error::VcxAnoncredsError;

use super::error::{AriesVcxError, AriesVcxErrorKind};

impl From<VcxAnoncredsError> for AriesVcxError {
    fn from(err: VcxAnoncredsError) -> Self {
        match err {
            VcxAnoncredsError::InvalidJson(_) => {
                AriesVcxError::from_msg(AriesVcxErrorKind::InvalidJson, err)
            }
            VcxAnoncredsError::InvalidInput(_) => {
                AriesVcxError::from_msg(AriesVcxErrorKind::InvalidInput, err)
            }
            VcxAnoncredsError::InvalidState(_) => {
                AriesVcxError::from_msg(AriesVcxErrorKind::InvalidState, err)
            }
            VcxAnoncredsError::WalletError(inner) => inner.into(),
            VcxAnoncredsError::UrsaError(_) => {
                AriesVcxError::from_msg(AriesVcxErrorKind::UrsaError, err)
            }
            VcxAnoncredsError::IOError(_) => {
                AriesVcxError::from_msg(AriesVcxErrorKind::IOError, err)
            }
            VcxAnoncredsError::UnknownError(_) => {
                AriesVcxError::from_msg(AriesVcxErrorKind::UnknownError, err)
            }
            VcxAnoncredsError::ProofRejected(_) => {
                AriesVcxError::from_msg(AriesVcxErrorKind::ProofRejected, err)
            }
            VcxAnoncredsError::ActionNotSupported(_) => {
                AriesVcxError::from_msg(AriesVcxErrorKind::ActionNotSupported, err)
            }
            VcxAnoncredsError::InvalidProofRequest(_) => {
                AriesVcxError::from_msg(AriesVcxErrorKind::InvalidProofRequest, err)
            }
            VcxAnoncredsError::InvalidAttributesStructure(_) => {
                AriesVcxError::from_msg(AriesVcxErrorKind::InvalidAttributesStructure, err)
            }
            VcxAnoncredsError::InvalidOption(_) => {
                AriesVcxError::from_msg(AriesVcxErrorKind::InvalidOption, err)
            }
            VcxAnoncredsError::InvalidSchema(_) => {
                AriesVcxError::from_msg(AriesVcxErrorKind::InvalidSchema, err)
            }
            VcxAnoncredsError::DuplicationMasterSecret(_) => {
                AriesVcxError::from_msg(AriesVcxErrorKind::DuplicationMasterSecret, err)
            }
            VcxAnoncredsError::UnimplementedFeature(_) => {
                AriesVcxError::from_msg(AriesVcxErrorKind::UnimplementedFeature, err)
            }
        }
    }
}
