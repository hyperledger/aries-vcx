use std::collections::HashMap;
use std::fmt::Display;
use std::sync::Arc;

use crate::common::proofs::proof_request::PresentationRequestData;
use crate::core::profile::profile::Profile;
use crate::errors::error::prelude::*;
use crate::handlers::util::{make_attach_from_str, matches_opt_thread_id, matches_thread_id, Status};
use crate::protocols::common::build_problem_report_msg;
use crate::protocols::proof_presentation::verifier::messages::VerifierMessages;
use crate::protocols::proof_presentation::verifier::states::finished::FinishedState;
use crate::protocols::proof_presentation::verifier::states::initial::InitialVerifierState;
use crate::protocols::proof_presentation::verifier::states::presentation_proposal_received::PresentationProposalReceivedState;
use crate::protocols::proof_presentation::verifier::states::presentation_request_sent::PresentationRequestSentState;
use crate::protocols::proof_presentation::verifier::states::presentation_request_set::PresentationRequestSetState;
use crate::protocols::proof_presentation::verifier::verification_status::PresentationVerificationStatus;
use crate::protocols::proof_presentation::verifier::verify_thread_id;
use crate::protocols::SendClosure;
use chrono::Utc;
use messages2::decorators::thread::Thread;
use messages2::decorators::timing::Timing;
use messages2::msg_fields::protocols::notification::{AckDecorators, AckStatus};
use messages2::msg_fields::protocols::present_proof::ack::{AckPresentation, AckPresentationContent};
use messages2::msg_fields::protocols::present_proof::request::{
    RequestPresentation, RequestPresentationContent, RequestPresentationDecorators,
};
use messages2::msg_fields::protocols::present_proof::PresentProof;
use messages2::msg_fields::protocols::present_proof::{present::Presentation, propose::ProposePresentation};
use messages2::msg_fields::protocols::report_problem::ProblemReport;
use messages2::AriesMessage;
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Default)]
pub struct VerifierSM {
    source_id: String,
    thread_id: String,
    state: VerifierFullState,
}

#[derive(Debug, PartialEq, Eq)]
pub enum VerifierState {
    Initial,
    PresentationProposalReceived,
    PresentationRequestSet,
    PresentationRequestSent,
    Finished,
    Failed,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum VerifierFullState {
    Initial(InitialVerifierState),
    PresentationRequestSet(PresentationRequestSetState),
    PresentationProposalReceived(PresentationProposalReceivedState),
    PresentationRequestSent(PresentationRequestSentState),
    Finished(FinishedState),
}

impl Display for VerifierFullState {
    fn fmt(&self, f: &mut ::std::fmt::Formatter) -> ::std::result::Result<(), ::std::fmt::Error> {
        match *self {
            VerifierFullState::Initial(_) => f.write_str("Initial"),
            VerifierFullState::PresentationRequestSet(_) => f.write_str("PresentationRequestSet"),
            VerifierFullState::PresentationProposalReceived(_) => f.write_str("PresentationProposalReceived"),
            VerifierFullState::PresentationRequestSent(_) => f.write_str("PresentationRequestSent"),
            VerifierFullState::Finished(_) => f.write_str("Finished"),
        }
    }
}

impl Default for VerifierFullState {
    fn default() -> Self {
        Self::Initial(InitialVerifierState::default())
    }
}

fn build_verification_ack(thread_id: &str) -> AckPresentation {
    let content = AckPresentationContent::new(AckStatus::Ok);
    let mut decorators = AckDecorators::new(Thread::new(thread_id.to_owned()));
    let mut timing = Timing::default();
    timing.out_time = Some(Utc::now());
    decorators.timing = Some(timing);

    AckPresentation::with_decorators(Uuid::new_v4().to_string(), content, decorators)
}

fn build_starting_presentation_request(
    thread_id: &str,
    request_data: &PresentationRequestData,
    comment: Option<String>,
) -> VcxResult<RequestPresentation> {
    let id = Uuid::new_v4().to_string();
    let content = RequestPresentationContent::new(vec![make_attach_from_str!(&json!(request_data).to_string())]);
    let mut decorators = RequestPresentationDecorators::default();
    let mut timing = Timing::default();
    timing.out_time = Some(Utc::now());
    decorators.timing = Some(timing);

    Ok(RequestPresentation::with_decorators(id, content, decorators))
}

impl VerifierSM {
    pub fn new(source_id: &str) -> Self {
        Self {
            thread_id: String::new(),
            source_id: source_id.to_string(),
            state: VerifierFullState::Initial(InitialVerifierState {}),
        }
    }

    // todo: eliminate VcxResult (follow set_request err chain and eliminate possibility of err at the bottom)
    pub fn from_request(source_id: &str, presentation_request_data: &PresentationRequestData) -> VcxResult<Self> {
        let sm = Self {
            source_id: source_id.to_string(),
            thread_id: Uuid::new_v4().to_string(),
            state: VerifierFullState::Initial(InitialVerifierState {}),
        };
        sm.set_request(presentation_request_data, None)
    }

    pub fn from_proposal(source_id: &str, presentation_proposal: &ProposePresentation) -> Self {
        Self {
            source_id: source_id.to_string(),
            thread_id: presentation_proposal.id.clone(),
            state: VerifierFullState::PresentationProposalReceived(PresentationProposalReceivedState::new(
                presentation_proposal.clone(),
            )),
        }
    }

    pub fn receive_presentation_proposal(self, proposal: ProposePresentation) -> VcxResult<Self> {
        verify_thread_id(
            &self.thread_id,
            &VerifierMessages::PresentationProposalReceived(proposal.clone()),
        )?;
        let (state, thread_id) = match self.state {
            VerifierFullState::Initial(_) => {
                let thread_id = match proposal.decorators.thread {
                    Some(ref thread) => thread.thid.clone(),
                    None => proposal.id.clone(),
                };
                (
                    VerifierFullState::PresentationProposalReceived(PresentationProposalReceivedState::new(proposal)),
                    thread_id,
                )
            }
            VerifierFullState::PresentationRequestSent(_) => {
                verify_thread_id(
                    &self.thread_id,
                    &VerifierMessages::PresentationProposalReceived(proposal.clone()),
                )?;
                (
                    VerifierFullState::PresentationProposalReceived(PresentationProposalReceivedState::new(proposal)),
                    self.thread_id.clone(),
                )
            }
            s => {
                warn!("Unable to receive presentation proposal in state {}", s);
                (s, self.thread_id.clone())
            }
        };
        Ok(Self {
            state,
            thread_id,
            ..self
        })
    }

    pub fn receive_presentation_request_reject(self, problem_report: ProblemReport) -> VcxResult<Self> {
        verify_thread_id(
            &self.thread_id,
            &VerifierMessages::PresentationRejectReceived(problem_report.clone()),
        )?;
        let state = match self.state {
            VerifierFullState::PresentationRequestSent(state) => {
                VerifierFullState::Finished((state, problem_report).into())
            }
            s => {
                warn!("Unable to receive presentation request reject in state {}", s);
                s
            }
        };
        Ok(Self { state, ..self })
    }

    pub async fn reject_presentation_proposal(self, reason: String, send_message: SendClosure) -> VcxResult<Self> {
        let (state, thread_id) = match self.state {
            VerifierFullState::PresentationProposalReceived(state) => {
                let thread_id = match state.presentation_proposal.decorators.thread {
                    Some(thread) => thread.thid,
                    None => state.presentation_proposal.id,
                };
                let problem_report = build_problem_report_msg(Some(reason.to_string()), &thread_id);
                send_message(problem_report.clone().into()).await?;
                (
                    VerifierFullState::Finished(FinishedState::declined(problem_report)),
                    thread_id,
                )
            }
            s => {
                warn!("Unable to reject presentation proposal in state {}", s);
                (s, self.thread_id.clone())
            }
        };
        Ok(Self {
            state,
            thread_id,
            ..self
        })
    }

    pub async fn verify_presentation(
        self,
        profile: &Arc<dyn Profile>,
        presentation: Presentation,
        send_message: SendClosure,
    ) -> VcxResult<Self> {
        verify_thread_id(
            &self.thread_id,
            &VerifierMessages::VerifyPresentation(presentation.clone()),
        )?;
        let state = match self.state {
            VerifierFullState::PresentationRequestSent(state) => {
                let verification_result = state.verify_presentation(profile, &presentation, &self.thread_id).await;
                let ack = build_verification_ack(&self.thread_id);
                send_message(ack.into()).await?;
                match verification_result {
                    Ok(()) => {
                        VerifierFullState::Finished((state, presentation, PresentationVerificationStatus::Valid).into())
                    }
                    Err(err) => match err.kind() {
                        AriesVcxErrorKind::InvalidProof => VerifierFullState::Finished(
                            (state, presentation, PresentationVerificationStatus::Invalid).into(),
                        ),
                        _ => {
                            let problem_report = build_problem_report_msg(Some(err.to_string()), &self.thread_id);
                            VerifierFullState::Finished((state, problem_report).into())
                        }
                    },
                }
            }
            s => {
                warn!("Unable to verify presentation in state {}", s);
                s
            }
        };
        Ok(Self { state, ..self })
    }

    pub async fn send_presentation_ack(self, send_message: SendClosure) -> VcxResult<Self> {
        let state = match self.state {
            VerifierFullState::Finished(state) => {
                let ack = build_verification_ack(&self.thread_id);
                send_message(ack.into()).await?;
                VerifierFullState::Finished(state)
            }
            s => {
                warn!("Unable to send presentation ack in state {}", s);
                s
            }
        };
        Ok(Self { state, ..self })
    }

    pub fn find_message_to_handle(&self, messages: HashMap<String, AriesMessage>) -> Option<(String, AriesMessage)> {
        trace!("VerifierSM::find_message_to_handle >>> messages: {:?}", messages);
        for (uid, message) in messages {
            match &self.state {
                VerifierFullState::Initial(_) => match &message {
                    AriesMessage::PresentProof(PresentProof::ProposePresentation(proposal)) => {
                        return Some((uid, message));
                    }
                    AriesMessage::PresentProof(PresentProof::ProposePresentation(request)) => {
                        return Some((uid, message));
                    }
                    _ => {}
                },
                VerifierFullState::PresentationRequestSent(_) => match message {
                    AriesMessage::PresentProof(PresentProof::Presentation(presentation)) => {
                        if matches_thread_id!(presentation, self.thread_id.as_str()) {
                            return Some((uid, message));
                        }
                    }
                    AriesMessage::PresentProof(PresentProof::ProposePresentation(proposal)) => {
                        if matches_opt_thread_id!(proposal, self.thread_id.as_str()) {
                            return Some((uid, message));
                        }
                    }
                    AriesMessage::ReportProblem(problem_report) => {
                        if matches_opt_thread_id!(problem_report, self.thread_id.as_str()) {
                            return Some((uid, message));
                        }
                    }
                    _ => {}
                },
                _ => {}
            };
        }
        None
    }

    pub fn set_request(self, request_data: &PresentationRequestData, comment: Option<String>) -> VcxResult<Self> {
        let Self {
            source_id,
            thread_id,
            state,
        } = self;
        let state = match state {
            VerifierFullState::Initial(_)
            | VerifierFullState::PresentationRequestSet(_)
            | VerifierFullState::PresentationProposalReceived(_) => {
                let presentation_request = build_starting_presentation_request(&thread_id, request_data, comment)?;
                VerifierFullState::PresentationRequestSet(PresentationRequestSetState::new(presentation_request))
            }
            _ => {
                return Err(AriesVcxError::from_msg(
                    AriesVcxErrorKind::InvalidState,
                    "Cannot set presentation request in this state",
                ));
            }
        };
        Ok(Self {
            source_id,
            state,
            thread_id,
        })
    }

    pub fn mark_presentation_request_msg_sent(self) -> VcxResult<Self> {
        let Self {
            state,
            source_id,
            thread_id,
        } = self;
        let state = match state {
            VerifierFullState::PresentationRequestSet(state) => {
                VerifierFullState::PresentationRequestSent(state.into())
            }
            VerifierFullState::PresentationRequestSent(state) => VerifierFullState::PresentationRequestSent(state),
            _ => {
                return Err(AriesVcxError::from_msg(
                    AriesVcxErrorKind::InvalidState,
                    "Can not mark_presentation_request_msg_sent in current state.",
                ))
            }
        };
        Ok(Self {
            source_id,
            thread_id,
            state,
        })
    }

    pub async fn step(
        self,
        profile: &Arc<dyn Profile>,
        message: VerifierMessages,
        send_message: Option<SendClosure>,
    ) -> VcxResult<Self> {
        trace!("VerifierSM::step >>> message: {:?}", message);
        let verifier_sm = match message {
            VerifierMessages::PresentationProposalReceived(proposal) => self.receive_presentation_proposal(proposal)?,
            VerifierMessages::RejectPresentationProposal(reason) => {
                let send_message = send_message.ok_or(AriesVcxError::from_msg(
                    AriesVcxErrorKind::InvalidState,
                    "Attempted to call undefined send_message callback",
                ))?;
                self.reject_presentation_proposal(reason, send_message).await?
            }
            VerifierMessages::VerifyPresentation(presentation) => {
                let send_message = send_message.ok_or(AriesVcxError::from_msg(
                    AriesVcxErrorKind::InvalidState,
                    "Attempted to call undefined send_message callback",
                ))?;
                self.verify_presentation(profile, presentation, send_message).await?
            }
            VerifierMessages::SendPresentationAck() => {
                let send_message = send_message.ok_or(AriesVcxError::from_msg(
                    AriesVcxErrorKind::InvalidState,
                    "Attempted to call undefined send_message callback",
                ))?;
                self.send_presentation_ack(send_message).await?
            }
            // TODO: Rename to PresentationRequestRejectReceived
            VerifierMessages::PresentationRejectReceived(problem_report) => {
                self.receive_presentation_request_reject(problem_report)?
            }
            // TODO: This code path is not used currently; would need to convert ProofRequest to
            // ProofRequestData
            VerifierMessages::SetPresentationRequest(_) | VerifierMessages::Unknown => self,
        };
        Ok(verifier_sm)
    }

    pub fn source_id(&self) -> String {
        self.source_id.clone()
    }

    pub fn thread_id(&self) -> String {
        self.thread_id.clone()
    }

    pub fn get_state(&self) -> VerifierState {
        warn!("get_state >>> {:?}", self.state);
        match self.state {
            VerifierFullState::Initial(_) => VerifierState::Initial,
            VerifierFullState::PresentationRequestSet(_) => VerifierState::PresentationRequestSet,
            VerifierFullState::PresentationProposalReceived(_) => VerifierState::PresentationProposalReceived,
            VerifierFullState::PresentationRequestSent(_) => VerifierState::PresentationRequestSent,
            VerifierFullState::Finished(ref status) => match status.status {
                Status::Success => VerifierState::Finished,
                _ => VerifierState::Failed,
            },
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

    pub fn get_verification_status(&self) -> PresentationVerificationStatus {
        match self.state {
            VerifierFullState::Finished(ref state) => state.verification_status.clone(),
            _ => PresentationVerificationStatus::Unavailable,
        }
    }

    pub fn presentation_request_msg(&self) -> VcxResult<RequestPresentation> {
        match self.state {
            VerifierFullState::Initial(_) => Err(AriesVcxError::from_msg(
                AriesVcxErrorKind::InvalidState,
                "Presentation request not set yet",
            )),
            VerifierFullState::PresentationRequestSet(ref state) => Ok(state.presentation_request.clone()),
            VerifierFullState::PresentationProposalReceived(ref state) => state.presentation_request.clone().ok_or(
                AriesVcxError::from_msg(AriesVcxErrorKind::InvalidState, "No presentation request set"),
            ),
            VerifierFullState::PresentationRequestSent(ref state) => Ok(state.presentation_request.clone()),
            VerifierFullState::Finished(ref state) => Ok(state
                .presentation_request
                .as_ref()
                .ok_or(AriesVcxError::from_msg(
                    AriesVcxErrorKind::InvalidState,
                    "No presentation request set",
                ))?
                .clone()),
        }
    }

    pub fn get_presentation_msg(&self) -> VcxResult<Presentation> {
        match self.state {
            VerifierFullState::Finished(ref state) => state.presentation.clone().ok_or(AriesVcxError::from_msg(
                AriesVcxErrorKind::InvalidState,
                "State machine is final state, but presentation is not available".to_string(),
            )),
            _ => Err(AriesVcxError::from_msg(
                AriesVcxErrorKind::InvalidState,
                "Presentation not received yet",
            )),
        }
    }

    pub fn presentation_proposal(&self) -> VcxResult<ProposePresentation> {
        match self.state {
            VerifierFullState::PresentationProposalReceived(ref state) => Ok(state.presentation_proposal.clone()),
            _ => Err(AriesVcxError::from_msg(
                AriesVcxErrorKind::InvalidState,
                "Presentation proposal not received yet",
            )),
        }
    }
}

#[cfg(test)]
#[cfg(feature = "general_test")]
pub mod unit_tests {
    use crate::common::proofs::proof_request::test_utils::_presentation_request_data;
    use crate::common::test_utils::mock_profile;
    use crate::test::source_id;
    use crate::utils::devsetup::{SetupEmpty, SetupMocks};
    use messages::protocols::proof_presentation::presentation::test_utils::{_presentation, _presentation_1};
    use messages::protocols::proof_presentation::presentation_proposal::test_utils::_presentation_proposal;
    use messages::protocols::proof_presentation::presentation_request::test_utils::_presentation_request;
    use messages::protocols::proof_presentation::test_utils::{_ack, _problem_report};

    use super::*;

    pub fn _verifier_sm() -> VerifierSM {
        VerifierSM::new(&source_id())
    }

    pub fn _verifier_sm_from_request() -> VerifierSM {
        VerifierSM::from_request(&source_id(), &_presentation_request_data()).unwrap()
    }

    pub fn _send_message() -> Option<SendClosure> {
        Some(Box::new(|_: A2AMessage| Box::pin(async { VcxResult::Ok(()) })))
    }

    pub fn _reason() -> String {
        String::from("Unqualified")
    }

    impl VerifierSM {
        async fn to_presentation_proposal_received_state(mut self) -> VerifierSM {
            self = self
                .step(
                    &mock_profile(),
                    VerifierMessages::PresentationProposalReceived(_presentation_proposal()),
                    None,
                )
                .await
                .unwrap();
            self
        }

        async fn to_presentation_proposal_received_state_with_request(mut self) -> VerifierSM {
            self = self
                .step(
                    &mock_profile(),
                    VerifierMessages::PresentationProposalReceived(_presentation_proposal()),
                    None,
                )
                .await
                .unwrap();
            self = self
                .set_request(&_presentation_request_data(), Some("foo".into()))
                .unwrap();
            self
        }

        fn to_presentation_request_sent_state(mut self) -> VerifierSM {
            self = self.mark_presentation_request_msg_sent().unwrap();
            self
        }

        async fn to_finished_state(mut self) -> VerifierSM {
            self = self.to_presentation_request_sent_state();
            self = self
                .step(
                    &mock_profile(),
                    VerifierMessages::VerifyPresentation(_presentation()),
                    _send_message(),
                )
                .await
                .unwrap();
            self
        }
    }

    mod build_messages {
        use super::*;

        use crate::protocols::proof_presentation::verifier::state_machine::{
            build_starting_presentation_request, build_verification_ack,
        };
        use crate::utils::devsetup::{was_in_past, SetupMocks};
        use messages::a2a::MessageId;

        #[test]
        #[cfg(feature = "general_test")]
        fn test_verifier_build_verification_ack() {
            let _setup = SetupMocks::init();

            let msg = build_verification_ack("12345");

            assert_eq!(msg.id, MessageId::default()); // todo: it should generate random uuid even in test
            assert_eq!(msg.thread.thid, Some("12345".into()));
            assert!(was_in_past(
                &msg.timing.unwrap().out_time.unwrap(),
                chrono::Duration::milliseconds(100)
            )
            .unwrap());
        }

        #[tokio::test]
        #[cfg(feature = "general_test")]
        async fn test_verifier_build_presentation_request() {
            let _setup = SetupMocks::init();

            let presentation_request_data = PresentationRequestData::create(&mock_profile(), "1").await.unwrap();
            let msg = build_starting_presentation_request("12345", &presentation_request_data, Some("foobar".into()))
                .unwrap();

            assert_eq!(msg.id, MessageId("12345".into()));
            assert!(msg.thread.is_none());
            assert_eq!(msg.comment, Some("foobar".into()));
            assert!(was_in_past(
                &msg.timing.unwrap().out_time.unwrap(),
                chrono::Duration::milliseconds(100)
            )
            .unwrap());
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
        use crate::utils::devsetup::was_in_past;
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
            verifier_sm = verifier_sm.set_request(&_presentation_request_data(), None).unwrap();

            assert_match!(VerifierFullState::PresentationRequestSet(_), verifier_sm.state);
        }

        #[test]
        #[cfg(feature = "general_test")]
        fn test_presentation_request_should_have_set_timing() {
            let _setup = SetupMocks::init();

            let mut verifier_sm = _verifier_sm();
            verifier_sm = verifier_sm.set_request(&_presentation_request_data(), None).unwrap();

            assert_match!(VerifierFullState::PresentationRequestSet(_), verifier_sm.state);

            let msg_presentation_request = verifier_sm.presentation_request_msg().unwrap();
            let out_time = msg_presentation_request.timing.unwrap().out_time.unwrap();
            assert!(was_in_past(&out_time, chrono::Duration::milliseconds(100)).unwrap());
        }

        #[tokio::test]
        #[cfg(feature = "general_test")]
        async fn test_prover_handle_set_presentation_proposal_received_from_initial_state() {
            let _setup = SetupMocks::init();

            let mut verifier_sm = _verifier_sm();
            verifier_sm = verifier_sm
                .step(
                    &mock_profile(),
                    VerifierMessages::PresentationProposalReceived(_presentation_proposal()),
                    _send_message(),
                )
                .await
                .unwrap();

            assert_match!(VerifierFullState::PresentationProposalReceived(_), verifier_sm.state);
        }

        #[test]
        #[cfg(feature = "general_test")]
        fn test_prover_handle_send_presentation_request_message_from_presentation_request_set_state() {
            let _setup = SetupMocks::init();

            let mut verifier_sm = _verifier_sm_from_request();
            verifier_sm = verifier_sm.mark_presentation_request_msg_sent().unwrap();

            assert_match!(VerifierFullState::PresentationRequestSent(_), verifier_sm.state);
        }

        #[tokio::test]
        #[cfg(feature = "general_test")]
        async fn test_prover_handle_other_messages_from_presentation_request_set_state() {
            let _setup = SetupMocks::init();

            let mut verifier_sm = _verifier_sm_from_request();

            verifier_sm = verifier_sm
                .step(
                    &mock_profile(),
                    VerifierMessages::PresentationRejectReceived(_problem_report()),
                    _send_message(),
                )
                .await
                .unwrap();
            assert_match!(VerifierFullState::PresentationRequestSet(_), verifier_sm.state);

            verifier_sm = verifier_sm
                .step(
                    &mock_profile(),
                    VerifierMessages::VerifyPresentation(_presentation()),
                    _send_message(),
                )
                .await
                .unwrap();
            assert_match!(VerifierFullState::PresentationRequestSet(_), verifier_sm.state);

            verifier_sm = verifier_sm
                .step(
                    &mock_profile(),
                    VerifierMessages::PresentationProposalReceived(_presentation_proposal()),
                    _send_message(),
                )
                .await
                .unwrap();

            assert_match!(VerifierFullState::PresentationRequestSet(_), verifier_sm.state);
        }

        #[tokio::test]
        #[cfg(feature = "general_test")]
        async fn test_prover_handle_send_presentation_request_message_from_presentation_proposal_received_state() {
            let _setup = SetupMocks::init();

            let mut verifier_sm = _verifier_sm()
                .to_presentation_proposal_received_state_with_request()
                .await;
            verifier_sm = verifier_sm.mark_presentation_request_msg_sent().unwrap();

            assert_match!(VerifierFullState::PresentationRequestSent(_), verifier_sm.state);
        }

        #[tokio::test]
        #[cfg(feature = "general_test")]
        async fn test_prover_handle_send_presentation_request_from_presentation_proposal_received_state_fails_without_request(
        ) {
            let _setup = SetupMocks::init();

            let verifier_sm = _verifier_sm().to_presentation_proposal_received_state().await;
            let res = verifier_sm.mark_presentation_request_msg_sent();

            assert!(res.is_err());
        }

        #[tokio::test]
        #[cfg(feature = "general_test")]
        async fn test_prover_handle_reject_presentation_proposal_message_from_presentation_proposal_received_state() {
            let _setup = SetupMocks::init();

            let mut verifier_sm = _verifier_sm().to_presentation_proposal_received_state().await;
            verifier_sm = verifier_sm
                .step(
                    &mock_profile(),
                    VerifierMessages::RejectPresentationProposal(_reason()),
                    _send_message(),
                )
                .await
                .unwrap();

            assert_match!(VerifierState::Failed, verifier_sm.get_state());
            assert_match!(
                PresentationVerificationStatus::Unavailable,
                verifier_sm.get_verification_status()
            );
        }

        #[tokio::test]
        #[cfg(feature = "general_test")]
        async fn test_prover_handle_other_messages_from_presentation_proposal_received_state() {
            let _setup = SetupMocks::init();

            let mut verifier_sm = _verifier_sm().to_presentation_proposal_received_state().await;
            assert_match!(VerifierFullState::PresentationProposalReceived(_), verifier_sm.state);

            verifier_sm = verifier_sm
                .step(
                    &mock_profile(),
                    VerifierMessages::VerifyPresentation(_presentation()),
                    _send_message(),
                )
                .await
                .unwrap();
            assert_match!(VerifierFullState::PresentationProposalReceived(_), verifier_sm.state);

            verifier_sm = verifier_sm
                .step(
                    &mock_profile(),
                    VerifierMessages::PresentationRejectReceived(_problem_report()),
                    _send_message(),
                )
                .await
                .unwrap();
            assert_match!(VerifierFullState::PresentationProposalReceived(_), verifier_sm.state);
        }

        #[tokio::test]
        #[cfg(feature = "general_test")]
        async fn test_prover_handle_verify_presentation_message_from_presentation_request_sent_state() {
            let _setup = SetupMocks::init();
            let _mock_builder = MockBuilder::init().set_mock_result_for_validate_indy_proof(Ok(true));

            let mut verifier_sm = _verifier_sm_from_request();
            verifier_sm = verifier_sm.mark_presentation_request_msg_sent().unwrap();
            verifier_sm = verifier_sm
                .step(
                    &mock_profile(),
                    VerifierMessages::VerifyPresentation(_presentation()),
                    _send_message(),
                )
                .await
                .unwrap();

            assert_match!(VerifierFullState::Finished(_), verifier_sm.state);
            assert_eq!(
                PresentationVerificationStatus::Valid,
                verifier_sm.get_verification_status()
            );
        }

        #[tokio::test]
        #[cfg(feature = "general_test")]
        async fn test_prover_handle_invalid_presentation_message() {
            let _setup = SetupMocks::init();
            let _mock_builder = MockBuilder::init().set_mock_result_for_validate_indy_proof(Ok(false));

            let mut verifier_sm = _verifier_sm_from_request();
            verifier_sm = verifier_sm.mark_presentation_request_msg_sent().unwrap();
            verifier_sm = verifier_sm
                .step(
                    &mock_profile(),
                    VerifierMessages::VerifyPresentation(_presentation()),
                    _send_message(),
                )
                .await
                .unwrap();

            assert_match!(VerifierState::Finished, verifier_sm.get_state());
            assert_match!(
                PresentationVerificationStatus::Invalid,
                verifier_sm.get_verification_status()
            );
        }

        #[tokio::test]
        #[cfg(feature = "general_test")]
        async fn test_prover_presentation_verification_fails_with_incorrect_thread_id() {
            let _setup = SetupEmpty::init();
            let _mock_builder = MockBuilder::init().set_mock_result_for_validate_indy_proof(Ok(false));

            let mut verifier_sm = _verifier_sm_from_request();
            verifier_sm = verifier_sm.mark_presentation_request_msg_sent().unwrap();
            let res = verifier_sm
                .clone()
                .step(
                    &mock_profile(),
                    VerifierMessages::VerifyPresentation(_presentation_1()),
                    _send_message(),
                )
                .await;
            assert!(res.is_err());
        }

        #[tokio::test]
        #[cfg(feature = "general_test")]
        async fn test_prover_handle_presentation_proposal_message_from_presentation_request_sent_state() {
            let _setup = SetupMocks::init();

            let mut verifier_sm = _verifier_sm_from_request();
            verifier_sm = verifier_sm.mark_presentation_request_msg_sent().unwrap();
            verifier_sm = verifier_sm
                .step(
                    &mock_profile(),
                    VerifierMessages::PresentationProposalReceived(_presentation_proposal()),
                    _send_message(),
                )
                .await
                .unwrap();

            assert_match!(VerifierFullState::PresentationProposalReceived(_), verifier_sm.state);
        }

        #[tokio::test]
        #[cfg(feature = "general_test")]
        async fn test_prover_handle_presentation_reject_message_from_presentation_request_sent_state() {
            let _setup = SetupMocks::init();

            let mut verifier_sm = _verifier_sm_from_request();
            verifier_sm = verifier_sm.mark_presentation_request_msg_sent().unwrap();
            verifier_sm = verifier_sm
                .step(
                    &mock_profile(),
                    VerifierMessages::PresentationRejectReceived(_problem_report()),
                    _send_message(),
                )
                .await
                .unwrap();

            assert_match!(VerifierState::Failed, verifier_sm.get_state());
        }

        #[tokio::test]
        #[cfg(feature = "general_test")]
        async fn test_prover_handle_messages_from_presentation_finished_state() {
            let _setup = SetupMocks::init();
            let _mock_builder = MockBuilder::init().set_mock_result_for_validate_indy_proof(Ok(true));

            let mut verifier_sm = _verifier_sm_from_request();
            verifier_sm = verifier_sm.mark_presentation_request_msg_sent().unwrap();
            verifier_sm = verifier_sm
                .step(
                    &mock_profile(),
                    VerifierMessages::VerifyPresentation(_presentation()),
                    _send_message(),
                )
                .await
                .unwrap();

            verifier_sm = verifier_sm
                .step(
                    &mock_profile(),
                    VerifierMessages::PresentationRejectReceived(_problem_report()),
                    _send_message(),
                )
                .await
                .unwrap();
            assert_match!(VerifierState::Finished, verifier_sm.get_state());

            verifier_sm = verifier_sm
                .step(
                    &mock_profile(),
                    VerifierMessages::PresentationProposalReceived(_presentation_proposal()),
                    _send_message(),
                )
                .await
                .unwrap();
            assert_match!(VerifierState::Finished, verifier_sm.get_state());
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

                let s = verifier.get_state();
                warn!("State is = {:?}", s);
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

        #[tokio::test]
        #[cfg(feature = "general_test")]
        async fn test_verifier_find_message_to_handle_from_finished_state() {
            let _setup = SetupMocks::init();

            let verifier = _verifier_sm_from_request().to_finished_state().await;

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

        #[tokio::test]
        #[cfg(feature = "general_test")]
        async fn test_get_state() {
            let _setup = SetupMocks::init();
            let _mock_builder = MockBuilder::init().set_mock_result_for_validate_indy_proof(Ok(true));

            assert_eq!(
                VerifierState::PresentationRequestSet,
                _verifier_sm_from_request().get_state()
            );
            assert_eq!(
                VerifierState::PresentationRequestSent,
                _verifier_sm_from_request()
                    .to_presentation_request_sent_state()
                    .get_state()
            );
            assert_eq!(
                VerifierState::Finished,
                _verifier_sm_from_request().to_finished_state().await.get_state()
            );
        }
    }
}
