use async_trait::async_trait;

use crate::error::prelude::{AgencyClientResult, AgencyClientError, AgencyClientErrorKind};

#[async_trait]
pub trait BaseAgencyClientWallet : std::fmt::Debug + Send + Sync {
    async fn pack_message(
        &self,
        sender_vk: Option<&str>,
        receiver_keys: &str,
        msg: &[u8],
    ) -> AgencyClientResult<Vec<u8>>;

    async fn unpack_message(&self, msg: &[u8]) -> AgencyClientResult<Vec<u8>>;
}

#[derive(Debug)]
pub(crate) struct StubAgencyClientWallet;

#[async_trait]
impl BaseAgencyClientWallet for StubAgencyClientWallet {
    async fn pack_message(
        &self,
        _sender_vk: Option<&str>,
        _receiver_keys: &str,
        _msg: &[u8],
    ) -> AgencyClientResult<Vec<u8>> {
        Err(AgencyClientError::from_msg(AgencyClientErrorKind::UnknownError, "Error - using a stub method: StubAgencyClientWallet::pack_message"))
    }

    async fn unpack_message(&self, _msg: &[u8]) -> AgencyClientResult<Vec<u8>> {
        Err(AgencyClientError::from_msg(AgencyClientErrorKind::UnknownError, "Error - using a stub method: StubAgencyClientWallet::unpack_message"))
    }
}