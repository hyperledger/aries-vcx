use aries_vcx_ledger::errors::error::VcxLedgerError;

use super::error::{AriesVcxError, AriesVcxErrorKind};

impl From<VcxLedgerError> for AriesVcxError {
    fn from(value: VcxLedgerError) -> Self {
        match value {
            VcxLedgerError::LedgerItemNotFound => {
                Self::from_msg(AriesVcxErrorKind::LedgerItemNotFound, value)
            }
            VcxLedgerError::InvalidLedgerResponse => {
                Self::from_msg(AriesVcxErrorKind::InvalidLedgerResponse, value)
            }
            VcxLedgerError::DuplicationSchema => {
                Self::from_msg(AriesVcxErrorKind::DuplicationSchema, value)
            }
            VcxLedgerError::InvalidJson(_) => Self::from_msg(AriesVcxErrorKind::InvalidJson, value),
            VcxLedgerError::VdrError(inner) => inner.into(),
            VcxLedgerError::WalletError(_) => Self::from_msg(AriesVcxErrorKind::WalletError, value),
            VcxLedgerError::InvalidState(_) => {
                Self::from_msg(AriesVcxErrorKind::InvalidState, value)
            }
            VcxLedgerError::InvalidOption(_) => {
                Self::from_msg(AriesVcxErrorKind::InvalidOption, value)
            }
            VcxLedgerError::IndyVdrValidation(inner) => inner.into(),
            VcxLedgerError::ParseError(_) => Self::from_msg(AriesVcxErrorKind::ParsingError, value),
            VcxLedgerError::UnimplementedFeature(_) => {
                Self::from_msg(AriesVcxErrorKind::UnimplementedFeature, value)
            }
        }
    }
}
