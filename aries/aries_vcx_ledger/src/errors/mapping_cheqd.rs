use did_cheqd::error::{parsing::ParsingErrorSource, DidCheqdError};

use super::error::VcxLedgerError;

impl From<DidCheqdError> for VcxLedgerError {
    fn from(value: DidCheqdError) -> Self {
        match value {
            DidCheqdError::MethodNotSupported(_) => VcxLedgerError::InvalidInput(value.to_string()),
            DidCheqdError::NetworkNotSupported(_) => {
                VcxLedgerError::InvalidInput(value.to_string())
            }
            DidCheqdError::BadConfiguration(_) => VcxLedgerError::InvalidInput(value.to_string()),
            DidCheqdError::TransportError(_) => {
                VcxLedgerError::InvalidLedgerResponse(value.to_string())
            }
            DidCheqdError::NonSuccessResponse(_) => {
                VcxLedgerError::InvalidLedgerResponse(value.to_string())
            }
            DidCheqdError::InvalidResponse(_) => {
                VcxLedgerError::InvalidLedgerResponse(value.to_string())
            }
            DidCheqdError::InvalidDidDocument(_) => VcxLedgerError::InvalidInput(value.to_string()),
            DidCheqdError::InvalidDidUrl(_) => VcxLedgerError::InvalidInput(value.to_string()),
            DidCheqdError::ParsingError(ParsingErrorSource::DidDocumentParsingError(e)) => {
                VcxLedgerError::ParseError(e)
            }
            DidCheqdError::Other(_) => VcxLedgerError::UnknownError(value.to_string()),
            _ => VcxLedgerError::UnknownError(value.to_string()),
        }
    }
}
