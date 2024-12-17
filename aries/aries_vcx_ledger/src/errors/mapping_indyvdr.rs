use indy_vdr::{
    common::error::{VdrError, VdrErrorKind},
    utils::ValidationError,
};

use crate::errors::error::VcxLedgerError;

impl From<VdrError> for VcxLedgerError {
    fn from(err: VdrError) -> Self {
        match err.kind() {
            VdrErrorKind::Config => Self::InvalidConfiguration(err),
            VdrErrorKind::Connection => Self::PoolLedgerConnect(err),
            VdrErrorKind::FileSystem => Self::IOError(err),
            VdrErrorKind::Input => Self::InvalidInput(err.to_string()),
            VdrErrorKind::Resource
            | VdrErrorKind::Unavailable
            | VdrErrorKind::Unexpected
            | VdrErrorKind::Incompatible
            | VdrErrorKind::PoolNoConsensus
            | VdrErrorKind::Resolver
            | VdrErrorKind::PoolTimeout => Self::UnknownError(err.to_string()),
            VdrErrorKind::PoolRequestFailed(_) => Self::InvalidLedgerResponse(err.to_string()),
        }
    }
}

impl From<ValidationError> for VcxLedgerError {
    fn from(err: ValidationError) -> Self {
        VcxLedgerError::InvalidInput(err.to_string())
    }
}
