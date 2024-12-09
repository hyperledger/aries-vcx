use aries_vcx_ledger::errors::error::VcxLedgerError;

use super::error::{AriesVcxError, AriesVcxErrorKind};

impl From<VcxLedgerError> for AriesVcxError {
    fn from(value: VcxLedgerError) -> Self {
        match value {
            VcxLedgerError::LedgerItemNotFound => {
                Self::from_msg(AriesVcxErrorKind::LedgerItemNotFound, value)
            }
            VcxLedgerError::InvalidLedgerResponse(_) => {
                Self::from_msg(AriesVcxErrorKind::InvalidLedgerResponse, value)
            }
            VcxLedgerError::DuplicationSchema => {
                Self::from_msg(AriesVcxErrorKind::DuplicationSchema, value)
            }
            VcxLedgerError::InvalidJson(_) => Self::from_msg(AriesVcxErrorKind::InvalidJson, value),
            VcxLedgerError::WalletError(_) => Self::from_msg(AriesVcxErrorKind::WalletError, value),
            VcxLedgerError::InvalidState(_) => {
                Self::from_msg(AriesVcxErrorKind::InvalidState, value)
            }
            VcxLedgerError::InvalidOption(_) => {
                Self::from_msg(AriesVcxErrorKind::InvalidOption, value)
            }
            VcxLedgerError::ParseError(_) => Self::from_msg(AriesVcxErrorKind::ParsingError, value),
            VcxLedgerError::UnimplementedFeature(_) => {
                Self::from_msg(AriesVcxErrorKind::UnimplementedFeature, value)
            }
            VcxLedgerError::InvalidConfiguration(_) => {
                Self::from_msg(AriesVcxErrorKind::InvalidConfiguration, value)
            }
            VcxLedgerError::PoolLedgerConnect(_) => {
                Self::from_msg(AriesVcxErrorKind::PoolLedgerConnect, value)
            }
            VcxLedgerError::IOError(_) => Self::from_msg(AriesVcxErrorKind::IOError, value),
            VcxLedgerError::InvalidInput(_)
            | VcxLedgerError::IndyVdrValidation(_)
            | VcxLedgerError::UnsupportedLedgerIdentifier(_) => {
                Self::from_msg(AriesVcxErrorKind::InvalidInput, value)
            }
            VcxLedgerError::UnknownError(_) => {
                Self::from_msg(AriesVcxErrorKind::UnknownError, value)
            }
        }
    }
}
