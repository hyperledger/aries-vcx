use crate::errors::error::{AriesVcxCoreError, AriesVcxCoreErrorKind};
use indy_vdr::common::error::{VdrError, VdrErrorKind};
use indy_vdr::utils::ValidationError;

impl From<VdrError> for AriesVcxCoreError {
    fn from(err: VdrError) -> Self {
        match err.kind() {
            VdrErrorKind::Config => AriesVcxCoreError::from_msg(AriesVcxCoreErrorKind::InvalidConfiguration, err),
            VdrErrorKind::Connection => AriesVcxCoreError::from_msg(AriesVcxCoreErrorKind::PoolLedgerConnect, err),
            // todo: we are losing information about the err
            VdrErrorKind::FileSystem(_) => AriesVcxCoreError::from_msg(AriesVcxCoreErrorKind::IOError, err),
            VdrErrorKind::Input => AriesVcxCoreError::from_msg(AriesVcxCoreErrorKind::InvalidInput, err),
            VdrErrorKind::Resource => AriesVcxCoreError::from_msg(AriesVcxCoreErrorKind::UnknownError, err),
            VdrErrorKind::Unavailable => AriesVcxCoreError::from_msg(AriesVcxCoreErrorKind::UnknownError, err),
            VdrErrorKind::Unexpected => AriesVcxCoreError::from_msg(AriesVcxCoreErrorKind::UnknownError, err),
            VdrErrorKind::Incompatible => AriesVcxCoreError::from_msg(AriesVcxCoreErrorKind::UnknownError, err),
            VdrErrorKind::PoolNoConsensus => AriesVcxCoreError::from_msg(AriesVcxCoreErrorKind::UnknownError, err),
            // todo: we are losing information about the err
            VdrErrorKind::PoolRequestFailed(_) => {
                AriesVcxCoreError::from_msg(AriesVcxCoreErrorKind::InvalidLedgerResponse, err)
            }
            VdrErrorKind::Resolver => AriesVcxCoreError::from_msg(AriesVcxCoreErrorKind::UnknownError, err),
            VdrErrorKind::PoolTimeout => AriesVcxCoreError::from_msg(AriesVcxCoreErrorKind::UnknownError, err),
        }
    }
}

impl From<ValidationError> for AriesVcxCoreError {
    fn from(err: ValidationError) -> Self {
        AriesVcxCoreError::from_msg(AriesVcxCoreErrorKind::InvalidInput, err)
    }
}
