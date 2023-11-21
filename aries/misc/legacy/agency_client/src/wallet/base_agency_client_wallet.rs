use async_trait::async_trait;

use crate::{
    errors::error::{AgencyClientError, AgencyClientErrorKind, AgencyClientResult},
    testing::mocking::agency_mocks_enabled,
};

#[async_trait]
pub trait BaseAgencyClientWallet: std::fmt::Debug + Send + Sync {
    async fn pack_message(
        &self,
        sender_vk: Option<&str>,
        receiver_keys: &str,
        msg: &[u8],
    ) -> AgencyClientResult<Vec<u8>>;

    async fn unpack_message(&self, msg: &[u8]) -> AgencyClientResult<Vec<u8>>;
}

// Stub of [BaseAgencyClientWallet] used by [AgencyClient::new] when creating a stub [AgencyClient]
// Should never be used, and should be overwritten with a proper [BaseAgencyClientWallet]
// implementation.
#[derive(Debug)]
pub(crate) struct StubAgencyClientWallet;

#[async_trait]
impl BaseAgencyClientWallet for StubAgencyClientWallet {
    async fn pack_message(
        &self,
        _sender_vk: Option<&str>,
        _receiver_keys: &str,
        msg: &[u8],
    ) -> AgencyClientResult<Vec<u8>> {
        if agency_mocks_enabled() {
            trace!("pack_message >>> mocks enabled, returning message");
            return Ok(msg.to_vec());
        }
        Err(AgencyClientError::from_msg(
            AgencyClientErrorKind::UnknownError,
            "Error - using a stub method: StubAgencyClientWallet::pack_message",
        ))
    }

    async fn unpack_message(&self, msg: &[u8]) -> AgencyClientResult<Vec<u8>> {
        if agency_mocks_enabled() {
            trace!("pack_message >>> mocks enabled, returning message");
            return Ok(msg.to_vec());
        }
        Err(AgencyClientError::from_msg(
            AgencyClientErrorKind::UnknownError,
            "Error - using a stub method: StubAgencyClientWallet::unpack_message",
        ))
    }
}
