use indy_ledger_response_parser::error::LedgerResponseParserError;

use super::error::{AriesVcxCoreError, AriesVcxCoreErrorKind};

impl From<LedgerResponseParserError> for AriesVcxCoreError {
    fn from(err: LedgerResponseParserError) -> Self {
        match &err {
            LedgerResponseParserError::JsonError(err) => {
                AriesVcxCoreError::from_msg(AriesVcxCoreErrorKind::InvalidJson, err.to_string())
            }
            LedgerResponseParserError::LedgerItemNotFound(_) => AriesVcxCoreError::from_msg(
                AriesVcxCoreErrorKind::LedgerItemNotFound,
                err.to_string(),
            ),
            LedgerResponseParserError::InvalidTransaction(_) => AriesVcxCoreError::from_msg(
                AriesVcxCoreErrorKind::InvalidLedgerResponse,
                err.to_string(),
            ),
        }
    }
}
