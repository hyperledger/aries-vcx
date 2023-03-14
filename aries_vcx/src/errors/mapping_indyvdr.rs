use indy_vdr::{
    common::error::{VdrError, VdrErrorKind},
    utils::ValidationError,
};

use crate::errors::error::{AriesVcxError, AriesVcxErrorKind};

impl From<VdrError> for AriesVcxError {
    fn from(err: VdrError) -> Self {
        match err.kind() {
            VdrErrorKind::Config => AriesVcxError::from_msg(AriesVcxErrorKind::InvalidConfiguration, err),
            VdrErrorKind::Connection => AriesVcxError::from_msg(AriesVcxErrorKind::PoolLedgerConnect, err),
            VdrErrorKind::FileSystem(_) => AriesVcxError::from_msg(AriesVcxErrorKind::IOError, err),
            VdrErrorKind::Input => AriesVcxError::from_msg(AriesVcxErrorKind::InvalidInput, err),
            VdrErrorKind::Resource => AriesVcxError::from_msg(AriesVcxErrorKind::UnknownError, err),
            VdrErrorKind::Unavailable => AriesVcxError::from_msg(AriesVcxErrorKind::UnknownError, err),
            VdrErrorKind::Unexpected => AriesVcxError::from_msg(AriesVcxErrorKind::UnknownError, err),
            VdrErrorKind::Incompatible => AriesVcxError::from_msg(AriesVcxErrorKind::UnknownError, err),
            VdrErrorKind::PoolNoConsensus => AriesVcxError::from_msg(AriesVcxErrorKind::UnknownError, err),
            VdrErrorKind::PoolRequestFailed(_) => AriesVcxError::from_msg(AriesVcxErrorKind::PoolLedgerConnect, err),
            VdrErrorKind::PoolTimeout => AriesVcxError::from_msg(AriesVcxErrorKind::UnknownError, err),
        }
    }
}

impl From<ValidationError> for AriesVcxError {
    fn from(err: ValidationError) -> Self {
        AriesVcxError::from_msg(AriesVcxErrorKind::InvalidInput, err)
    }
}
