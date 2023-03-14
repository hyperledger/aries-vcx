use messages::diddoc::errors::error::{DiddocError, DiddocErrorKind};

use crate::errors::error::{AriesVcxError, AriesVcxErrorKind};

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
        }
    }
}
