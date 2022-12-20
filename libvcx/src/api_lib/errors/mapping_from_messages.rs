use aries_vcx::messages::errors::error::ErrorMessages;
use aries_vcx::messages::errors::error::ErrorKindMessages;
use crate::api_lib::errors::error::{ErrorLibvcx, ErrorKindLibvcx};

impl From<ErrorMessages> for ErrorLibvcx {
    fn from(msg_err: ErrorMessages) -> ErrorLibvcx {
        let vcx_error_kind: ErrorKindLibvcx = msg_err.kind().into();
        ErrorLibvcx::from_msg(vcx_error_kind, msg_err.to_string())
    }
}

impl From<ErrorKindMessages> for ErrorKindLibvcx {
    fn from(msg_err: ErrorKindMessages) -> ErrorKindLibvcx {
        match msg_err {
            ErrorKindMessages::InvalidState => ErrorKindLibvcx::InvalidState,
            ErrorKindMessages::InvalidJson => ErrorKindLibvcx::InvalidJson,
            ErrorKindMessages::IOError => ErrorKindLibvcx::IOError,
            ErrorKindMessages::InvalidDid => ErrorKindLibvcx::InvalidDid,
            ErrorKindMessages::InvalidVerkey => ErrorKindLibvcx::InvalidVerkey,
            ErrorKindMessages::InvalidUrl => ErrorKindLibvcx::InvalidUrl,
            ErrorKindMessages::NotBase58 => ErrorKindLibvcx::NotBase58,
            ErrorKindMessages::SerializationError => ErrorKindLibvcx::SerializationError
        }
    }
}
