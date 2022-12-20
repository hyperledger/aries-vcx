use aries_vcx::agency_client::errors::error::{ErrorAgencyClient, ErrorKindAgencyClient};
use crate::api_lib::errors::error::{ErrorLibvcx, ErrorKindLibvcx};

impl From<ErrorAgencyClient> for ErrorLibvcx {
    fn from(agency_err: ErrorAgencyClient) -> ErrorLibvcx {
        let vcx_error_kind: ErrorKindLibvcx = agency_err.kind().into();
        ErrorLibvcx::from_msg(vcx_error_kind, agency_err.to_string())
    }
}

impl From<ErrorKindAgencyClient> for ErrorKindLibvcx {
    fn from(agency_err: ErrorKindAgencyClient) -> ErrorKindLibvcx {
        match agency_err {
            ErrorKindAgencyClient::InvalidState => ErrorKindLibvcx::InvalidState,
            ErrorKindAgencyClient::InvalidConfiguration => ErrorKindLibvcx::InvalidConfiguration,
            ErrorKindAgencyClient::InvalidJson => ErrorKindLibvcx::InvalidJson,
            ErrorKindAgencyClient::InvalidOption => ErrorKindLibvcx::InvalidOption,
            ErrorKindAgencyClient::InvalidMessagePack => ErrorKindLibvcx::InvalidMessagePack,
            ErrorKindAgencyClient::IOError => ErrorKindLibvcx::IOError,
            ErrorKindAgencyClient::PostMessageFailed => ErrorKindLibvcx::PostMessageFailed,
            ErrorKindAgencyClient::InvalidWalletHandle => ErrorKindLibvcx::InvalidWalletHandle,
            ErrorKindAgencyClient::UnknownError => ErrorKindLibvcx::UnknownError,
            ErrorKindAgencyClient::InvalidDid => ErrorKindLibvcx::InvalidDid,
            ErrorKindAgencyClient::InvalidVerkey => ErrorKindLibvcx::InvalidVerkey,
            ErrorKindAgencyClient::InvalidUrl => ErrorKindLibvcx::InvalidUrl,
            ErrorKindAgencyClient::SerializationError => ErrorKindLibvcx::SerializationError,
            ErrorKindAgencyClient::NotBase58 => ErrorKindLibvcx::NotBase58,
            ErrorKindAgencyClient::InvalidHttpResponse => ErrorKindLibvcx::InvalidHttpResponse,
        }
    }
}
