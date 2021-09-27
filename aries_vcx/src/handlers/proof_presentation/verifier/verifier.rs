use std::collections::HashMap;

use crate::error::prelude::*;
use crate::handlers::connection::connection::Connection;
use crate::handlers::proof_presentation::verifier::messages::VerifierMessages;
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
    PresentationRequestSent,
    Finished,
    Failed,
}

impl Verifier {
    pub fn create(source_id: String,
                  requested_attrs: String,
                  requested_predicates: String,
                  revocation_details: String,
                  name: String) -> VcxResult<Verifier> {
        trace!("Verifier::create >>> source_id: {:?}, requested_attrs: {:?}, requested_predicates: {:?}, revocation_details: {:?}, name: {:?}",
               source_id, requested_attrs, requested_predicates, revocation_details, name);

        let presentation_request =
            PresentationRequestData::create()
                .set_name(name)
                .set_requested_attributes(requested_attrs)?
                .set_requested_predicates(requested_predicates)?
                .set_not_revoked_interval(revocation_details)?
                .set_nonce()?;

        Ok(Verifier {
            verifier_sm: VerifierSM::new(presentation_request, source_id),
        })
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

    pub fn get_thread_id(&self) -> VcxResult<String> {
        Ok(self.verifier_sm.thread_id())
    }

    pub fn step(&mut self, message: VerifierMessages, send_message: Option<&impl Fn(&A2AMessage) -> VcxResult<()>>)
                -> VcxResult<()>
    {
        self.verifier_sm = self.verifier_sm.clone().step(message, send_message)?;
        Ok(())
    }

    pub fn has_transitions(&self) -> bool {
        self.verifier_sm.has_transitions()
    }

    pub fn find_message_to_handle(&self, messages: HashMap<String, A2AMessage>) -> Option<(String, A2AMessage)> {
        self.verifier_sm.find_message_to_handle(messages)
    }

    pub fn update_state(&mut self, connection: &Connection) -> VcxResult<VerifierState> {
        trace!("Verifier::update_state >>> ");
        if !self.has_transitions() { return Ok(self.get_state()); }
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
    use crate::utils::mockdata::mockdata_proof;

    use super::*;

    fn _verifier() -> Verifier {
        Verifier::create("1".to_string(),
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
