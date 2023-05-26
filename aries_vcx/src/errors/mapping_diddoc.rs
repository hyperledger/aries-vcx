use crate::errors::error::{AriesVcxError, AriesVcxErrorKind};
use did_doc::error::DidDocumentBuilderError;
use diddoc_legacy::errors::error::{DiddocError, DiddocErrorKind};

impl From<DiddocError> for AriesVcxError {
    fn from(msg_err: DiddocError) -> AriesVcxError {
        let vcx_error_kind: AriesVcxErrorKind = msg_err.kind().into();
        AriesVcxError::from_msg(vcx_error_kind, msg_err.to_string())
    }
}

impl From<DiddocErrorKind> for AriesVcxErrorKind {
    fn from(msg_err: DiddocErrorKind) -> AriesVcxErrorKind {
        match msg_err {
            DiddocErrorKind::InvalidState => AriesVcxErrorKind::InvalidState,
            DiddocErrorKind::InvalidJson => AriesVcxErrorKind::InvalidJson,
            DiddocErrorKind::InvalidDid => AriesVcxErrorKind::InvalidDid,
            DiddocErrorKind::InvalidVerkey => AriesVcxErrorKind::InvalidVerkey,
            DiddocErrorKind::InvalidUrl => AriesVcxErrorKind::InvalidUrl,
            DiddocErrorKind::NotBase58 => AriesVcxErrorKind::NotBase58,
            DiddocErrorKind::SerializationError => AriesVcxErrorKind::SerializationError,
            DiddocErrorKind::ConversionError => AriesVcxErrorKind::DidDocumentError,
        }
    }
}

impl From<DidDocumentBuilderError> for AriesVcxError {
    fn from(msg_err: DidDocumentBuilderError) -> AriesVcxError {
        AriesVcxError::from_msg(AriesVcxErrorKind::DidDocumentError, msg_err.to_string())
    }
}
