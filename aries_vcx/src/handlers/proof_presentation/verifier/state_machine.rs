use std::collections::HashMap;

use crate::error::prelude::*;
use crate::handlers::proof_presentation::verifier::messages::VerifierMessages;
use crate::handlers::proof_presentation::verifier::states::initial::InitialVerifierState;
use crate::handlers::proof_presentation::verifier::states::finished::FinishedState;
use crate::handlers::proof_presentation::verifier::states::presentation_request_set::PresentationRequestSet;
use crate::handlers::proof_presentation::verifier::states::presentation_request_sent::PresentationRequestSentState;
use crate::handlers::proof_presentation::verifier::states::presentation_proposal_received::PresentationProposalReceivedState;
use crate::handlers::proof_presentation::verifier::verifier::VerifierState;
use crate::handlers::proof_presentation::verifier::verify_thread_id;
use crate::messages::a2a::{A2AMessage, MessageId};
use crate::messages::error::ProblemReport;
use crate::messages::proof_presentation::presentation::Presentation;
use crate::messages::proof_presentation::presentation_proposal::PresentationProposal;
use crate::messages::proof_presentation::presentation_request::{PresentationRequest, PresentationRequestData};
use crate::messages::status::Status;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Default)]
pub struct VerifierSM {
    source_id: String,
    thread_id: String,
    state: VerifierFullState,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum VerifierFullState {
    Initial(InitialVerifierState),
    PresentationRequestSet(PresentationRequestSet),
    PresentationProposalReceived(PresentationProposalReceivedState),
    PresentationRequestSent(PresentationRequestSentState),
    Finished(FinishedState),
}

impl Default for VerifierFullState {
    fn default() -> Self {
        Self::Initial(InitialVerifierState::default())
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum RevocationStatus {
    Revoked,
    NonRevoked,
}

impl VerifierSM {
    pub fn new(source_id: &str) -> Self {
        Self {
            thread_id: String::new(),
            source_id: source_id.to_string(),
            state: VerifierFullState::Initial(InitialVerifierState {}),
        }
    }

    pub fn from_request(source_id: &str, presentation_request: PresentationRequestData) -> Self {
        Self {
            source_id: source_id.to_string(),
            thread_id: MessageId::new().0,
            state: VerifierFullState::PresentationRequestSet(PresentationRequestSet { presentation_request_data: presentation_request }),

        }
    }

    pub fn from_proposal(source_id: &str, presentation_proposal: &PresentationProposal) -> Self {
        Self {
            source_id: source_id.to_string(),
            thread_id: presentation_proposal.id.0.clone(),
            state: VerifierFullState::PresentationProposalReceived(PresentationProposalReceivedState::new(presentation_proposal.clone())),
        }
    }

    pub fn find_message_to_handle(&self, messages: HashMap<String, A2AMessage>) -> Option<(String, A2AMessage)> {
        trace!("VerifierSM::find_message_to_handle >>> messages: {:?}", messages);
        for (uid, message) in messages {
            match &self.state {
                VerifierFullState::Initial(_) => {
                    match message {
                        A2AMessage::PresentationProposal(proposal) => {
                            return Some((uid, A2AMessage::PresentationProposal(proposal)));
                        }
                        A2AMessage::PresentationRequest(request) => {
                            return Some((uid, A2AMessage::PresentationRequest(request)));
                        }
                        _ => {}
                    }
                }
                VerifierFullState::PresentationRequestSent(_) => {
                    match message {
                        A2AMessage::Presentation(presentation) => {
                            if presentation.from_thread(&self.thread_id) {
                                return Some((uid, A2AMessage::Presentation(presentation)));
                            }
                        }
                        A2AMessage::PresentationProposal(proposal) => {
                            if proposal.from_thread(&self.thread_id) {
                                return Some((uid, A2AMessage::PresentationProposal(proposal)));
                            }
                        }
                        A2AMessage::CommonProblemReport(problem_report) => {
                            if problem_report.from_thread(&self.thread_id) {
                                return Some((uid, A2AMessage::CommonProblemReport(problem_report)));
                            }
                        }
                        _ => {}
                    }
                }
                _ => {}
            };
        }
        None
    }

    pub fn step(self, message: VerifierMessages, send_message: Option<&impl Fn(&A2AMessage) -> VcxResult<()>>) -> VcxResult<Self> {
        trace!("VerifierSM::step >>> message: {:?}", message);
        let Self { source_id, state, thread_id } = self.clone();
        verify_thread_id(&thread_id, &message)?;
        let (state, thread_id) = match state {
            VerifierFullState::Initial(state) => {
                match message {
                    VerifierMessages::SetPresentationRequest(request) => {
                        (VerifierFullState::PresentationRequestSet(PresentationRequestSet::new(request)), thread_id)
                    }
                    VerifierMessages::PresentationProposalReceived(ref proposal) => {
                        let thread_id = match proposal.thread {
                            Some(ref thread) => thread.thid.clone().ok_or(VcxError::from_msg(VcxErrorKind::InvalidState, "Received proposal with invalid thid"))?,
                            None => proposal.id.0.clone()
                        };
                        (VerifierFullState::PresentationProposalReceived(PresentationProposalReceivedState::new(proposal.clone())), thread_id)
                    }
                    _ => {
                        warn!("Unable to process received message in this state");
                        (VerifierFullState::Initial(state), thread_id)
                    }
                }
            }
            VerifierFullState::PresentationRequestSet(state) => {
                match message {
                    VerifierMessages::SendPresentationRequest(comment) => {
                        let presentation_request =
                            PresentationRequest::create()
                                .set_id(thread_id.clone())
                                .set_comment(comment)
                                .set_request_presentations_attach(&state.presentation_request_data)?;
                        send_message.ok_or(
                            VcxError::from_msg(VcxErrorKind::InvalidState, "Attempted to call undefined send_message callback")
                        )?(&presentation_request.to_a2a_message())?;
                        (VerifierFullState::PresentationRequestSent(PresentationRequestSentState { presentation_request }), thread_id)
                    }
                    _ => {
                        warn!("Unable to process received message in this state");
                        (VerifierFullState::PresentationRequestSet(state), thread_id)
                    }
                }
            }
            VerifierFullState::PresentationProposalReceived(state) => {
                match message {
                    VerifierMessages::SendPresentationRequest(comment) => {
                        let presentation_request_data = match state.presentation_request_data {
                            None => return Err(VcxError::from_msg(VcxErrorKind::InvalidState, "`set_request()` must be called before sending presentation request after receiving proposal")),
                            Some(request_data) => request_data
                        };
                        let presentation_request =
                            PresentationRequest::create()
                                .set_request_presentations_attach(&presentation_request_data)?
                                .set_comment(comment)
                                .set_thread_id(&thread_id);
                        send_message.ok_or(
                            VcxError::from_msg(VcxErrorKind::InvalidState, "Attempted to call undefined send_message callback")
                        )?(&presentation_request.to_a2a_message())?;
                        (VerifierFullState::PresentationRequestSent(PresentationRequestSentState { presentation_request }), thread_id)
                    }
                    VerifierMessages::RejectPresentationProposal(reason) => {
                        let thread_id = match state.presentation_proposal.thread {
                            Some(thread) => thread.thid.ok_or(VcxError::from_msg(VcxErrorKind::InvalidState, "Thread id undefined"))?,
                            None => state.presentation_proposal.id.0
                        };
                        let problem_report = ProblemReport::create()
                            .set_comment(Some(reason.to_string()))
                            .set_thread_id(&thread_id);
                        send_message.ok_or(
                            VcxError::from_msg(VcxErrorKind::InvalidState, "Attempted to call undefined send_message callback")
                        )?(&problem_report.to_a2a_message())?;
                        (VerifierFullState::Finished(FinishedState::declined(problem_report)), thread_id)
                    }
                    _ => {
                        warn!("Unable to process received message in this state");
                        (VerifierFullState::PresentationProposalReceived(state), thread_id)
                    }
                }
            }
            VerifierFullState::PresentationRequestSent(state) => {
                match message {
                    VerifierMessages::VerifyPresentation(presentation) => {
                        match state.verify_presentation(&presentation, &thread_id, send_message) {
                            Ok(()) => {
                                (VerifierFullState::Finished((state, presentation, RevocationStatus::NonRevoked).into()), thread_id)
                            }
                            Err(err) => {
                                let problem_report =
                                    ProblemReport::create()
                                        .set_comment(Some(err.to_string()))
                                        .set_thread_id(&thread_id);
                                send_message.ok_or(
                                    VcxError::from_msg(VcxErrorKind::InvalidState, "Attempted to call undefined send_message callback")
                                )?(&problem_report.to_a2a_message())?;
                                match err.kind() {
                                    VcxErrorKind::InvalidProof => {
                                        (VerifierFullState::Finished((state, presentation, RevocationStatus::Revoked).into()), thread_id)
                                    }
                                    _ => (VerifierFullState::Finished((state, problem_report).into()), thread_id)
                                }
                            }
                        }
                    }
                    VerifierMessages::PresentationRejectReceived(problem_report) => {
                        (VerifierFullState::Finished((state, problem_report).into()), thread_id)
                    }
                    VerifierMessages::PresentationProposalReceived(proposal) => {
                        (VerifierFullState::PresentationProposalReceived(PresentationProposalReceivedState::new(proposal)), thread_id)
                    }
                    _ => {
                        warn!("Unable to process received message in this state");
                        (VerifierFullState::PresentationRequestSent(state), thread_id)
                    }
                }
            }
            VerifierFullState::Finished(state) => (VerifierFullState::Finished(state), thread_id)
        };

        Ok(Self { source_id, state, thread_id })
    }

    pub fn source_id(&self) -> String { self.source_id.clone() }

    pub fn thread_id(&self) -> String { self.thread_id.clone() }

    pub fn get_state(&self) -> VerifierState {
        match self.state {
            VerifierFullState::Initial(_) => VerifierState::Initial,
            VerifierFullState::PresentationRequestSet(_) => VerifierState::PresentationRequestSet,
            VerifierFullState::PresentationProposalReceived(_) => VerifierState::PresentationProposalReceived,
            VerifierFullState::PresentationRequestSent(_) => VerifierState::PresentationRequestSent,
            VerifierFullState::Finished(ref status) => {
                match status.status {
                    Status::Success => VerifierState::Finished,
                    _ => VerifierState::Failed
                }
            }
        }
    }

    pub fn progressable_by_message(&self) -> bool {
        match self.state {
            VerifierFullState::Initial(_) => true,
            VerifierFullState::PresentationRequestSet(_) => false,
            VerifierFullState::PresentationProposalReceived(_) => false,
            VerifierFullState::PresentationRequestSent(_) => true,
            VerifierFullState::Finished(_) => false,
        }
    }

    pub fn presentation_status(&self) -> u32 {
        match self.state {
            VerifierFullState::Finished(ref state) => {
                match &state.status {
                    Status::Success => {
                        match state.revocation_status {
                            Some(RevocationStatus::NonRevoked) => Status::Success.code(),
                            None => Status::Success.code(), // for backward compatibility
                            Some(RevocationStatus::Revoked) => {
                                let problem_report = ProblemReport::create().set_comment(Some(String::from("Revoked credential was used.")));
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
            VerifierFullState::Initial(_) => Err(VcxError::from_msg(VcxErrorKind::InvalidState, "Presentation request not set yet")),
            VerifierFullState::PresentationRequestSet(ref state) => {
                PresentationRequest::create().set_request_presentations_attach(&state.presentation_request_data)
            }
            VerifierFullState::PresentationProposalReceived(ref state) => {
                PresentationRequest::create().set_request_presentations_attach(state.presentation_request_data.as_ref().ok_or(VcxError::from_msg(VcxErrorKind::InvalidState, "No presentation request set"))?)
            }
            VerifierFullState::PresentationRequestSent(ref state) => Ok(state.presentation_request.clone()),
            VerifierFullState::Finished(ref state) => Ok(state.presentation_request.as_ref().ok_or(VcxError::from_msg(VcxErrorKind::InvalidState, "No presentation request set"))?.clone()),
        }
    }

    pub fn presentation(&self) -> VcxResult<Presentation> {
        match self.state {
            VerifierFullState::Finished(ref state) => {
                state.presentation.clone()
                    .ok_or(VcxError::from(VcxErrorKind::InvalidProofHandle))
            }
            _ => Err(VcxError::from_msg(VcxErrorKind::InvalidState, "Presentation not received yet"))
        }
    }

    pub fn presentation_proposal(&self) -> VcxResult<PresentationProposal> {
        match self.state {
            VerifierFullState::PresentationProposalReceived(ref state) => {
                Ok(state.presentation_proposal.clone())
            }
            _ => Err(VcxError::from_msg(VcxErrorKind::InvalidState, "Presentation proposal not received yet"))
        }
    }

    pub fn set_request(self, presentation_request_data: PresentationRequestData) -> VcxResult<Self> {
        let Self { state, .. } = self;
        let state = match state {
            VerifierFullState::PresentationRequestSet(_) => {
                VerifierFullState::PresentationRequestSet(PresentationRequestSet {
                    presentation_request_data
                })
            }
            VerifierFullState::PresentationProposalReceived(state) => {
                VerifierFullState::PresentationProposalReceived(PresentationProposalReceivedState {
                    presentation_request_data: Some(presentation_request_data),
                    ..state
                })
            }
            _ => { return Err(VcxError::from_msg(VcxErrorKind::InvalidState, "Cannot set presentation request in this state")) }
        };
        Ok(Self { state, ..self })
    }
}

#[cfg(test)]
pub mod test {
    use crate::messages::proof_presentation::presentation::test_utils::{_comment, _presentation, _presentation_1};
    use crate::messages::proof_presentation::presentation_proposal::test_utils::_presentation_proposal;
    use crate::messages::proof_presentation::presentation_request::test_utils::_presentation_request;
    use crate::messages::proof_presentation::presentation_request::test_utils::_presentation_request_data;
    use crate::messages::proof_presentation::test::{_ack, _problem_report};
    use crate::test::source_id;
    use crate::utils::devsetup::{SetupMocks, SetupEmpty};

    use super::*;

    pub fn _verifier_sm() -> VerifierSM {
        VerifierSM::new(&source_id())
    }

    pub fn _verifier_sm_from_request() -> VerifierSM {
        VerifierSM::from_request(&source_id(), _presentation_request_data())
    }

    pub fn _send_message() -> Option<&'static impl Fn(&A2AMessage) -> VcxResult<()>> {
        Some(&|_: &A2AMessage| VcxResult::Ok(()))
    }

    pub fn _reason() -> String {
        String::from("Unqualified")
    }

    impl VerifierSM {
        fn to_presentation_proposal_received_state(mut self) -> VerifierSM {
            self = self.step(VerifierMessages::PresentationProposalReceived(_presentation_proposal()), None::<&fn(&A2AMessage) -> _>).unwrap();
            self
        }

        fn to_presentation_proposal_received_state_with_request(mut self) -> VerifierSM {
            self = self.step(VerifierMessages::PresentationProposalReceived(_presentation_proposal()), None::<&fn(&A2AMessage) -> _>).unwrap();
            self = self.set_request(_presentation_request_data()).unwrap();
            self
        }

        fn to_presentation_request_set_state(mut self) -> VerifierSM {
            self = self.set_request(_presentation_request_data()).unwrap();
            self
        }

        fn to_presentation_request_sent_state(mut self) -> VerifierSM {
            self = self.step(VerifierMessages::SendPresentationRequest(_comment()), _send_message()).unwrap();
            self
        }

        fn to_finished_state(mut self) -> VerifierSM {
            self = self.step(VerifierMessages::SendPresentationRequest(_comment()), _send_message()).unwrap();
            self = self.step(VerifierMessages::VerifyPresentation(_presentation()), _send_message()).unwrap();
            self
        }
    }

    mod new {
        use super::*;

        #[test]
        #[cfg(feature = "general_test")]
        fn test_verifier_from_request() {
            let _setup = SetupMocks::init();

            let verifier_sm = _verifier_sm_from_request();

            assert_match!(VerifierFullState::PresentationRequestSet(_), verifier_sm.state);
            assert_eq!(source_id(), verifier_sm.source_id());
        }

        #[test]
        #[cfg(feature = "general_test")]
        fn test_verifier_new() {
            let _setup = SetupMocks::init();

            let verifier_sm = _verifier_sm();

            assert_match!(VerifierFullState::Initial(_), verifier_sm.state);
            assert_eq!(source_id(), verifier_sm.source_id());
        }
    }

    mod step {
        use crate::utils::mockdata::mock_settings::MockBuilder;

        use super::*;

        #[test]
        #[cfg(feature = "general_test")]
        fn test_verifier_init() {
            let _setup = SetupMocks::init();

            let verifier_sm = _verifier_sm_from_request();
            assert_match!(VerifierFullState::PresentationRequestSet(_), verifier_sm.state);
        }

        #[test]
        #[cfg(feature = "general_test")]
        fn test_prover_handle_set_presentation_request_message_from_initial_state() {
            let _setup = SetupMocks::init();

            let mut verifier_sm = _verifier_sm();
            verifier_sm = verifier_sm.step(VerifierMessages::SetPresentationRequest(_presentation_request_data()), _send_message()).unwrap();

            assert_match!(VerifierFullState::PresentationRequestSet(_), verifier_sm.state);
        }

        #[test]
        #[cfg(feature = "general_test")]
        fn test_prover_handle_set_presentation_proposal_received_from_initial_state() {
            let _setup = SetupMocks::init();

            let mut verifier_sm = _verifier_sm();
            verifier_sm = verifier_sm.step(VerifierMessages::PresentationProposalReceived(_presentation_proposal()), _send_message()).unwrap();

            assert_match!(VerifierFullState::PresentationProposalReceived(_), verifier_sm.state);
        }

        #[test]
        #[cfg(feature = "general_test")]
        fn test_prover_handle_send_presentation_request_message_from_presentation_request_set_state() {
            let _setup = SetupMocks::init();

            let mut verifier_sm = _verifier_sm_from_request();
            verifier_sm = verifier_sm.step(VerifierMessages::SendPresentationRequest(_comment()), _send_message()).unwrap();

            assert_match!(VerifierFullState::PresentationRequestSent(_), verifier_sm.state);
        }

        #[test]
        #[cfg(feature = "general_test")]
        fn test_prover_handle_other_messages_from_presentation_request_set_state() {
            let _setup = SetupMocks::init();

            let mut verifier_sm = _verifier_sm_from_request();

            verifier_sm = verifier_sm.step(VerifierMessages::PresentationRejectReceived(_problem_report()), _send_message()).unwrap();
            assert_match!(VerifierFullState::PresentationRequestSet(_), verifier_sm.state);

            verifier_sm = verifier_sm.step(VerifierMessages::VerifyPresentation(_presentation()), _send_message()).unwrap();
            assert_match!(VerifierFullState::PresentationRequestSet(_), verifier_sm.state);

            verifier_sm = verifier_sm.step(VerifierMessages::PresentationProposalReceived(_presentation_proposal()), _send_message()).unwrap();

            assert_match!(VerifierFullState::PresentationRequestSet(_), verifier_sm.state);
        }

        #[test]
        #[cfg(feature = "general_test")]
        fn test_prover_handle_send_presentation_request_message_from_presentation_proposal_received_state() {
            let _setup = SetupMocks::init();

            let mut verifier_sm = _verifier_sm().to_presentation_proposal_received_state_with_request();
            verifier_sm = verifier_sm.step(VerifierMessages::SendPresentationRequest(_comment()), _send_message()).unwrap();

            assert_match!(VerifierFullState::PresentationRequestSent(_), verifier_sm.state);
        }

        #[test]
        #[cfg(feature = "general_test")]
        fn test_prover_handle_send_presentation_request_from_presentation_proposal_received_state_fails_without_request() {
            let _setup = SetupMocks::init();

            let mut verifier_sm = _verifier_sm().to_presentation_proposal_received_state();
            let res = verifier_sm.step(VerifierMessages::SendPresentationRequest(_comment()), _send_message());

            assert!(res.is_err());
        }

        #[test]
        #[cfg(feature = "general_test")]
        fn test_prover_handle_reject_presentation_proposal_message_from_presentation_proposal_received_state() {
            let _setup = SetupMocks::init();

            let mut verifier_sm = _verifier_sm().to_presentation_proposal_received_state();
            verifier_sm = verifier_sm.step(VerifierMessages::RejectPresentationProposal(_reason()), _send_message()).unwrap();

            assert_match!(VerifierFullState::Finished(_), verifier_sm.state);
            assert_eq!(Status::Declined(ProblemReport::default()).code(), verifier_sm.presentation_status());
        }

        #[test]
        #[cfg(feature = "general_test")]
        fn test_prover_handle_other_messages_from_presentation_proposal_received_state() {
            let _setup = SetupMocks::init();

            let mut verifier_sm = _verifier_sm().to_presentation_proposal_received_state();

            verifier_sm = verifier_sm.step(VerifierMessages::VerifyPresentation(_presentation()), _send_message()).unwrap();
            assert_match!(VerifierFullState::PresentationProposalReceived(_), verifier_sm.state);

            verifier_sm = verifier_sm.step(VerifierMessages::PresentationRejectReceived(_problem_report()), _send_message()).unwrap();
            assert_match!(VerifierFullState::PresentationProposalReceived(_), verifier_sm.state);
        }

        #[test]
        #[cfg(feature = "general_test")]
        fn test_prover_handle_verify_presentation_message_from_presentation_request_sent_state() {
            let _setup = SetupMocks::init();
            let _mock_builder = MockBuilder::init().
                set_mock_result_for_validate_indy_proof(Ok(true));

            let mut verifier_sm = _verifier_sm_from_request();
            verifier_sm = verifier_sm.step(VerifierMessages::SendPresentationRequest(_comment()), _send_message()).unwrap();
            verifier_sm = verifier_sm.step(VerifierMessages::VerifyPresentation(_presentation()), _send_message()).unwrap();

            assert_match!(VerifierFullState::Finished(_), verifier_sm.state);
            assert_eq!(Status::Success.code(), verifier_sm.presentation_status());
        }

        #[test]
        #[cfg(feature = "general_test")]
        fn test_prover_handle_invalid_presentation_message() {
            let _setup = SetupMocks::init();
            let _mock_builder = MockBuilder::init().
                set_mock_result_for_validate_indy_proof(Ok(false));

            let mut verifier_sm = _verifier_sm_from_request();
            verifier_sm = verifier_sm.step(VerifierMessages::SendPresentationRequest(_comment()), _send_message()).unwrap();
            verifier_sm = verifier_sm.step(VerifierMessages::VerifyPresentation(_presentation()), _send_message()).unwrap();

            assert_match!(VerifierFullState::Finished(_), verifier_sm.state);
            assert_eq!(VerifierState::Finished, verifier_sm.get_state());
            assert_eq!(Status::Failed(ProblemReport::create()).code(), verifier_sm.presentation_status());
        }

        #[test]
        #[cfg(feature = "general_test")]
        fn test_prover_presentation_verification_fails_with_incorrect_thread_id() {
            let _setup = SetupEmpty::init();
            let _mock_builder = MockBuilder::init().
                set_mock_result_for_validate_indy_proof(Ok(false));

            let mut verifier_sm = _verifier_sm_from_request();
            verifier_sm = verifier_sm.step(VerifierMessages::SendPresentationRequest(_comment()), _send_message()).unwrap();
            let res = verifier_sm.clone().step(VerifierMessages::VerifyPresentation(_presentation_1()), _send_message());
            assert!(res.is_err());
        }

        //    #[test]
        //    fn test_prover_handle_verify_presentation_message_from_presentation_request_sent_state_for_invalid_presentation() {
        //        let _setup = Setup::init();
        //
        //        let mut verifier_sm = _verifier_sm_from_request();
        //        verifier_sm = verifier_sm.step(VerifierMessages::SendPresentationRequest(_comment())(mock_connection())).unwrap();
        //        verifier_sm = verifier_sm.step(VerifierMessages::VerifyPresentation(_presentation())).unwrap();
        //
        //        assert_match!(VerifierFullState::Finished(_), verifier_sm.state);
        //        assert_eq!(Status::Failed(_problem_report()).code(), verifier_sm.presentation_status());
        //    }

        #[test]
        #[cfg(feature = "general_test")]
        fn test_prover_handle_presentation_proposal_message_from_presentation_request_sent_state() {
            let _setup = SetupMocks::init();

            let mut verifier_sm = _verifier_sm_from_request();
            verifier_sm = verifier_sm.step(VerifierMessages::SendPresentationRequest(_comment()), _send_message()).unwrap();
            verifier_sm = verifier_sm.step(VerifierMessages::PresentationProposalReceived(_presentation_proposal()), _send_message()).unwrap();

            assert_match!(VerifierFullState::PresentationProposalReceived(_), verifier_sm.state);
        }

        #[test]
        #[cfg(feature = "general_test")]
        fn test_prover_handle_presentation_reject_message_from_presentation_request_sent_state() {
            let _setup = SetupMocks::init();

            let mut verifier_sm = _verifier_sm_from_request();
            verifier_sm = verifier_sm.step(VerifierMessages::SendPresentationRequest(_comment()), _send_message()).unwrap();
            verifier_sm = verifier_sm.step(VerifierMessages::PresentationRejectReceived(_problem_report()), _send_message()).unwrap();

            assert_match!(VerifierFullState::Finished(_), verifier_sm.state);
            assert_eq!(Status::Failed(_problem_report()).code(), verifier_sm.presentation_status());
        }

        #[test]
        #[cfg(feature = "general_test")]
        fn test_prover_handle_other_messages_from_presentation_request_sent_state() {
            let _setup = SetupMocks::init();

            let mut verifier_sm = _verifier_sm_from_request();
            verifier_sm = verifier_sm.step(VerifierMessages::SendPresentationRequest(_comment()), _send_message()).unwrap();
            verifier_sm = verifier_sm.step(VerifierMessages::SendPresentationRequest(_comment()), _send_message()).unwrap();

            assert_match!(VerifierFullState::PresentationRequestSent(_), verifier_sm.state);
        }

        #[test]
        #[cfg(feature = "general_test")]
        fn test_prover_handle_messages_from_presentation_finished_state() {
            let _setup = SetupMocks::init();

            let mut verifier_sm = _verifier_sm_from_request();
            verifier_sm = verifier_sm.step(VerifierMessages::SendPresentationRequest(_comment()), _send_message()).unwrap();
            verifier_sm = verifier_sm.step(VerifierMessages::VerifyPresentation(_presentation()), _send_message()).unwrap();

            verifier_sm = verifier_sm.step(VerifierMessages::PresentationRejectReceived(_problem_report()), _send_message()).unwrap();
            assert_match!(VerifierFullState::Finished(_), verifier_sm.state);

            verifier_sm = verifier_sm.step(VerifierMessages::PresentationProposalReceived(_presentation_proposal()), _send_message()).unwrap();
            assert_match!(VerifierFullState::Finished(_), verifier_sm.state);
        }
    }

    mod find_message_to_handle {
        use super::*;

        #[test]
        #[cfg(feature = "general_test")]
        fn test_verifier_find_message_to_handle_from_initial_state() {
            let _setup = SetupMocks::init();

            let verifier = _verifier_sm_from_request();

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

            let verifier = _verifier_sm_from_request().to_presentation_request_sent_state();

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

            let verifier = _verifier_sm_from_request().to_finished_state();

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
        use crate::utils::mockdata::mock_settings::MockBuilder;

        use super::*;

        #[test]
        #[cfg(feature = "general_test")]
        fn test_get_state() {
            let _setup = SetupMocks::init();
            let _mock_builder = MockBuilder::init().
                set_mock_result_for_validate_indy_proof(Ok(true));

            assert_eq!(VerifierState::PresentationRequestSet, _verifier_sm_from_request().get_state());
            assert_eq!(VerifierState::PresentationRequestSent, _verifier_sm_from_request().to_presentation_request_sent_state().get_state());
            assert_eq!(VerifierState::Finished, _verifier_sm_from_request().to_finished_state().get_state());
        }
    }
}
