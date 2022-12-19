use messages::utils::error::MessagesError;
use crate::error::{VcxError, VcxErrorKind};
use messages::utils::error::MessagesErrorKind;

impl From<MessagesError> for VcxError {
    fn from(msg_err: MessagesError) -> VcxError {
        let vcx_error_kind: VcxErrorKind = msg_err.kind().into();
        VcxError::from_msg(vcx_error_kind, msg_err.to_string())
    }
}

impl From<MessagesErrorKind> for VcxErrorKind {
    fn from(msg_err: MessagesErrorKind) -> VcxErrorKind {
        match msg_err {
            MessagesErrorKind::InvalidState => VcxErrorKind::InvalidState,
            MessagesErrorKind::InvalidJson => VcxErrorKind::InvalidJson,
            MessagesErrorKind::IOError => VcxErrorKind::IOError,
            MessagesErrorKind::InvalidDid => VcxErrorKind::InvalidDid,
            MessagesErrorKind::InvalidVerkey => VcxErrorKind::InvalidVerkey,
            MessagesErrorKind::InvalidUrl => VcxErrorKind::InvalidUrl,
            MessagesErrorKind::NotBase58 => VcxErrorKind::NotBase58,
            _ => VcxErrorKind::UnknownLibndyError,
        }
    }
}
