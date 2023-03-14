use messages::errors::error::{MessagesError, MessagesErrorKind};

use crate::errors::error::{AriesVcxError, AriesVcxErrorKind};

impl From<MessagesError> for AriesVcxError {
    fn from(msg_err: MessagesError) -> AriesVcxError {
        let vcx_error_kind: AriesVcxErrorKind = msg_err.kind().into();
        AriesVcxError::from_msg(vcx_error_kind, msg_err.to_string())
    }
}

impl From<MessagesErrorKind> for AriesVcxErrorKind {
    fn from(msg_err: MessagesErrorKind) -> AriesVcxErrorKind {
        match msg_err {
            MessagesErrorKind::InvalidState => AriesVcxErrorKind::InvalidState,
            MessagesErrorKind::InvalidJson => AriesVcxErrorKind::InvalidJson,
            MessagesErrorKind::IOError => AriesVcxErrorKind::IOError,
            MessagesErrorKind::InvalidDid => AriesVcxErrorKind::InvalidDid,
            MessagesErrorKind::InvalidVerkey => AriesVcxErrorKind::InvalidVerkey,
            MessagesErrorKind::InvalidUrl => AriesVcxErrorKind::InvalidUrl,
            MessagesErrorKind::NotBase58 => AriesVcxErrorKind::NotBase58,
            MessagesErrorKind::SerializationError => AriesVcxErrorKind::SerializationError,
        }
    }
}
