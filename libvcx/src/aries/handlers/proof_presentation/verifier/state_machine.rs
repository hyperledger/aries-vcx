use std::collections::HashMap;

use crate::api::VcxStateType;
use crate::connection::{get_pw_did, get_their_pw_verkey};
use crate::{connection, settings};
use crate::error::prelude::*;
use crate::aries::handlers::proof_presentation::verifier::messages::VerifierMessages;
use crate::aries::messages::a2a::A2AMessage;
use crate::aries::messages::error::ProblemReport;
use crate::aries::messages::proof_presentation::presentation::Presentation;
use crate::aries::messages::proof_presentation::presentation_request::{PresentationRequest, PresentationRequestData};
use crate::aries::messages::status::Status;
use crate::aries::handlers::proof_presentation::verifier::states::initial::InitialState;
use crate::aries::handlers::proof_presentation::verifier::states::presentation_request_sent::PresentationRequestSentState;
use crate::aries::handlers::proof_presentation::verifier::states::finished::FinishedState;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct VerifierSM {
    source_id: String,
    state: VerifierState,
}

impl VerifierSM {
    pub fn new(presentation_request: PresentationRequestData, source_id: String) -> VerifierSM {
        VerifierSM { source_id, state: VerifierState::Initiated(InitialState { presentation_request_data: presentation_request }) }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum VerifierState {
    Initiated(InitialState),
    PresentationRequestSent(PresentationRequestSentState),
    Finished(FinishedState),
}


#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum RevocationStatus {
    Revoked,
    NonRevoked,
}

impl VerifierSM {
    pub fn find_message_to_handle(&self, messages: HashMap<String, A2AMessage>) -> Option<(String, A2AMessage)> {
        trace!("VerifierSM::find_message_to_handle >>> messages: {:?}", messages);

        for (uid, message) in messages {
            match self.state {
                VerifierState::Initiated(_) => {
                    // do not process message
                }
                VerifierState::PresentationRequestSent(_) => {
                    match message {
                        A2AMessage::Presentation(presentation) => {
                            if presentation.from_thread(&self.thread_id()) {
                                return Some((uid, A2AMessage::Presentation(presentation)));
                            }
                        }
                        A2AMessage::PresentationProposal(proposal) => {
                            if proposal.from_thread(&self.thread_id()) {
                                return Some((uid, A2AMessage::PresentationProposal(proposal)));
                            }
                        }
                        A2AMessage::CommonProblemReport(problem_report) => {
                            if problem_report.from_thread(&self.thread_id()) {
                                return Some((uid, A2AMessage::CommonProblemReport(problem_report)));
                            }
                        }
                        _ => {}
                    }
                }
                VerifierState::Finished(_) => {
                    // do not process message
                }
            };
        }

        None
    }

    pub fn step(self, message: VerifierMessages, connection_handle: u32) -> VcxResult<VerifierSM> {
        trace!("VerifierSM::step >>> message: {:?}", message);

        let VerifierSM { source_id, state } = self;

        let state = match state {
            VerifierState::Initiated(state) => {
                match message {
                    VerifierMessages::SendPresentationRequest => {
                        let my_did = get_pw_did(connection_handle)?;
                        let remote_did = get_their_pw_verkey(connection_handle)?;

                        let presentation_request: PresentationRequestData =
                            state.presentation_request_data.clone()
                                .set_format_version_for_did(&my_did, &remote_did)?;

                        let title = format!("{} wants you to share {}",
                                            settings::get_config_value(settings::CONFIG_INSTITUTION_NAME)?, presentation_request.name);

                        let presentation_request =
                            PresentationRequest::create()
                                .set_comment(title)
                                .set_request_presentations_attach(&presentation_request)?;

                        connection::send_message(connection_handle, presentation_request.to_a2a_message())?;
                        VerifierState::PresentationRequestSent((state, presentation_request).into())
                    }
                    _ => {
                        VerifierState::Initiated(state)
                    }
                }
            }
            VerifierState::PresentationRequestSent(state) => {
                match message {
                    VerifierMessages::VerifyPresentation(presentation) => {
                        match state.verify_presentation(&presentation, connection_handle) {
                            Ok(()) => {
                                VerifierState::Finished((state, presentation, RevocationStatus::NonRevoked).into())
                            }
                            Err(err) => {
                                let problem_report =
                                    ProblemReport::create()
                                        .set_comment(err.to_string())
                                        .set_thread_id(&state.presentation_request.id.0);

                                connection::send_message(connection_handle, problem_report.to_a2a_message())?;
                                match err.kind() {
                                    VcxErrorKind::InvalidProof => {
                                        VerifierState::Finished((state, presentation, RevocationStatus::Revoked).into())
                                    }
                                    _ => VerifierState::Finished((state, problem_report).into())
                                }
                            }
                        }
                    }
                    VerifierMessages::PresentationRejectReceived(problem_report) => {
                        VerifierState::Finished((state, problem_report).into())
                    }
                    VerifierMessages::PresentationProposalReceived(_) => { // TODO: handle Presentation Proposal
                        let problem_report =
                            ProblemReport::create()
                                .set_comment(String::from("PresentationProposal is not supported"))
                                .set_thread_id(&state.presentation_request.id.0);

                        connection::send_message(connection_handle, problem_report.to_a2a_message())?;
                        VerifierState::Finished((state, problem_report).into())
                    }
                    _ => {
                        VerifierState::PresentationRequestSent(state)
                    }
                }
            }
            VerifierState::Finished(state) => VerifierState::Finished(state)
        };

        Ok(VerifierSM { source_id, state })
    }

    pub fn source_id(&self) -> String { self.source_id.clone() }

    pub fn thread_id(&self) -> String { self.presentation_request().map(|request| request.id.0.clone()).unwrap_or_default() }

    pub fn state(&self) -> u32 {
        match self.state {
            VerifierState::Initiated(_) => VcxStateType::VcxStateInitialized as u32,
            VerifierState::PresentationRequestSent(_) => VcxStateType::VcxStateOfferSent as u32,
            VerifierState::Finished(ref status) => {
                match status.status {
                    Status::Success => VcxStateType::VcxStateAccepted as u32,
                    _ => VcxStateType::VcxStateNone as u32,
                }
            }
        }
    }

    pub fn has_transitions(&self) -> bool {
        match self.state {
            VerifierState::Initiated(_) => false,
            VerifierState::PresentationRequestSent(_) => true,
            VerifierState::Finished(_) => false,
        }
    }

    pub fn presentation_status(&self) -> u32 {
        match self.state {
            VerifierState::Finished(ref state) => {
                match &state.status {
                    Status::Success => {
                        match state.revocation_status {
                            Some(RevocationStatus::NonRevoked) => Status::Success.code(),
                            None => Status::Success.code(), // for backward compatibility
                            Some(RevocationStatus::Revoked) => {
                                let problem_report = ProblemReport::create().set_comment(String::from("Revoked credential was used."));
                                Status::Failed(problem_report).code()
                            }
                        }
                    }
                    _ => state.status.code(),
                }
            }
            _ => Status::Undefined.code()
        }
    }

    pub fn presentation_request(&self) -> VcxResult<PresentationRequest> {
        match self.state {
            VerifierState::Initiated(ref state) => {
                PresentationRequest::create().set_request_presentations_attach(&state.presentation_request_data)
            }
            VerifierState::PresentationRequestSent(ref state) => Ok(state.presentation_request.clone()),
            VerifierState::Finished(ref state) => Ok(state.presentation_request.clone()),
        }
    }

    pub fn presentation(&self) -> VcxResult<Presentation> {
        match self.state {
            VerifierState::Finished(ref state) => {
                state.presentation.clone()
                    .ok_or(VcxError::from(VcxErrorKind::InvalidProofHandle))
            }
            _ => Err(VcxError::from_msg(VcxErrorKind::NotReady, "Presentation is not received yet"))
        }
    }
}

#[cfg(test)]
pub mod test {
    use crate::utils::devsetup::SetupMocks;
    use crate::aries::handlers::connection::tests::mock_connection;
    use crate::aries::messages::proof_presentation::presentation::tests::_presentation;
    use crate::aries::messages::proof_presentation::presentation_proposal::tests::_presentation_proposal;
    use crate::aries::messages::proof_presentation::presentation_request::tests::_presentation_request;
    use crate::aries::messages::proof_presentation::presentation_request::tests::_presentation_request_data;
    use crate::aries::messages::proof_presentation::test::{_ack, _problem_report};
    use crate::aries::test::source_id;

    use super::*;

    pub fn _verifier_sm() -> VerifierSM {
        VerifierSM::new(_presentation_request_data(), source_id())
    }

    impl VerifierSM {
        fn to_presentation_request_sent_state(mut self) -> VerifierSM {
            self = self.step(VerifierMessages::SendPresentationRequest, mock_connection()).unwrap();
            self
        }

        fn to_finished_state(mut self) -> VerifierSM {
            let connection_handle = mock_connection();
            self = self.step(VerifierMessages::SendPresentationRequest, connection_handle).unwrap();
            self = self.step(VerifierMessages::VerifyPresentation(_presentation()), connection_handle).unwrap();
            self
        }
    }

    mod new {
        use super::*;

        #[test]
        #[cfg(feature = "general_test")]
        fn test_verifier_new() {
            let _setup = SetupMocks::init();

            let verifier_sm = _verifier_sm();

            assert_match!(VerifierState::Initiated(_), verifier_sm.state);
            assert_eq!(source_id(), verifier_sm.source_id());
        }
    }

    mod step {
        use super::*;
        use crate::utils::mockdata::mock_settings::MockBuilder;

        #[test]
        #[cfg(feature = "general_test")]
        fn test_verifier_init() {
            let _setup = SetupMocks::init();

            let verifier_sm = _verifier_sm();
            assert_match!(VerifierState::Initiated(_), verifier_sm.state);
        }

        #[test]
        #[cfg(feature = "general_test")]
        fn test_prover_handle_send_presentation_request_message_from_initiated_state() {
            let _setup = SetupMocks::init();

            let connection_handle = mock_connection();
            let mut verifier_sm = _verifier_sm();
            verifier_sm = verifier_sm.step(VerifierMessages::SendPresentationRequest, connection_handle).unwrap();

            assert_match!(VerifierState::PresentationRequestSent(_), verifier_sm.state);
        }

        #[test]
        #[cfg(feature = "general_test")]
        fn test_prover_handle_other_messages_from_initiated_state() {
            let _setup = SetupMocks::init();

            let connection_handle = mock_connection();
            let mut verifier_sm = _verifier_sm();

            verifier_sm = verifier_sm.step(VerifierMessages::PresentationRejectReceived(_problem_report()), connection_handle).unwrap();
            assert_match!(VerifierState::Initiated(_), verifier_sm.state);

            verifier_sm = verifier_sm.step(VerifierMessages::VerifyPresentation(_presentation()), connection_handle).unwrap();
            assert_match!(VerifierState::Initiated(_), verifier_sm.state);
        }

        #[test]
        #[cfg(feature = "general_test")]
        fn test_prover_handle_verify_presentation_message_from_presentation_request_sent_state() {
            let _setup = SetupMocks::init();
            let _mock_builder = MockBuilder::init().
                set_mock_result_for_validate_indy_proof(Ok(true));

            let connection_handle = mock_connection();
            let mut verifier_sm = _verifier_sm();
            verifier_sm = verifier_sm.step(VerifierMessages::SendPresentationRequest, connection_handle).unwrap();
            verifier_sm = verifier_sm.step(VerifierMessages::VerifyPresentation(_presentation()), connection_handle).unwrap();

            assert_match!(VerifierState::Finished(_), verifier_sm.state);
            assert_eq!(Status::Success.code(), verifier_sm.presentation_status());
        }

        #[test]
        #[cfg(feature = "general_test")]
        fn test_prover_handle_invalid_presentation_message() {
            let _setup = SetupMocks::init();
            let _mock_builder = MockBuilder::init().
                set_mock_result_for_validate_indy_proof(Ok(false));

            let connection_handle = mock_connection();
            let mut verifier_sm = _verifier_sm();
            verifier_sm = verifier_sm.step(VerifierMessages::SendPresentationRequest, connection_handle).unwrap();
            verifier_sm = verifier_sm.step(VerifierMessages::VerifyPresentation(_presentation()), connection_handle).unwrap();

            assert_match!(VerifierState::Finished(_), verifier_sm.state);
            assert_eq!(VcxStateType::VcxStateAccepted as u32, verifier_sm.state());
            assert_eq!(Status::Failed(ProblemReport::create()).code(), verifier_sm.presentation_status());
        }

        //    #[test]
        //    fn test_prover_handle_verify_presentation_message_from_presentation_request_sent_state_for_invalid_presentation() {
        //        let _setup = Setup::init();
        //
        //        let mut verifier_sm = _verifier_sm();
        //        verifier_sm = verifier_sm.step(VerifierMessages::SendPresentationRequest(mock_connection())).unwrap();
        //        verifier_sm = verifier_sm.step(VerifierMessages::VerifyPresentation(_presentation())).unwrap();
        //
        //        assert_match!(VerifierState::Finished(_), verifier_sm.state);
        //        assert_eq!(Status::Failed(_problem_report()).code(), verifier_sm.presentation_status());
        //    }

        #[test]
        #[cfg(feature = "general_test")]
        fn test_prover_handle_presentation_proposal_message_from_presentation_request_sent_state() {
            let _setup = SetupMocks::init();

            let connection_handle = mock_connection();
            let mut verifier_sm = _verifier_sm();
            verifier_sm = verifier_sm.step(VerifierMessages::SendPresentationRequest, connection_handle).unwrap();
            verifier_sm = verifier_sm.step(VerifierMessages::PresentationProposalReceived(_presentation_proposal()), connection_handle).unwrap();

            assert_match!(VerifierState::Finished(_), verifier_sm.state);
            assert_eq!(Status::Failed(_problem_report()).code(), verifier_sm.presentation_status());
        }

        #[test]
        #[cfg(feature = "general_test")]
        fn test_prover_handle_presentation_reject_message_from_presentation_request_sent_state() {
            let _setup = SetupMocks::init();

            let connection_handle = mock_connection();
            let mut verifier_sm = _verifier_sm();
            verifier_sm = verifier_sm.step(VerifierMessages::SendPresentationRequest, connection_handle).unwrap();
            verifier_sm = verifier_sm.step(VerifierMessages::PresentationRejectReceived(_problem_report()), connection_handle).unwrap();

            assert_match!(VerifierState::Finished(_), verifier_sm.state);
            assert_eq!(Status::Failed(_problem_report()).code(), verifier_sm.presentation_status());
        }

        #[test]
        #[cfg(feature = "general_test")]
        fn test_prover_handle_other_messages_from_presentation_request_sent_state() {
            let _setup = SetupMocks::init();

            let connection_handle = mock_connection();
            let mut verifier_sm = _verifier_sm();
            verifier_sm = verifier_sm.step(VerifierMessages::SendPresentationRequest, connection_handle).unwrap();
            verifier_sm = verifier_sm.step(VerifierMessages::SendPresentationRequest, connection_handle).unwrap();

            assert_match!(VerifierState::PresentationRequestSent(_), verifier_sm.state);
        }

        #[test]
        #[cfg(feature = "general_test")]
        fn test_prover_handle_messages_from_presentation_finished_state() {
            let _setup = SetupMocks::init();

            let connection_handle = mock_connection();
            let mut verifier_sm = _verifier_sm();
            verifier_sm = verifier_sm.step(VerifierMessages::SendPresentationRequest, connection_handle).unwrap();
            verifier_sm = verifier_sm.step(VerifierMessages::VerifyPresentation(_presentation()), connection_handle).unwrap();

            verifier_sm = verifier_sm.step(VerifierMessages::PresentationRejectReceived(_problem_report()), connection_handle).unwrap();
            assert_match!(VerifierState::Finished(_), verifier_sm.state);

            verifier_sm = verifier_sm.step(VerifierMessages::PresentationProposalReceived(_presentation_proposal()), connection_handle).unwrap();
            assert_match!(VerifierState::Finished(_), verifier_sm.state);
        }
    }

    mod find_message_to_handle {
        use super::*;

        #[test]
        #[cfg(feature = "general_test")]
        fn test_verifier_find_message_to_handle_from_initiated_state() {
            let _setup = SetupMocks::init();

            let verifier = _verifier_sm();

            // No messages
            {
                let messages = map!(
                    "key_1".to_string() => A2AMessage::PresentationProposal(_presentation_proposal()),
                    "key_2".to_string() => A2AMessage::Presentation(_presentation()),
                    "key_3".to_string() => A2AMessage::PresentationRequest(_presentation_request()),
                    "key_4".to_string() => A2AMessage::PresentationAck(_ack()),
                    "key_5".to_string() => A2AMessage::CommonProblemReport(_problem_report())
                );

                assert!(verifier.find_message_to_handle(messages).is_none());
            }
        }

        #[test]
        #[cfg(feature = "general_test")]
        fn test_verifier_find_message_to_handle_from_presentation_request_sent_state() {
            let _setup = SetupMocks::init();

            let verifier = _verifier_sm().to_presentation_request_sent_state();

            // Presentation
            {
                let messages = map!(
                    "key_1".to_string() => A2AMessage::PresentationRequest(_presentation_request()),
                    "key_2".to_string() => A2AMessage::Presentation(_presentation()),
                    "key_3".to_string() => A2AMessage::PresentationAck(_ack())
                );

                let (uid, message) = verifier.find_message_to_handle(messages).unwrap();
                assert_eq!("key_2", uid);
                assert_match!(A2AMessage::Presentation(_), message);
            }

            // Presentation Proposal
            {
                let messages = map!(
                    "key_1".to_string() => A2AMessage::PresentationRequest(_presentation_request()),
                    "key_2".to_string() => A2AMessage::PresentationProposal(_presentation_proposal()),
                    "key_3".to_string() => A2AMessage::PresentationAck(_ack())
                );

                let (uid, message) = verifier.find_message_to_handle(messages).unwrap();
                assert_eq!("key_2", uid);
                assert_match!(A2AMessage::PresentationProposal(_), message);
            }

            // Problem Report
            {
                let messages = map!(
                    "key_1".to_string() => A2AMessage::PresentationRequest(_presentation_request()),
                    "key_2".to_string() => A2AMessage::PresentationAck(_ack()),
                    "key_3".to_string() => A2AMessage::CommonProblemReport(_problem_report())
                );

                let (uid, message) = verifier.find_message_to_handle(messages).unwrap();
                assert_eq!("key_3", uid);
                assert_match!(A2AMessage::CommonProblemReport(_), message);
            }

            // No messages for different Thread ID
            {
                let messages = map!(
                    "key_1".to_string() => A2AMessage::PresentationProposal(_presentation_proposal().set_thread_id("")),
                    "key_2".to_string() => A2AMessage::Presentation(_presentation().set_thread_id("")),
                    "key_3".to_string() => A2AMessage::PresentationAck(_ack().set_thread_id("")),
                    "key_4".to_string() => A2AMessage::CommonProblemReport(_problem_report().set_thread_id(""))
                );

                assert!(verifier.find_message_to_handle(messages).is_none());
            }

            // No messages
            {
                let messages = map!(
                    "key_1".to_string() => A2AMessage::PresentationRequest(_presentation_request())
                );

                assert!(verifier.find_message_to_handle(messages).is_none());
            }
        }

        #[test]
        #[cfg(feature = "general_test")]
        fn test_verifier_find_message_to_handle_from_finished_state() {
            let _setup = SetupMocks::init();

            let verifier = _verifier_sm().to_finished_state();

            // No messages
            {
                let messages = map!(
                    "key_1".to_string() => A2AMessage::PresentationProposal(_presentation_proposal()),
                    "key_2".to_string() => A2AMessage::Presentation(_presentation()),
                    "key_3".to_string() => A2AMessage::PresentationRequest(_presentation_request()),
                    "key_4".to_string() => A2AMessage::PresentationAck(_ack()),
                    "key_5".to_string() => A2AMessage::CommonProblemReport(_problem_report())
                );

                assert!(verifier.find_message_to_handle(messages).is_none());
            }
        }
    }

    mod get_state {
        use super::*;
        use crate::utils::mockdata::mock_settings::MockBuilder;

        #[test]
        #[cfg(feature = "general_test")]
        fn test_get_state() {
            let _setup = SetupMocks::init();
            let _mock_builder = MockBuilder::init().
                set_mock_result_for_validate_indy_proof(Ok(true));

            assert_eq!(VcxStateType::VcxStateInitialized as u32, _verifier_sm().state());
            assert_eq!(VcxStateType::VcxStateOfferSent as u32, _verifier_sm().to_presentation_request_sent_state().state());
            assert_eq!(VcxStateType::VcxStateAccepted as u32, _verifier_sm().to_finished_state().state());
        }
    }
}
