use indy_ledger_response_parser::error::LedgerResponseParserError;

use super::error::VcxLedgerError;

impl From<LedgerResponseParserError> for VcxLedgerError {
    fn from(err: LedgerResponseParserError) -> Self {
        match err {
            LedgerResponseParserError::JsonError(err) => VcxLedgerError::InvalidJson(err),
            LedgerResponseParserError::LedgerItemNotFound(_) => VcxLedgerError::LedgerItemNotFound,
            LedgerResponseParserError::InvalidTransaction(_) => {
                VcxLedgerError::InvalidLedgerResponse(err.to_string())
            }
        }
    }
}
