use std::collections::HashMap;
use std::fmt::Display;

use indy_sys::{WalletHandle, PoolHandle};

use crate::error::prelude::*;
use crate::messages::a2a::{A2AMessage, MessageId};
use crate::messages::error::ProblemReport;
use crate::messages::proof_presentation::presentation::Presentation;
use crate::messages::proof_presentation::presentation_ack::PresentationAck;
use crate::messages::proof_presentation::presentation_proposal::PresentationProposal;
use crate::messages::proof_presentation::presentation_request::{PresentationRequest, PresentationRequestData};
use crate::messages::status::Status;
use crate::protocols::proof_presentation::verifier::messages::VerifierMessages;
use crate::protocols::proof_presentation::verifier::states::finished::FinishedState;
use crate::protocols::proof_presentation::verifier::states::initial::InitialVerifierState;
use crate::protocols::proof_presentation::verifier::states::presentation_proposal_received::PresentationProposalReceivedState;
use crate::protocols::proof_presentation::verifier::states::presentation_request_sent::PresentationRequestSentState;
use crate::protocols::proof_presentation::verifier::states::presentation_request_set::PresentationRequestSetState;
use crate::protocols::proof_presentation::verifier::verify_thread_id;
use crate::protocols::SendClosure;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Default)]
pub struct VerifierSM {
    source_id: String,
    thread_id: String,
    state: VerifierFullState,
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

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum RevocationStatus {
    Revoked,
    NonRevoked,
}

fn build_verification_ack(thread_id: &str) -> PresentationAck {
    PresentationAck::create().set_thread_id(thread_id).set_out_time()
}

fn build_starting_presentation_request(
    thread_id: &str,
    request_data: &PresentationRequestData,
    comment: Option<String>,
) -> VcxResult<PresentationRequest> {
    Ok(PresentationRequest::create()
        .set_id(thread_id.into())
        .set_comment(comment)
        .set_request_presentations_attach(request_data)?
        .set_out_time())
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
            thread_id: MessageId::new().0,
            state: VerifierFullState::Initial(InitialVerifierState {}),
        };
        sm.set_request(presentation_request_data, None)
    }

    pub fn from_proposal(source_id: &str, presentation_proposal: &PresentationProposal) -> Self {
        Self {
            source_id: source_id.to_string(),
            thread_id: presentation_proposal.id.0.clone(),
            state: VerifierFullState::PresentationProposalReceived(PresentationProposalReceivedState::new(
                presentation_proposal.clone(),
            )),
        }
    }

    pub fn find_message_to_handle(&self, messages: HashMap<String, A2AMessage>) -> Option<(String, A2AMessage)> {
        trace!("VerifierSM::find_message_to_handle >>> messages: {:?}", messages);
        for (uid, message) in messages {
            match &self.state {
                VerifierFullState::Initial(_) => match message {
                    A2AMessage::PresentationProposal(proposal) => {
                        return Some((uid, A2AMessage::PresentationProposal(proposal)));
                    }
                    A2AMessage::PresentationRequest(request) => {
                        return Some((uid, A2AMessage::PresentationRequest(request)));
                    }
                    _ => {}
                },
                VerifierFullState::PresentationRequestSent(_) => match message {
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
                return Err(VcxError::from_msg(
                    VcxErrorKind::InvalidState,
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
                return Err(VcxError::from_msg(
                    VcxErrorKind::InvalidState,
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
        wallet_handle: WalletHandle,
        pool_handle: PoolHandle,
        message: VerifierMessages,
        send_message: Option<SendClosure>,
    ) -> VcxResult<Self> {
        trace!("VerifierSM::step >>> message: {:?}", message);
        let state_name = self.state.to_string();
        let Self {
            source_id,
            state,
            thread_id,
        } = self.clone();
        verify_thread_id(&thread_id, &message)?;
        let (state, thread_id) = match state {
            VerifierFullState::Initial(state) => match message {
                VerifierMessages::PresentationProposalReceived(ref proposal) => {
                    let thread_id = match proposal.thread {
                        Some(ref thread) => thread.thid.clone().ok_or(VcxError::from_msg(
                            VcxErrorKind::InvalidState,
                            "Received proposal with invalid thid",
                        ))?,
                        None => proposal.id.0.clone(),
                    };
                    (
                        VerifierFullState::PresentationProposalReceived(PresentationProposalReceivedState::new(
                            proposal.clone(),
                        )),
                        thread_id,
                    )
                }
                _ => {
                    warn!("Unable to process received message in state {}", state_name);
                    (VerifierFullState::Initial(state), thread_id)
                }
            },
            VerifierFullState::PresentationRequestSet(state) => {
                warn!("Unable to process received message in state {}", state_name);
                (VerifierFullState::PresentationRequestSet(state), thread_id)
            }
            VerifierFullState::PresentationProposalReceived(state) => match message {
                VerifierMessages::RejectPresentationProposal(reason) => {
                    let thread_id = match state.presentation_proposal.thread {
                        Some(thread) => thread
                            .thid
                            .ok_or(VcxError::from_msg(VcxErrorKind::InvalidState, "Thread id undefined"))?,
                        None => state.presentation_proposal.id.0,
                    };
                    let problem_report = ProblemReport::create()
                        .set_comment(Some(reason.to_string()))
                        .set_thread_id(&thread_id);
                    send_message.ok_or(VcxError::from_msg(
                        VcxErrorKind::InvalidState,
                        "Attempted to call undefined send_message callback",
                    ))?(problem_report.to_a2a_message())
                    .await?;
                    (
                        VerifierFullState::Finished(FinishedState::declined(problem_report)),
                        thread_id,
                    )
                }
                _ => {
                    warn!("Unable to process received message in state {}", state_name);
                    (VerifierFullState::PresentationProposalReceived(state), thread_id)
                }
            },
            VerifierFullState::PresentationRequestSent(state) => match message {
                VerifierMessages::VerifyPresentation(presentation) => match state
                    .verify_presentation(wallet_handle, pool_handle, &presentation, &thread_id)
                    .await
                {
                    Ok(()) => {
                        if presentation.please_ack.is_some() {
                            let ack = build_verification_ack(&thread_id);
                            send_message.ok_or(VcxError::from_msg(
                                VcxErrorKind::InvalidState,
                                "Attempted to call undefined send_message callback",
                            ))?(A2AMessage::PresentationAck(ack))
                            .await?;
                        };
                        (
                            VerifierFullState::Finished((state, presentation, RevocationStatus::NonRevoked).into()),
                            thread_id,
                        )
                    }
                    Err(err) => {
                        let problem_report = ProblemReport::create()
                            .set_comment(Some(err.to_string()))
                            .set_thread_id(&thread_id);
                        send_message.ok_or(VcxError::from_msg(
                            VcxErrorKind::InvalidState,
                            "Attempted to call undefined send_message callback",
                        ))?(problem_report.to_a2a_message())
                        .await?;
                        match err.kind() {
                            VcxErrorKind::InvalidProof => (
                                VerifierFullState::Finished((state, presentation, RevocationStatus::Revoked).into()),
                                thread_id,
                            ),
                            _ => (VerifierFullState::Finished((state, problem_report).into()), thread_id),
                        }
                    }
                },
                VerifierMessages::PresentationRejectReceived(problem_report) => {
                    (VerifierFullState::Finished((state, problem_report).into()), thread_id)
                }
                VerifierMessages::PresentationProposalReceived(proposal) => (
                    VerifierFullState::PresentationProposalReceived(PresentationProposalReceivedState::new(proposal)),
                    thread_id,
                ),
                _ => {
                    warn!("Unable to process received message in state {}", state_name);
                    (VerifierFullState::PresentationRequestSent(state), thread_id)
                }
            },
            VerifierFullState::Finished(state) => {
                if matches!(message, VerifierMessages::SendPresentationAck()) {
                    let ack = build_verification_ack(&thread_id);
                    send_message.ok_or(VcxError::from_msg(
                        VcxErrorKind::InvalidState,
                        "Attempted to call undefined send_message callback",
                    ))?(A2AMessage::PresentationAck(ack))
                    .await?;
                }
                (VerifierFullState::Finished(state), thread_id)
            }
        };

        Ok(Self {
            source_id,
            state,
            thread_id,
        })
    }

    pub fn source_id(&self) -> String {
        self.source_id.clone()
    }

    pub fn thread_id(&self) -> String {
        self.thread_id.clone()
    }

    pub fn get_state(&self) -> VerifierState {
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

    pub fn presentation_status(&self) -> u32 {
        match self.state {
            VerifierFullState::Finished(ref state) => {
                match &state.status {
                    Status::Success => {
                        match state.revocation_status {
                            Some(RevocationStatus::NonRevoked) => Status::Success.code(),
                            None => Status::Success.code(), // for backward compatibility
                            Some(RevocationStatus::Revoked) => {
                                let problem_report = ProblemReport::create()
                                    .set_comment(Some(String::from("Revoked credential was used.")));
                                Status::Failed(problem_report).code()
                            }
                        }
                    }
                    _ => state.status.code(),
                }
            }
            _ => Status::Undefined.code(),
        }
    }

    pub fn presentation_request(&self) -> VcxResult<PresentationRequest> {
        match self.state {
            VerifierFullState::Initial(_) => Err(VcxError::from_msg(
                VcxErrorKind::InvalidState,
                "Presentation request not set yet",
            )),
            VerifierFullState::PresentationRequestSet(ref state) => Ok(state.presentation_request.clone()),
            VerifierFullState::PresentationProposalReceived(ref state) => state.presentation_request.clone().ok_or(
                VcxError::from_msg(VcxErrorKind::InvalidState, "No presentation request set"),
            ),
            VerifierFullState::PresentationRequestSent(ref state) => Ok(state.presentation_request.clone()),
            VerifierFullState::Finished(ref state) => Ok(state
                .presentation_request
                .as_ref()
                .ok_or(VcxError::from_msg(
                    VcxErrorKind::InvalidState,
                    "No presentation request set",
                ))?
                .clone()),
        }
    }

    pub fn presentation(&self) -> VcxResult<Presentation> {
        match self.state {
            VerifierFullState::Finished(ref state) => state
                .presentation
                .clone()
                .ok_or(VcxError::from(VcxErrorKind::InvalidProofHandle)),
            _ => Err(VcxError::from_msg(
                VcxErrorKind::InvalidState,
                "Presentation not received yet",
            )),
        }
    }

    pub fn presentation_proposal(&self) -> VcxResult<PresentationProposal> {
        match self.state {
            VerifierFullState::PresentationProposalReceived(ref state) => Ok(state.presentation_proposal.clone()),
            _ => Err(VcxError::from_msg(
                VcxErrorKind::InvalidState,
                "Presentation proposal not received yet",
            )),
        }
    }
}

#[cfg(test)]
#[cfg(feature = "general_test")]
pub mod unit_tests {
    use crate::messages::proof_presentation::presentation::test_utils::{_presentation, _presentation_1};
    use crate::messages::proof_presentation::presentation_proposal::test_utils::_presentation_proposal;
    use crate::messages::proof_presentation::presentation_request::test_utils::_presentation_request;
    use crate::messages::proof_presentation::presentation_request::test_utils::_presentation_request_data;
    use crate::messages::proof_presentation::test_utils::{_ack, _problem_report};
    use crate::test::source_id;
    use crate::utils::devsetup::{SetupEmpty, SetupMocks};

    use super::*;

    fn _dummy_wallet_handle() -> WalletHandle {
        WalletHandle(0)
    }

    fn _dummy_pool_handle() -> PoolHandle {
        0
    }

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
                    _dummy_wallet_handle(),
                    _dummy_pool_handle(),
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
                    _dummy_wallet_handle(),
                    _dummy_pool_handle(),
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
                    _dummy_wallet_handle(),
                    _dummy_pool_handle(),
                    VerifierMessages::VerifyPresentation(_presentation()),
                    _send_message(),
                )
                .await
                .unwrap();
            self
        }
    }

    mod build_messages {
        use crate::messages::a2a::MessageId;
        use crate::messages::proof_presentation::presentation_request::PresentationRequestData;
        use crate::protocols::proof_presentation::verifier::state_machine::{
            build_starting_presentation_request, build_verification_ack,
        };
        use crate::utils::devsetup::{was_in_past, SetupMocks};

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

            let presentation_request_data = PresentationRequestData::create("1").await.unwrap();
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

            let msg_presentation_request = verifier_sm.presentation_request().unwrap();
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
                    _dummy_wallet_handle(),
                    _dummy_pool_handle(),
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
                    _dummy_wallet_handle(),
                    _dummy_pool_handle(),
                    VerifierMessages::PresentationRejectReceived(_problem_report()),
                    _send_message(),
                )
                .await
                .unwrap();
            assert_match!(VerifierFullState::PresentationRequestSet(_), verifier_sm.state);

            verifier_sm = verifier_sm
                .step(
                    _dummy_wallet_handle(),
                    _dummy_pool_handle(),
                    VerifierMessages::VerifyPresentation(_presentation()),
                    _send_message(),
                )
                .await
                .unwrap();
            assert_match!(VerifierFullState::PresentationRequestSet(_), verifier_sm.state);

            verifier_sm = verifier_sm
                .step(
                    _dummy_wallet_handle(),
                    _dummy_pool_handle(),
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
                    _dummy_wallet_handle(),
                    _dummy_pool_handle(),
                    VerifierMessages::RejectPresentationProposal(_reason()),
                    _send_message(),
                )
                .await
                .unwrap();

            assert_match!(VerifierFullState::Finished(_), verifier_sm.state);
            assert_eq!(
                Status::Declined(ProblemReport::default()).code(),
                verifier_sm.presentation_status()
            );
        }

        #[tokio::test]
        #[cfg(feature = "general_test")]
        async fn test_prover_handle_other_messages_from_presentation_proposal_received_state() {
            let _setup = SetupMocks::init();

            let mut verifier_sm = _verifier_sm().to_presentation_proposal_received_state().await;

            verifier_sm = verifier_sm
                .step(
                    _dummy_wallet_handle(),
                    _dummy_pool_handle(),
                    VerifierMessages::VerifyPresentation(_presentation()),
                    _send_message(),
                )
                .await
                .unwrap();
            assert_match!(VerifierFullState::PresentationProposalReceived(_), verifier_sm.state);

            verifier_sm = verifier_sm
                .step(
                    _dummy_wallet_handle(),
                    _dummy_pool_handle(),
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
                    _dummy_wallet_handle(),
                    _dummy_pool_handle(),
                    VerifierMessages::VerifyPresentation(_presentation()),
                    _send_message(),
                )
                .await
                .unwrap();

            assert_match!(VerifierFullState::Finished(_), verifier_sm.state);
            assert_eq!(Status::Success.code(), verifier_sm.presentation_status());
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
                    _dummy_wallet_handle(),
                    _dummy_pool_handle(),
                    VerifierMessages::VerifyPresentation(_presentation()),
                    _send_message(),
                )
                .await
                .unwrap();

            assert_match!(VerifierFullState::Finished(_), verifier_sm.state);
            assert_eq!(VerifierState::Finished, verifier_sm.get_state());
            assert_eq!(
                Status::Failed(ProblemReport::create()).code(),
                verifier_sm.presentation_status()
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
                    _dummy_wallet_handle(),
                    _dummy_pool_handle(),
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
                    _dummy_wallet_handle(),
                    _dummy_pool_handle(),
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
                    _dummy_wallet_handle(),
                    _dummy_pool_handle(),
                    VerifierMessages::PresentationRejectReceived(_problem_report()),
                    _send_message(),
                )
                .await
                .unwrap();

            assert_match!(VerifierFullState::Finished(_), verifier_sm.state);
            assert_eq!(
                Status::Failed(_problem_report()).code(),
                verifier_sm.presentation_status()
            );
        }

        #[tokio::test]
        #[cfg(feature = "general_test")]
        async fn test_prover_handle_messages_from_presentation_finished_state() {
            let _setup = SetupMocks::init();

            let mut verifier_sm = _verifier_sm_from_request();
            verifier_sm = verifier_sm.mark_presentation_request_msg_sent().unwrap();
            verifier_sm = verifier_sm
                .step(
                    _dummy_wallet_handle(),
                    _dummy_pool_handle(),
                    VerifierMessages::VerifyPresentation(_presentation()),
                    _send_message(),
                )
                .await
                .unwrap();

            verifier_sm = verifier_sm
                .step(
                    _dummy_wallet_handle(),
                    _dummy_pool_handle(),
                    VerifierMessages::PresentationRejectReceived(_problem_report()),
                    _send_message(),
                )
                .await
                .unwrap();
            assert_match!(VerifierFullState::Finished(_), verifier_sm.state);

            verifier_sm = verifier_sm
                .step(
                    _dummy_wallet_handle(),
                    _dummy_pool_handle(),
                    VerifierMessages::PresentationProposalReceived(_presentation_proposal()),
                    _send_message(),
                )
                .await
                .unwrap();
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
