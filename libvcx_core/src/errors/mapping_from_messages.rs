use aries_vcx::messages::errors::error::{MessagesError, MessagesErrorKind};

use crate::errors::error::{LibvcxError, LibvcxErrorKind};

impl From<MessagesError> for LibvcxError {
    fn from(msg_err: MessagesError) -> LibvcxError {
        let vcx_error_kind: LibvcxErrorKind = msg_err.kind().into();
        LibvcxError::from_msg(vcx_error_kind, msg_err.to_string())
    }
}

impl From<MessagesErrorKind> for LibvcxErrorKind {
    fn from(msg_err: MessagesErrorKind) -> LibvcxErrorKind {
        match msg_err {
            MessagesErrorKind::InvalidState => LibvcxErrorKind::InvalidState,
            MessagesErrorKind::InvalidJson => LibvcxErrorKind::InvalidJson,
            MessagesErrorKind::IOError => LibvcxErrorKind::IOError,
            MessagesErrorKind::InvalidDid => LibvcxErrorKind::InvalidDid,
            MessagesErrorKind::InvalidVerkey => LibvcxErrorKind::InvalidVerkey,
            MessagesErrorKind::InvalidUrl => LibvcxErrorKind::InvalidUrl,
            MessagesErrorKind::NotBase58 => LibvcxErrorKind::NotBase58,
            MessagesErrorKind::SerializationError => LibvcxErrorKind::SerializationError,
        }
    }
}
