use std::collections::HashMap;

use messages::status::Status;
use messages::proof_presentation::presentation::Presentation;
use std::sync::Arc;

use agency_client::agency_client::AgencyClient;

use crate::core::profile::profile::Profile;
use crate::error::prelude::*;
use crate::handlers::connection::connection::Connection;
use crate::xyz::proofs::proof_request::PresentationRequestData;
use crate::protocols::proof_presentation::verifier::messages::VerifierMessages;
use crate::protocols::proof_presentation::verifier::state_machine::{VerifierSM, VerifierState};
use crate::protocols::SendClosure;
use messages::a2a::A2AMessage;
use messages::proof_presentation::presentation_proposal::PresentationProposal;
use messages::proof_presentation::presentation_request::PresentationRequest;

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Default)]
pub struct Verifier {
    verifier_sm: VerifierSM,
}

impl Verifier {
    pub fn create(source_id: &str) -> VcxResult<Self> {
        trace!("Verifier::create >>> source_id: {:?}", source_id);

        Ok(Self {
            verifier_sm: VerifierSM::new(source_id),
        })
    }

    pub fn create_from_request(source_id: String, presentation_request: &PresentationRequestData) -> VcxResult<Self> {
        trace!(
            "Verifier::create_from_request >>> source_id: {:?}, presentation_request: {:?}",
            source_id,
            presentation_request
        );
        let verifier_sm = VerifierSM::from_request(&source_id, presentation_request)?;
        Ok(Self { verifier_sm })
    }

    pub fn create_from_proposal(source_id: &str, presentation_proposal: &PresentationProposal) -> VcxResult<Self> {
        trace!(
            "Issuer::create_from_proposal >>> source_id: {:?}, presentation_proposal: {:?}",
            source_id,
            presentation_proposal
        );
        Ok(Self {
            verifier_sm: VerifierSM::from_proposal(source_id, presentation_proposal),
        })
    }

    pub fn get_source_id(&self) -> String {
        self.verifier_sm.source_id()
    }

    pub fn get_state(&self) -> VerifierState {
        self.verifier_sm.get_state()
    }

    pub async fn handle_message(
        &mut self,
        profile: &Arc<dyn Profile>,
        message: VerifierMessages,
        send_message: Option<SendClosure>,
    ) -> VcxResult<()> {
        trace!("Verifier::handle_message >>> message: {:?}", message);
        self.step(profile, message, send_message).await
    }

    pub async fn send_presentation_request(&mut self, send_message: SendClosure) -> VcxResult<()> {
        if self.verifier_sm.get_state() == VerifierState::PresentationRequestSet {
            let offer = self.verifier_sm.presentation_request()?.to_a2a_message();
            send_message(offer).await?;
            self.verifier_sm = self.verifier_sm.clone().mark_presentation_request_msg_sent()?;
        }
        Ok(())
    }

    pub async fn send_presentation_ack(&mut self, send_message: SendClosure) -> VcxResult<()> {
        trace!("Verifier::send_presentation_ack >>>");
        self.verifier_sm = self.verifier_sm.clone().send_presentation_ack(send_message).await?;
        Ok(())
    }

    pub async fn verify_presentation(&mut self, profile: &Arc<dyn Profile>, presentation: Presentation, send_message: SendClosure) -> VcxResult<()> {
        trace!("Verifier::verify_presentation >>>");
        self.verifier_sm = self.verifier_sm.clone().verify_presentation(profile, presentation, send_message).await?;
        Ok(())
    }

    pub fn set_request(
        &mut self,
        presentation_request_data: PresentationRequestData,
        comment: Option<String>,
    ) -> VcxResult<()> {
        trace!(
            "Verifier::set_request >>> presentation_request_data: {:?}, comment: ${:?}",
            presentation_request_data,
            comment
        );
        self.verifier_sm = self
            .verifier_sm
            .clone()
            .set_request(&presentation_request_data, comment)?;
        Ok(())
    }

    pub fn mark_presentation_request_msg_sent(&mut self) -> VcxResult<()> {
        trace!("Verifier::mark_presentation_request_msg_sent >>>");
        self.verifier_sm = self.verifier_sm.clone().mark_presentation_request_msg_sent()?;
        Ok(())
    }

    pub fn get_presentation_request_msg(&self) -> VcxResult<String> {
        let msg = self.verifier_sm.presentation_request()?.to_a2a_message();
        Ok(json!(msg).to_string())
    }

    pub fn get_presentation_request(&self) -> VcxResult<PresentationRequest> {
        self.verifier_sm.presentation_request()
    }

    pub fn get_presentation_msg(&self) -> VcxResult<String> {
        trace!("Verifier::get_presentation >>>");
        let msg = self.verifier_sm.presentation()?.to_a2a_message();
        Ok(json!(msg).to_string())
    }

    pub fn get_presentation_status(&self) -> Status {
        trace!("Verifier::presentation_state >>>");
        self.verifier_sm.presentation_status()
    }

    pub fn get_presentation_attachment(&self) -> VcxResult<String> {
        self.verifier_sm
            .presentation()?
            .presentations_attach
            .content()
            .map_err(|err| err.into())
    }

    pub fn get_presentation_proposal(&self) -> VcxResult<PresentationProposal> {
        trace!("Verifier::get_presentation_proposal >>>");
        self.verifier_sm.presentation_proposal()
    }

    pub fn get_thread_id(&self) -> VcxResult<String> {
        Ok(self.verifier_sm.thread_id())
    }

    pub async fn step(
        &mut self,
        profile: &Arc<dyn Profile>,
        message: VerifierMessages,
        send_message: Option<SendClosure>,
    ) -> VcxResult<()> {
        self.verifier_sm = self
            .verifier_sm
            .clone()
            .step(profile, message, send_message)
            .await?;
        Ok(())
    }

    pub fn progressable_by_message(&self) -> bool {
        self.verifier_sm.progressable_by_message()
    }

    pub fn find_message_to_handle(&self, messages: HashMap<String, A2AMessage>) -> Option<(String, A2AMessage)> {
        self.verifier_sm.find_message_to_handle(messages)
    }

    pub async fn decline_presentation_proposal<'a>(
        &'a mut self,
        send_message: SendClosure,
        reason: &'a str,
    ) -> VcxResult<()> {
        trace!("Verifier::decline_presentation_proposal >>> reason: {:?}", reason);
        self.verifier_sm = self.verifier_sm.clone().reject_presentation_proposal(reason.to_string(), send_message).await?;
        Ok(())
    }

    pub async fn update_state(
        &mut self,
        profile: &Arc<dyn Profile>,
        agency_client: &AgencyClient,
        connection: &Connection,
    ) -> VcxResult<VerifierState> {
        trace!("Verifier::update_state >>> ");
        if !self.progressable_by_message() {
            return Ok(self.get_state());
        }
        let send_message = connection.send_message_closure(profile).await?;

        let messages = connection.get_messages(agency_client).await?;
        if let Some((uid, msg)) = self.find_message_to_handle(messages) {
            self.step(profile, msg.into(), Some(send_message)).await?;
            connection.update_message_status(&uid, agency_client).await?;
        }
        Ok(self.get_state())
    }
}

#[cfg(test)]
#[cfg(feature = "general_test")]
mod unit_tests {
    use crate::core::profile::indy_profile::IndySdkProfile;
    use crate::utils::constants::{REQUESTED_ATTRS, REQUESTED_PREDICATES};
    use crate::utils::devsetup::*;
    use crate::utils::mockdata::mock_settings::MockBuilder;
    use messages::a2a::A2AMessage;
    use messages::proof_presentation::presentation::test_utils::_presentation;
    use vdrtools_sys::WalletHandle;

    use super::*;

    fn _dummy_profile() -> Arc<dyn Profile> {
        Arc::new(IndySdkProfile::new(WalletHandle(0), 0))
    }

    async fn _verifier() -> Verifier {
        let presentation_request_data = PresentationRequestData::create(&_dummy_profile(), "1")
            .await
            .unwrap()
            .set_requested_attributes_as_string(REQUESTED_ATTRS.to_owned())
            .unwrap()
            .set_requested_predicates_as_string(REQUESTED_PREDICATES.to_owned())
            .unwrap()
            .set_not_revoked_interval(r#"{"support_revocation":false}"#.to_string())
            .unwrap();
        Verifier::create_from_request("1".to_string(), &presentation_request_data).unwrap()
    }

    pub fn _send_message() -> Option<SendClosure> {
        Some(Box::new(|_: A2AMessage| Box::pin(async { VcxResult::Ok(()) })))
    }

    impl Verifier {
        async fn to_presentation_request_sent_state(&mut self) {
            self.send_presentation_request(_send_message().unwrap()).await.unwrap();
        }

        async fn to_finished_state(&mut self) {
            self.to_presentation_request_sent_state().await;
            self.step(
                &_dummy_profile(),
                VerifierMessages::VerifyPresentation(_presentation()),
                _send_message(),
            )
            .await
            .unwrap();
        }
    }

    #[tokio::test]
    async fn test_get_presentation() {
        let _setup = SetupMocks::init();
        let _mock_builder = MockBuilder::init().set_mock_result_for_validate_indy_proof(Ok(true));
        let mut verifier = _verifier().await;
        verifier.to_finished_state().await;
        let presentation = verifier.get_presentation_msg().unwrap();
        assert_eq!(presentation, json!(_presentation().to_a2a_message()).to_string());
        assert_eq!(verifier.get_state(), VerifierState::Finished);
    }
}
