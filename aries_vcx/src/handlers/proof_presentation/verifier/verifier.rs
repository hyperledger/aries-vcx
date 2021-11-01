use std::collections::HashMap;

use crate::error::prelude::*;
use crate::handlers::connection::connection::Connection;
use crate::handlers::proof_presentation::verifier::messages::VerifierMessages;
use crate::messages::proof_presentation::presentation_proposal::PresentationProposal;
use crate::messages::proof_presentation::presentation_request::PresentationRequestData;
use crate::handlers::proof_presentation::verifier::state_machine::VerifierSM;
use crate::messages::a2a::A2AMessage;
use crate::messages::proof_presentation::presentation_request::*;

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Default)]
pub struct Verifier {
    verifier_sm: VerifierSM,
}

#[derive(Debug, PartialEq)]
pub enum VerifierState {
    Initial,
    PresentationProposalReceived,
    PresentationRequestSet,
    PresentationRequestSent,
    Finished,
    Failed,
}

impl Verifier {
    pub fn create(source_id: &str) -> VcxResult<Self> {
        trace!("Verifier::create >>> source_id: {:?}", source_id);

        Ok(Self {
            verifier_sm: VerifierSM::new(source_id),
        })
    }

    pub fn create_from_request(source_id: String,
                  requested_attrs: String,
                  requested_predicates: String,
                  revocation_details: String,
                  name: String) -> VcxResult<Self> {
        trace!("Verifier::create_from_request >>> source_id: {:?}, requested_attrs: {:?}, requested_predicates: {:?}, revocation_details: {:?}, name: {:?}",
               source_id, requested_attrs, requested_predicates, revocation_details, name);

        let presentation_request =
            PresentationRequestData::create(&name)?
                .set_requested_attributes_as_string(requested_attrs)?
                .set_requested_predicates_as_string(requested_predicates)?
                .set_not_revoked_interval(revocation_details)?;

        Ok(Self {
            verifier_sm: VerifierSM::from_request(&source_id, presentation_request),
        })
    }

    pub fn create_from_proposal(source_id: &str, presentation_proposal: &PresentationProposal) -> VcxResult<Self> {
        trace!("Issuer::create_from_proposal >>> source_id: {:?}, presentation_proposal: {:?}", source_id, presentation_proposal);
        Ok(Self { verifier_sm: VerifierSM::from_proposal(source_id, presentation_proposal) })
    }

    pub fn get_source_id(&self) -> String { self.verifier_sm.source_id() }

    pub fn get_state(&self) -> VerifierState {
        trace!("Verifier::get_state >>>");
        self.verifier_sm.get_state()
    }

    pub fn presentation_status(&self) -> u32 {
        trace!("Verifier::presentation_state >>>");
        self.verifier_sm.presentation_status()
    }

    pub fn handle_message(&mut self, message: VerifierMessages, send_message: Option<&impl Fn(&A2AMessage) -> VcxResult<()>>) -> VcxResult<()> {
        trace!("Verifier::handle_message >>> message: {:?}", message);
        self.step(message, send_message)
    }

    pub fn send_presentation_request(&mut self, send_message: impl Fn(&A2AMessage) -> VcxResult<()>, comment: Option<String>) -> VcxResult<()> {
        trace!("Verifier::send_presentation_request >>>");
        self.step(VerifierMessages::SendPresentationRequest(comment), Some(&send_message))
    }

    pub fn set_request(&mut self, presentation_request_data: PresentationRequestData) -> VcxResult<()> {
        self.verifier_sm = self.verifier_sm.clone().set_request(presentation_request_data)?;
        Ok(())
    }

    pub fn generate_presentation_request_msg(&self) -> VcxResult<String> {
        trace!("Verifier::generate_presentation_request_msg >>>");
        let proof_request = self.verifier_sm.presentation_request()?;
        Ok(json!(proof_request).to_string())
    }

    pub fn generate_presentation_request(&self) -> VcxResult<PresentationRequest> {
        trace!("Verifier::generate_presentation_request >>>");
        let proof_request = self.verifier_sm.presentation_request()?;
        Ok(proof_request)
    }

    pub fn get_presentation(&self) -> VcxResult<String> {
        trace!("Verifier::get_presentation >>>");
        let proof = self.verifier_sm.presentation()?.to_a2a_message();
        Ok(json!(proof).to_string())
    }

    pub fn get_presentation_attachment(&self) -> VcxResult<String> {
        self.verifier_sm.presentation()?.presentations_attach.content()
    }

    pub fn get_presentation_proposal(&self) -> VcxResult<PresentationProposal> {
        trace!("Verifier::get_presentation_proposal >>>");
        self.verifier_sm.presentation_proposal()
    }

    pub fn get_thread_id(&self) -> VcxResult<String> {
        Ok(self.verifier_sm.thread_id())
    }

    pub fn step(&mut self, message: VerifierMessages, send_message: Option<&impl Fn(&A2AMessage) -> VcxResult<()>>)
                -> VcxResult<()> {
        self.verifier_sm = self.verifier_sm.clone().step(message, send_message)?;
        Ok(())
    }

    pub fn progressable_by_message(&self) -> bool {
        self.verifier_sm.progressable_by_message()
    }

    pub fn find_message_to_handle(&self, messages: HashMap<String, A2AMessage>) -> Option<(String, A2AMessage)> {
        self.verifier_sm.find_message_to_handle(messages)
    }

    pub fn decline_presentation_proposal(&mut self, send_message: &impl Fn(&A2AMessage) -> VcxResult<()>, reason: &str) -> VcxResult<()> {
        trace!("Verifier::decline_presentation_proposal >>> reason: {:?}", reason);
        self.step(VerifierMessages::RejectPresentationProposal(reason.to_string()), Some(send_message))
    }

    pub fn update_state(&mut self, connection: &Connection) -> VcxResult<VerifierState> {
        trace!("Verifier::update_state >>> ");
        if !self.progressable_by_message() { return Ok(self.get_state()); }
        let send_message = connection.send_message_closure()?;

        let messages = connection.get_messages()?;
        if let Some((uid, msg)) = self.find_message_to_handle(messages) {
            self.step(msg.into(), Some(&send_message))?;
            connection.update_message_status(uid)?;
        }
        Ok(self.get_state())
    }
}

#[cfg(test)]
mod tests {
    use crate::messages::proof_presentation::presentation::test_utils::_presentation;
    use crate::utils::mockdata::mock_settings::MockBuilder;
    use crate::messages::connection::did_doc::DidDoc;
    use crate::messages::a2a::A2AMessage;
    use crate::messages::basic_message::message::BasicMessage;
    use crate::utils::devsetup::*;
    use crate::messages::proof_presentation::presentation::test_utils::_comment;
    use crate::utils::send_message;
    use crate::utils::constants::{REQUESTED_ATTRS, REQUESTED_PREDICATES};

    use super::*;

    fn _verifier() -> Verifier {
        Verifier::create_from_request("1".to_string(),
                     REQUESTED_ATTRS.to_owned(),
                     REQUESTED_PREDICATES.to_owned(),
                     r#"{"support_revocation":false}"#.to_string(),
                     "Optional".to_owned()).unwrap()

    }

    pub fn _send_message() -> Option<&'static impl Fn(&A2AMessage) -> VcxResult<()>> {
        Some(&|_: &A2AMessage| send_message("", &DidDoc::default(), &A2AMessage::BasicMessage(BasicMessage::default())))
    }

    impl Verifier {
        fn to_presentation_request_sent_state(&mut self) {
            self.send_presentation_request(_send_message().unwrap(), _comment()).unwrap();
        }

        fn to_finished_state(&mut self) {
            self.to_presentation_request_sent_state();
            self.step(VerifierMessages::VerifyPresentation(_presentation()), _send_message()).unwrap();
        }
    }

    #[test]
    #[cfg(feature = "general_test")]
    fn test_get_presentation() {
        let _setup = SetupMocks::init();
        let _mock_builder = MockBuilder::init().
            set_mock_result_for_validate_indy_proof(Ok(true));
        let mut verifier = _verifier();
        verifier.to_finished_state();
        let presentation = verifier.get_presentation().unwrap();
        assert_eq!(presentation, json!(_presentation().to_a2a_message()).to_string());
        assert_eq!(verifier.get_state(), VerifierState::Finished);
    }
}
