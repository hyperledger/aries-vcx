use messages::errors::error::ErrorMessages;
use crate::errors::error::{ErrorAriesVcx, ErrorKindAriesVcx};
use messages::errors::error::ErrorKindMessages;

impl From<ErrorMessages> for ErrorAriesVcx {
    fn from(msg_err: ErrorMessages) -> ErrorAriesVcx {
        let vcx_error_kind: ErrorKindAriesVcx = msg_err.kind().into();
        ErrorAriesVcx::from_msg(vcx_error_kind, msg_err.to_string())
    }
}

impl From<ErrorKindMessages> for ErrorKindAriesVcx {
    fn from(msg_err: ErrorKindMessages) -> ErrorKindAriesVcx {
        match msg_err {
            ErrorKindMessages::InvalidState => ErrorKindAriesVcx::InvalidState,
            ErrorKindMessages::InvalidJson => ErrorKindAriesVcx::InvalidJson,
            ErrorKindMessages::IOError => ErrorKindAriesVcx::IOError,
            ErrorKindMessages::InvalidDid => ErrorKindAriesVcx::InvalidDid,
            ErrorKindMessages::InvalidVerkey => ErrorKindAriesVcx::InvalidVerkey,
            ErrorKindMessages::InvalidUrl => ErrorKindAriesVcx::InvalidUrl,
            ErrorKindMessages::NotBase58 => ErrorKindAriesVcx::NotBase58,
            _ => ErrorKindAriesVcx::UnknownLibndyError,
        }
    }
}
