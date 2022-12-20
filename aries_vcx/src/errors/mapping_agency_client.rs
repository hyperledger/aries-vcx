use agency_client::errors::error::{ErrorAgencyClient, ErrorKindAgencyClient};
use crate::errors::error::{ErrorAriesVcx, ErrorKindAriesVcx};

impl From<ErrorAgencyClient> for ErrorAriesVcx {
    fn from(agency_err: ErrorAgencyClient) -> ErrorAriesVcx {
        let vcx_error_kind: ErrorKindAriesVcx = agency_err.kind().into();
        ErrorAriesVcx::from_msg(vcx_error_kind, agency_err.to_string())
    }
}

impl From<ErrorKindAgencyClient> for ErrorKindAriesVcx {
    fn from(agency_err: ErrorKindAgencyClient) -> ErrorKindAriesVcx {
        match agency_err {
            ErrorKindAgencyClient::InvalidState => ErrorKindAriesVcx::InvalidState,
            ErrorKindAgencyClient::InvalidConfiguration => ErrorKindAriesVcx::InvalidConfiguration,
            ErrorKindAgencyClient::InvalidJson => ErrorKindAriesVcx::InvalidJson,
            ErrorKindAgencyClient::InvalidOption => ErrorKindAriesVcx::InvalidOption,
            ErrorKindAgencyClient::InvalidMessagePack => ErrorKindAriesVcx::InvalidMessagePack,
            ErrorKindAgencyClient::IOError => ErrorKindAriesVcx::IOError,
            ErrorKindAgencyClient::PostMessageFailed => ErrorKindAriesVcx::PostMessageFailed,
            ErrorKindAgencyClient::InvalidWalletHandle => ErrorKindAriesVcx::InvalidWalletHandle,
            ErrorKindAgencyClient::UnknownError => ErrorKindAriesVcx::UnknownError,
            ErrorKindAgencyClient::InvalidDid => ErrorKindAriesVcx::InvalidDid,
            ErrorKindAgencyClient::InvalidVerkey => ErrorKindAriesVcx::InvalidVerkey,
            ErrorKindAgencyClient::InvalidUrl => ErrorKindAriesVcx::InvalidUrl,
            ErrorKindAgencyClient::SerializationError => ErrorKindAriesVcx::SerializationError,
            ErrorKindAgencyClient::NotBase58 => ErrorKindAriesVcx::NotBase58,
            ErrorKindAgencyClient::InvalidHttpResponse => ErrorKindAriesVcx::InvalidHttpResponse,
        }
    }
}
