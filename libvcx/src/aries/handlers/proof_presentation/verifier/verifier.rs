use error::prelude::*;
use aries::handlers::proof_presentation::verifier::messages::VerifierMessages;
use aries::handlers::proof_presentation::verifier::state_machine::VerifierSM;
use aries::messages::a2a::A2AMessage;
use aries::messages::proof_presentation::presentation::Presentation;
use aries::messages::proof_presentation::presentation_request::*;
use std::collections::HashMap;

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
pub struct Verifier {
    verifier_sm: VerifierSM
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

    pub fn state(&self) -> u32 {
        trace!("Verifier::state >>>");
        self.verifier_sm.state()
    }

    pub fn presentation_status(&self) -> u32 {
        trace!("Verifier::presentation_state >>>");
        self.verifier_sm.presentation_status()
    }

    pub fn update_state_with_message(&mut self, message: &str) -> VcxResult<u32> {
        trace!("Verifier::update_state_with_message >>> message: {:?}", message);

        let message: A2AMessage = ::serde_json::from_str(&message)
            .map_err(|err| VcxError::from_msg(VcxErrorKind::InvalidOption, format!("Cannot update state with message: Message deserialization failed: {:?}", err)))?;

        self.handle_message(message.into())?;

        Ok(self.state())
    }

    pub fn handle_message(&mut self, message: VerifierMessages) -> VcxResult<()> {
        trace!("Verifier::handle_message >>> message: {:?}", message);
        self.step(message)
    }

    pub fn verify_presentation(&mut self, presentation: Presentation) -> VcxResult<()> {
        trace!("Verifier::verify_presentation >>> presentation: {:?}", presentation);
        self.step(VerifierMessages::VerifyPresentation(presentation))
    }

    pub fn send_presentation_request(&mut self, connection_handle: u32) -> VcxResult<()> {
        trace!("Verifier::send_presentation_request >>> connection_handle: {:?}", connection_handle);
        self.step(VerifierMessages::SendPresentationRequest(connection_handle))
    }

    pub fn generate_presentation_request_msg(&self) -> VcxResult<String> {
        trace!("Verifier::generate_presentation_request_msg >>>");

        let proof_request = self.verifier_sm.presentation_request()?;

        Ok(json!(proof_request).to_string())
    }

    pub fn get_presentation(&self) -> VcxResult<String> {
        trace!("Verifier::get_presentation >>>");

        let proof = self.verifier_sm.presentation()?.to_a2a_message();
        Ok(json!(proof).to_string())
    }

    pub fn step(&mut self, message: VerifierMessages) -> VcxResult<()> {
        self.verifier_sm = self.verifier_sm.clone().step(message)?;
        Ok(())
    }

    pub fn has_transitions(&self) -> bool {
        self.verifier_sm.has_transitions()
    }

    pub fn maybe_update_connection_handle(&mut self, connection_handle: Option<u32>) -> VcxResult<u32> {
        let connection_handle = connection_handle.unwrap_or(self.verifier_sm.connection_handle()?);
        self.verifier_sm.set_connection_handle(connection_handle);
        Ok(connection_handle)
    }

    pub fn find_message_to_handle(&self, messages: HashMap<String, A2AMessage>) -> Option<(String, A2AMessage)> {
        self.verifier_sm.find_message_to_handle(messages)
    }
}

#[cfg(test)]
pub mod tests {
    use api::VcxStateType;
    use connection::tests::build_test_connection_inviter_requested;
    use utils::constants::{REQUESTED_ATTRS, REQUESTED_PREDICATES, PROOF_REJECT_RESPONSE_STR_V2};
    use utils::devsetup::*;
    use settings;

    use super::*;
    use utils::mockdata::mockdata_proof::ARIES_PROOF_PRESENTATION;
    use utils::mockdata::mock_settings::MockBuilder;

    #[test]
    #[cfg(feature = "general_test")]
    fn test_proof_validation_with_predicate() {
        let _setup = SetupAriesMocks::init();
        let _mock_builder = MockBuilder::init().
            set_mock_result_for_validate_indy_proof(Ok(true));

        let connection_handle = build_test_connection_inviter_requested();

        let mut proof = Verifier::create("1".to_string(),
                                         REQUESTED_ATTRS.to_owned(),
                                         REQUESTED_PREDICATES.to_owned(),
                                         r#"{"support_revocation":false}"#.to_string(),
                                         "Optional".to_owned()).unwrap();

        proof.send_presentation_request(connection_handle).unwrap();

        assert_eq!(proof.state(), VcxStateType::VcxStateOfferSent as u32);

        proof.update_state_with_message(ARIES_PROOF_PRESENTATION).unwrap();

        assert_eq!(proof.state(), VcxStateType::VcxStateAccepted as u32);
    }

    #[test]
    #[cfg(feature = "general_test")]
    fn test_send_presentation_request() {
        let _setup = SetupAriesMocks::init();

        let connection_handle = build_test_connection_inviter_requested();

        let mut proof = Verifier::create("1".to_string(),
                                         REQUESTED_ATTRS.to_owned(),
                                         REQUESTED_PREDICATES.to_owned(),
                                         r#"{"support_revocation":false}"#.to_string(),
                                         "Optional".to_owned()).unwrap();

        proof.send_presentation_request(connection_handle).unwrap();

        assert_eq!(proof.state(), VcxStateType::VcxStateOfferSent as u32);
    }

    #[test]
    #[cfg(feature = "general_test")]
    fn test_update_state_with_reject_message() {
        let _setup = SetupAriesMocks::init();

        let connection_handle = build_test_connection_inviter_requested();

        let mut proof = Verifier::create("1".to_string(),
                                         REQUESTED_ATTRS.to_owned(),
                                         REQUESTED_PREDICATES.to_owned(),
                                         r#"{"support_revocation":false}"#.to_string(),
                                         "Optional".to_owned()).unwrap();

        proof.send_presentation_request(connection_handle);

        proof.update_state_with_message(PROOF_REJECT_RESPONSE_STR_V2).unwrap();
        assert_eq!(proof.state(), VcxStateType::VcxStateNone as u32);
    }
}
