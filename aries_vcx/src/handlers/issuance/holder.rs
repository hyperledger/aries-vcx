use std::collections::HashMap;

use indy_sys::{WalletHandle, PoolHandle};

use agency_client::agency_client::AgencyClient;

use crate::error::prelude::*;
use crate::handlers::connection::connection::Connection;
use crate::messages::a2a::A2AMessage;
use crate::messages::issuance::credential_offer::CredentialOffer;
use crate::messages::issuance::credential_proposal::CredentialProposalData;
use crate::protocols::issuance::actions::CredentialIssuanceAction;
use crate::protocols::issuance::holder::state_machine::{HolderSM, HolderState};
use crate::protocols::SendClosure;

#[derive(Serialize, Deserialize, Debug, Clone, Default)]
pub struct Holder {
    holder_sm: HolderSM,
}

impl Holder {
    pub fn create(source_id: &str) -> VcxResult<Holder> {
        trace!("Holder::create >>> source_id: {:?}", source_id);
        let holder_sm = HolderSM::new(source_id.to_string());
        Ok(Holder { holder_sm })
    }

    pub fn create_from_offer(source_id: &str, credential_offer: CredentialOffer) -> VcxResult<Holder> {
        trace!(
            "Holder::create_from_offer >>> source_id: {:?}, credential_offer: {:?}",
            source_id,
            credential_offer
        );
        let holder_sm = HolderSM::from_offer(credential_offer, source_id.to_string());
        Ok(Holder { holder_sm })
    }

    pub async fn send_proposal(
        &mut self,
        wallet_handle: WalletHandle,
        pool_handle: PoolHandle,
        credential_proposal: CredentialProposalData,
        send_message: SendClosure,
    ) -> VcxResult<()> {
        self.step(
            wallet_handle,
            pool_handle,
            CredentialIssuanceAction::CredentialProposalSend(credential_proposal),
            Some(send_message),
        )
        .await
    }

    pub async fn send_request(
        &mut self,
        wallet_handle: WalletHandle,
        pool_handle: PoolHandle,
        my_pw_did: String,
        send_message: SendClosure,
    ) -> VcxResult<()> {
        self.step(
            wallet_handle,
            pool_handle,
            CredentialIssuanceAction::CredentialRequestSend(my_pw_did),
            Some(send_message),
        )
        .await
    }

    pub async fn decline_offer<'a>(
        &'a mut self,
        wallet_handle: WalletHandle,
        pool_handle: PoolHandle,
        comment: Option<&'a str>,
        send_message: SendClosure,
    ) -> VcxResult<()> {
        self.step(
            wallet_handle,
            pool_handle,
            CredentialIssuanceAction::CredentialOfferReject(comment.map(String::from)),
            Some(send_message),
        )
        .await
    }

    pub fn is_terminal_state(&self) -> bool {
        self.holder_sm.is_terminal_state()
    }

    pub fn find_message_to_handle(&self, messages: HashMap<String, A2AMessage>) -> Option<(String, A2AMessage)> {
        self.holder_sm.find_message_to_handle(messages)
    }

    pub fn get_state(&self) -> HolderState {
        self.holder_sm.get_state()
    }

    pub fn get_source_id(&self) -> String {
        self.holder_sm.get_source_id()
    }

    pub fn get_credential(&self) -> VcxResult<(String, A2AMessage)> {
        self.holder_sm.get_credential()
    }

    pub fn get_attributes(&self) -> VcxResult<String> {
        self.holder_sm.get_attributes()
    }

    pub fn get_attachment(&self) -> VcxResult<String> {
        self.holder_sm.get_attachment()
    }

    pub fn get_offer(&self) -> VcxResult<CredentialOffer> {
        self.holder_sm.get_offer()
    }

    pub fn get_tails_location(&self) -> VcxResult<String> {
        self.holder_sm.get_tails_location()
    }

    pub fn get_tails_hash(&self) -> VcxResult<String> {
        self.holder_sm.get_tails_hash()
    }

    pub fn get_rev_reg_id(&self) -> VcxResult<String> {
        self.holder_sm.get_rev_reg_id()
    }

    pub fn get_thread_id(&self) -> VcxResult<String> {
        self.holder_sm.get_thread_id()
    }

    pub async fn is_revokable(&self, wallet_handle: WalletHandle, pool_handle: PoolHandle) -> VcxResult<bool> {
        self.holder_sm.is_revokable(wallet_handle, pool_handle).await
    }

    pub async fn delete_credential(&self, wallet_handle: WalletHandle) -> VcxResult<()> {
        self.holder_sm.delete_credential(wallet_handle).await
    }

    pub fn get_credential_status(&self) -> VcxResult<u32> {
        Ok(self.holder_sm.credential_status())
    }

    pub async fn step(
        &mut self,
        wallet_handle: WalletHandle,
        pool_handle: PoolHandle,
        message: CredentialIssuanceAction,
        send_message: Option<SendClosure>,
    ) -> VcxResult<()> {
        self.holder_sm = self
            .holder_sm
            .clone()
            .handle_message(wallet_handle, pool_handle, message, send_message)
            .await?;
        Ok(())
    }

    pub async fn update_state(
        &mut self,
        wallet_handle: WalletHandle,
        pool_handle: PoolHandle,
        agency_client: &AgencyClient,
        connection: &Connection,
    ) -> VcxResult<HolderState> {
        trace!("Holder::update_state >>>");
        if self.is_terminal_state() {
            return Ok(self.get_state());
        }
        let send_message = connection.send_message_closure(wallet_handle, pool_handle)?;

        let messages = connection.get_messages(pool_handle, agency_client).await?;
        if let Some((uid, msg)) = self.find_message_to_handle(messages) {
            self.step(wallet_handle, pool_handle, msg.into(), Some(send_message)).await?;
            connection.update_message_status(&uid, agency_client).await?;
        }
        Ok(self.get_state())
    }
}

#[cfg(feature = "test_utils")]
pub mod test_utils {
    use indy_sys::PoolHandle;

    use agency_client::agency_client::AgencyClient;

    use crate::error::prelude::*;
    use crate::handlers::connection::connection::Connection;
    use crate::messages::a2a::A2AMessage;

    pub async fn get_credential_offer_messages(
        pool_handle: PoolHandle,
        agency_client: &AgencyClient,
        connection: &Connection,
    ) -> VcxResult<String> {
        let credential_offers: Vec<A2AMessage> = connection
            .get_messages(pool_handle, agency_client)
            .await?
            .into_iter()
            .filter_map(|(_, a2a_message)| match a2a_message {
                A2AMessage::CredentialOffer(_) => Some(a2a_message),
                _ => None,
            })
            .collect();

        Ok(json!(credential_offers).to_string())
    }
}

#[cfg(test)]
#[cfg(feature = "general_test")]
pub mod unit_tests {
    use crate::messages::issuance::credential::test_utils::_credential;
    use crate::messages::issuance::credential_offer::test_utils::_credential_offer;
    use crate::messages::issuance::credential_proposal::test_utils::_credential_proposal_data;
    use crate::messages::issuance::credential_request::test_utils::_my_pw_did;
    use crate::utils::devsetup::SetupMocks;

    use super::*;

    fn _dummy_wallet_handle() -> WalletHandle {
        WalletHandle(0)
    }

    fn _dummy_pool_handle() -> PoolHandle {
        0
    }

    pub fn _send_message() -> Option<SendClosure> {
        Some(Box::new(|_: A2AMessage| Box::pin(async { VcxResult::Ok(()) })))
    }

    fn _holder_from_offer() -> Holder {
        Holder::create_from_offer("test_source_id", _credential_offer()).unwrap()
    }

    fn _holder() -> Holder {
        Holder::create("test_source_id").unwrap()
    }

    impl Holder {
        async fn to_finished_state(mut self) -> Holder {
            self.step(
                _dummy_wallet_handle(),
                _dummy_pool_handle(),
                CredentialIssuanceAction::CredentialProposalSend(_credential_proposal_data()),
                _send_message(),
            )
            .await
            .unwrap();
            self.step(
                _dummy_wallet_handle(),
                _dummy_pool_handle(),
                CredentialIssuanceAction::CredentialOffer(_credential_offer()),
                _send_message(),
            )
            .await
            .unwrap();
            self.step(
                _dummy_wallet_handle(),
                _dummy_pool_handle(),
                CredentialIssuanceAction::CredentialRequestSend(_my_pw_did()),
                _send_message(),
            )
            .await
            .unwrap();
            self.step(
                _dummy_wallet_handle(),
                _dummy_pool_handle(),
                CredentialIssuanceAction::Credential(_credential()),
                _send_message(),
            )
            .await
            .unwrap();
            self
        }
    }

    #[tokio::test]
    async fn exchange_credential_from_proposal_without_negotiation() {
        let _setup = SetupMocks::init();
        let holder = _holder().to_finished_state().await;
        assert_eq!(HolderState::Finished, holder.get_state());
    }

    #[tokio::test]
    async fn exchange_credential_from_proposal_with_negotiation() {
        let _setup = SetupMocks::init();
        let mut holder = _holder();
        assert_eq!(HolderState::Initial, holder.get_state());

        holder
            .send_proposal(
                _dummy_wallet_handle(),
                _dummy_pool_handle(),
                _credential_proposal_data(),
                _send_message().unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(HolderState::ProposalSent, holder.get_state());

        let messages = map!(
            "key_1".to_string() => A2AMessage::CredentialOffer(_credential_offer())
        );
        let (_, msg) = holder.find_message_to_handle(messages).unwrap();
        holder
            .step(_dummy_wallet_handle(), _dummy_pool_handle(), msg.into(), _send_message())
            .await
            .unwrap();
        assert_eq!(HolderState::OfferReceived, holder.get_state());

        holder
            .send_proposal(
                _dummy_wallet_handle(),
                _dummy_pool_handle(),
                _credential_proposal_data(),
                _send_message().unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(HolderState::ProposalSent, holder.get_state());

        let messages = map!(
            "key_1".to_string() => A2AMessage::CredentialOffer(_credential_offer())
        );
        let (_, msg) = holder.find_message_to_handle(messages).unwrap();
        holder
            .step(_dummy_wallet_handle(), _dummy_pool_handle(), msg.into(), _send_message())
            .await
            .unwrap();
        assert_eq!(HolderState::OfferReceived, holder.get_state());

        holder
            .send_request(_dummy_wallet_handle(), _dummy_pool_handle(), _my_pw_did(), _send_message().unwrap())
            .await
            .unwrap();
        assert_eq!(HolderState::RequestSent, holder.get_state());

        let messages = map!(
            "key_1".to_string() => A2AMessage::Credential(_credential())
        );
        let (_, msg) = holder.find_message_to_handle(messages).unwrap();
        holder
            .step(_dummy_wallet_handle(), _dummy_pool_handle(), msg.into(), _send_message())
            .await
            .unwrap();
        assert_eq!(HolderState::Finished, holder.get_state());
    }
}
