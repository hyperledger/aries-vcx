use std::collections::HashMap;

use crate::error::prelude::*;
use crate::protocols::SendClosure;
use crate::messages::a2a::{A2AMessage, MessageId};
use crate::messages::error::ProblemReport;
use crate::messages::proof_presentation::presentation::Presentation;
use crate::messages::proof_presentation::presentation_proposal::{PresentationPreview, PresentationProposal};
use crate::messages::proof_presentation::presentation_request::PresentationRequest;
use crate::messages::status::Status;
use crate::protocols::proof_presentation::prover::messages::ProverMessages;
use crate::protocols::proof_presentation::prover::states::finished::FinishedState;
use crate::protocols::proof_presentation::prover::states::initial::InitialProverState;
use crate::protocols::proof_presentation::prover::states::presentation_preparation_failed::PresentationPreparationFailedState;
use crate::protocols::proof_presentation::prover::states::presentation_prepared::PresentationPreparedState;
use crate::protocols::proof_presentation::prover::states::presentation_proposal_sent::PresentationProposalSent;
use crate::protocols::proof_presentation::prover::states::presentation_request_received::PresentationRequestReceived;
use crate::protocols::proof_presentation::prover::states::presentation_sent::PresentationSentState;
use crate::protocols::proof_presentation::prover::verify_thread_id;

/// A state machine that tracks the evolution of states for a Prover during
/// the Present Proof protocol.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Default)]
pub struct ProverSM {
    source_id: String,
    thread_id: String,
    state: ProverFullState,
}

#[derive(Debug, PartialEq)]
pub enum ProverState {
    Initial,
    PresentationProposalSent,
    PresentationRequestReceived,
    PresentationPrepared,
    PresentationPreparationFailed,
    PresentationSent,
    Finished,
    Failed,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum ProverFullState {
    Initial(InitialProverState),
    PresentationProposalSent(PresentationProposalSent),
    PresentationRequestReceived(PresentationRequestReceived),
    PresentationPrepared(PresentationPreparedState),
    PresentationPreparationFailed(PresentationPreparationFailedState),
    PresentationSent(PresentationSentState),
    Finished(FinishedState),
}

impl Default for ProverFullState {
    fn default() -> Self {
        Self::PresentationRequestReceived(PresentationRequestReceived::default())
    }
}

impl ProverSM {
    pub fn new(source_id: String) -> ProverSM {
        ProverSM { source_id, thread_id: MessageId::new().0, state: ProverFullState::Initial(InitialProverState {}) }
    }

    pub fn from_request(presentation_request: PresentationRequest, source_id: String) -> ProverSM {
        ProverSM { source_id, thread_id: presentation_request.id.0.clone(), state: ProverFullState::PresentationRequestReceived(PresentationRequestReceived { presentation_request }) }
    }

    pub fn find_message_to_handle(&self, messages: HashMap<String, A2AMessage>) -> Option<(String, A2AMessage)> {
        trace!("Prover::find_message_to_handle >>> messages: {:?}", messages);
        for (uid, message) in messages {
            match self.state {
                ProverFullState::PresentationProposalSent(_) => {
                    match message {
                        A2AMessage::CommonProblemReport(problem_report) => {
                            if problem_report.from_thread(&self.thread_id) {
                                return Some((uid, A2AMessage::CommonProblemReport(problem_report)));
                            }
                        }
                        A2AMessage::PresentationRequest(request) => {
                            if request.from_thread(&self.thread_id) {
                                return Some((uid, A2AMessage::PresentationRequest(request)));
                            }
                        }
                        _ => {}
                    }
                }
                ProverFullState::PresentationSent(_) => {
                    match message {
                        A2AMessage::Ack(ack) | A2AMessage::PresentationAck(ack) => {
                            if ack.from_thread(&self.thread_id) {
                                return Some((uid, A2AMessage::PresentationAck(ack)));
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

    pub async fn step(self,
                      message: ProverMessages,
                      send_message: Option<SendClosure>,
    ) -> VcxResult<ProverSM> {
        trace!("ProverSM::step >>> message: {:?}", message);
        let ProverSM { source_id, state, thread_id } = self;
        verify_thread_id(&thread_id, &message)?;
        let state = match state {
            ProverFullState::Initial(state) => {
                match message {
                    ProverMessages::PresentationProposalSend(proposal_data) => {
                        let proposal = PresentationProposal::from(proposal_data)
                            .set_id(&thread_id);
                        send_message.ok_or(
                            VcxError::from_msg(VcxErrorKind::InvalidState, "Attempted to call undefined send_message callback")
                        )?(proposal.to_a2a_message()).await?;
                        ProverFullState::PresentationProposalSent(PresentationProposalSent::new(proposal))
                    }
                    _ => {
                        warn!("Unable to process received message in this state");
                        ProverFullState::Initial(state)
                    }
                }
            }
            ProverFullState::PresentationProposalSent(state) => {
                match message {
                    ProverMessages::PresentationRequestReceived(request) => {
                        ProverFullState::PresentationRequestReceived(PresentationRequestReceived::new(request))
                    }
                    // TODO: Perhaps use a different message type?
                    ProverMessages::PresentationRejectReceived(problem_report) => {
                        ProverFullState::Finished(FinishedState::declined(problem_report))
                    }
                    _ => {
                        warn!("Unable to process received message in this state");
                        ProverFullState::PresentationProposalSent(state)
                    }
                }
            }
            ProverFullState::PresentationRequestReceived(state) => {
                match message {
                    ProverMessages::PresentationProposalSend(proposal_data) => {
                        let proposal = PresentationProposal::from(proposal_data)
                            .set_thread_id(&thread_id);
                        send_message.ok_or(
                            VcxError::from_msg(VcxErrorKind::InvalidState, "Attempted to call undefined send_message callback")
                        )?(proposal.to_a2a_message()).await?;
                        ProverFullState::PresentationProposalSent(PresentationProposalSent::new(proposal))
                    }
                    ProverMessages::SetPresentation(presentation) => {
                        let presentation = presentation.set_thread_id(&thread_id);
                        ProverFullState::PresentationPrepared((state, presentation).into())
                    }
                    ProverMessages::PreparePresentation((credentials, self_attested_attrs)) => {
                        match state.build_presentation(&credentials, &self_attested_attrs).await {
                            Ok(presentation) => {
                                let presentation = Presentation::create()
                                    .ask_for_ack()
                                    .set_thread_id(&thread_id)
                                    .set_presentations_attach(presentation)?;

                                ProverFullState::PresentationPrepared((state, presentation).into())
                            }
                            Err(err) => {
                                let problem_report =
                                    ProblemReport::create()
                                        .set_comment(Some(err.to_string()))
                                        .set_thread_id(&thread_id);

                                ProverFullState::PresentationPreparationFailed((state, problem_report).into())
                            }
                        }
                    }
                    ProverMessages::RejectPresentationRequest(reason) => {
                        if let Some(send_message) = send_message {
                            let problem_report = Self::_handle_reject_presentation_request(send_message, &reason, &thread_id).await?;
                            ProverFullState::Finished((state, problem_report).into())
                        } else {
                            return Err(VcxError::from_msg(VcxErrorKind::ActionNotSupported, "Send message closure is required."));
                        }
                    }
                    ProverMessages::ProposePresentation(preview) => {
                        if let Some(send_message) = send_message {
                            Self::_handle_presentation_proposal(send_message, preview, &thread_id).await?;
                            ProverFullState::Finished(state.into())
                        } else {
                            return Err(VcxError::from_msg(VcxErrorKind::ActionNotSupported, "Send message closure is required."));
                        }
                    }
                    _ => {
                        warn!("Unable to process received message in this state");
                        ProverFullState::PresentationRequestReceived(state)
                    }
                }
            }
            ProverFullState::PresentationPrepared(state) => {
                match message {
                    ProverMessages::SendPresentation => {
                        if let Some(send_message) = send_message {
                            send_message(state.presentation.to_a2a_message()).await?;
                            ProverFullState::PresentationSent((state).into())
                        } else {
                            return Err(VcxError::from_msg(VcxErrorKind::ActionNotSupported, "Send message closure is required."));
                        }
                    }
                    ProverMessages::RejectPresentationRequest(reason) => {
                        if let Some(send_message) = send_message {
                            let problem_report = Self::_handle_reject_presentation_request(send_message, &reason, &thread_id).await?;
                            ProverFullState::Finished(FinishedState::declined(problem_report))
                        } else {
                            return Err(VcxError::from_msg(VcxErrorKind::ActionNotSupported, "Send message closure is required."));
                        }
                    }
                    ProverMessages::ProposePresentation(preview) => {
                        if let Some(send_message) = send_message {
                            Self::_handle_presentation_proposal(send_message, preview, &thread_id).await?;
                            ProverFullState::Finished(state.into())
                        } else {
                            return Err(VcxError::from_msg(VcxErrorKind::ActionNotSupported, "Send message closure is required."));
                        }
                    }
                    _ => {
                        warn!("Unable to process received message in this state");
                        ProverFullState::PresentationPrepared(state)
                    }
                }
            }
            ProverFullState::PresentationPreparationFailed(state) => {
                match message {
                    ProverMessages::SendPresentation => {
                        if let Some(send_message) = send_message {
                            send_message(state.problem_report.to_a2a_message()).await?;
                            ProverFullState::Finished((state).into())
                        } else {
                            return Err(VcxError::from_msg(VcxErrorKind::ActionNotSupported, "Send message closure is required."));
                        }
                    }
                    _ => {
                        warn!("Unable to process received message in this state");
                        ProverFullState::PresentationPreparationFailed(state)
                    }
                }
            }
            ProverFullState::PresentationSent(state) => {
                match message {
                    ProverMessages::PresentationAckReceived(ack) => {
                        ProverFullState::Finished((state, ack).into())
                    }
                    ProverMessages::PresentationRejectReceived(problem_report) => {
                        ProverFullState::Finished((state, problem_report).into())
                    }
                    ProverMessages::RejectPresentationRequest(_) => {
                        return Err(VcxError::from_msg(VcxErrorKind::ActionNotSupported, "Presentation is already sent"));
                    }
                    _ => {
                        warn!("Unable to process received message in this state");
                        ProverFullState::PresentationSent(state)
                    }
                }
            }
            ProverFullState::Finished(state) => ProverFullState::Finished(state)
        };

        Ok(ProverSM { source_id, state, thread_id })
    }

    async fn _handle_reject_presentation_request<'a>(
        send_message: SendClosure,
        reason: &'a str,
        thread_id: &'a str,
    ) -> VcxResult<ProblemReport> {
        let problem_report = ProblemReport::create()
            .set_comment(Some(reason.to_string()))
            .set_thread_id(thread_id);
        send_message(problem_report.to_a2a_message()).await?;
        Ok(problem_report)
    }

    async fn _handle_presentation_proposal(
        send_message: SendClosure,
        preview: PresentationPreview,
        thread_id: &str,
    ) -> VcxResult<()> {
        let proposal = PresentationProposal::create()
            .set_presentation_preview(preview)
            .set_thread_id(thread_id);
        send_message(proposal.to_a2a_message()).await
    }

    pub fn source_id(&self) -> String { self.source_id.clone() }

    pub fn get_thread_id(&self) -> VcxResult<String> { Ok(self.thread_id.clone()) }

    pub fn get_state(&self) -> ProverState {
        match self.state {
            ProverFullState::Initial(_) => ProverState::Initial,
            ProverFullState::PresentationProposalSent(_) => ProverState::PresentationProposalSent,
            ProverFullState::PresentationRequestReceived(_) => ProverState::PresentationRequestReceived,
            ProverFullState::PresentationPrepared(_) => ProverState::PresentationPrepared,
            ProverFullState::PresentationPreparationFailed(_) => ProverState::PresentationPreparationFailed,
            ProverFullState::PresentationSent(_) => ProverState::PresentationSent,
            ProverFullState::Finished(ref status) => {
                match status.status {
                    Status::Success => ProverState::Finished,
                    _ => ProverState::Failed
                }
            }
        }
    }

    pub fn progressable_by_message(&self) -> bool {
        trace!("Prover::states::progressable_by_message >> state: {:?}", self.state);
        match self.state {
            ProverFullState::Initial(_) => false,
            ProverFullState::PresentationProposalSent(_) => true,
            ProverFullState::PresentationRequestReceived(_) => false,
            ProverFullState::PresentationPrepared(_) => true,
            ProverFullState::PresentationPreparationFailed(_) => true,
            ProverFullState::PresentationSent(_) => true,
            ProverFullState::Finished(_) => false,
        }
    }

    pub fn presentation_status(&self) -> u32 {
        match self.state {
            ProverFullState::Finished(ref state) => state.status.code(),
            _ => Status::Undefined.code()
        }
    }

    pub fn presentation_request(&self) -> VcxResult<&PresentationRequest> {
        match self.state {
            ProverFullState::Initial(_) => Err(VcxError::from_msg(VcxErrorKind::NotReady, "Presentation request is not available")),
            ProverFullState::PresentationProposalSent(_) => Err(VcxError::from_msg(VcxErrorKind::NotReady, "Presentation request is not available")),
            ProverFullState::PresentationRequestReceived(ref state) => Ok(&state.presentation_request),
            ProverFullState::PresentationPrepared(ref state) => Ok(&state.presentation_request),
            ProverFullState::PresentationPreparationFailed(ref state) => Ok(&state.presentation_request),
            ProverFullState::PresentationSent(ref state) => Ok(&state.presentation_request),
            ProverFullState::Finished(ref state) => Ok(state.presentation_request.as_ref().ok_or(VcxError::from_msg(VcxErrorKind::NotReady, "Presentation request is not available"))?)
        }
    }

    pub fn presentation(&self) -> VcxResult<&Presentation> {
        match self.state {
            ProverFullState::Initial(_) => Err(VcxError::from_msg(VcxErrorKind::NotReady, "Presentation is not created yet")),
            ProverFullState::PresentationProposalSent(_) => Err(VcxError::from_msg(VcxErrorKind::NotReady, "Presentation is not created yet")),
            ProverFullState::PresentationRequestReceived(_) => Err(VcxError::from_msg(VcxErrorKind::NotReady, "Presentation is not created yet")),
            ProverFullState::PresentationPrepared(ref state) => Ok(&state.presentation),
            ProverFullState::PresentationPreparationFailed(_) => Err(VcxError::from_msg(VcxErrorKind::NotReady, "Presentation is not created yet")),
            ProverFullState::PresentationSent(ref state) => Ok(&state.presentation),
            ProverFullState::Finished(ref state) => Ok(state.presentation.as_ref().ok_or(VcxError::from_msg(VcxErrorKind::NotReady, "Presentation is not available"))?)
        }
    }
}

#[cfg(test)]
pub mod test {
    use crate::messages::proof_presentation::presentation::test_utils::_presentation;
    use crate::messages::proof_presentation::presentation_proposal::test_utils::{_presentation_preview, _presentation_proposal, _presentation_proposal_data};
    use crate::messages::proof_presentation::presentation_request::test_utils::_presentation_request;
    use crate::messages::proof_presentation::test::{_ack, _problem_report};
    use crate::test::source_id;
    use crate::utils::devsetup::SetupMocks;

    use super::*;

    pub fn _prover_sm_from_request() -> ProverSM {
        ProverSM::from_request(_presentation_request(), source_id())
    }

    pub fn _send_message() -> Option<SendClosure> {
        Some(Box::new(|_: A2AMessage| Box::pin(async { VcxResult::Ok(()) })))
    }

    pub fn _prover_sm() -> ProverSM {
        ProverSM::new(source_id())
    }

    impl ProverSM {
        async fn to_presentation_proposal_sent_state(mut self) -> ProverSM {
            self = self.step(ProverMessages::PresentationProposalSend(_presentation_proposal_data()), _send_message()).await.unwrap();
            self
        }

        async fn to_presentation_prepared_state(mut self) -> ProverSM {
            self = self.step(ProverMessages::PreparePresentation((_credentials(), _self_attested())), None).await.unwrap();
            self
        }

        async fn to_presentation_sent_state(mut self) -> ProverSM {
            self = self.step(ProverMessages::PreparePresentation((_credentials(), _self_attested())), _send_message()).await.unwrap();
            self = self.step(ProverMessages::SendPresentation, _send_message()).await.unwrap();
            self
        }

        async fn to_finished_state(mut self) -> ProverSM {
            self = self.step(ProverMessages::PreparePresentation((_credentials(), _self_attested())), None).await.unwrap();
            self = self.step(ProverMessages::SendPresentation, _send_message()).await.unwrap();
            self = self.step(ProverMessages::PresentationAckReceived(_ack()), _send_message()).await.unwrap();
            self
        }
    }

    fn _credentials() -> String {
        json!({
            "attrs":{
            "attribute_0":{
                "credential":{
                    "cred_info":{
                        "attrs":{"name": "alice"},
                        "cred_def_id": "V4SGRU86Z58d6TV7PBUe6f:3:CL:419:tag",
                        "referent": "a1991de8-8317-43fd-98b3-63bac40b9e8b",
                        "schema_id": "V4SGRU86Z58d6TV7PBUe6f:2:QcimrRShWQniqlHUtIDddYP0n:1.0"
                        }
                    }
                }
            }
        }).to_string()
    }

    fn _self_attested() -> String {
        json!({}).to_string()
    }

    mod new {
        use super::*;

        #[tokio::test]
        #[cfg(feature = "general_test")]
        async fn test_prover_new() {
            let _setup = SetupMocks::init();

            let prover_sm = _prover_sm();

            assert_match!(ProverFullState::Initial(_), prover_sm.state);
            assert_eq!(source_id(), prover_sm.source_id());
        }

        #[tokio::test]
        #[cfg(feature = "general_test")]
        async fn test_prover_from_request() {
            let _setup = SetupMocks::init();

            let prover_sm = _prover_sm_from_request();

            assert_match!(ProverFullState::PresentationRequestReceived(_), prover_sm.state);
            assert_eq!(source_id(), prover_sm.source_id());
        }
    }

    mod step {
        use crate::utils::constants::CREDS_FROM_PROOF_REQ;
        use crate::utils::mockdata::mock_settings::MockBuilder;

        use super::*;

        #[tokio::test]
        #[cfg(feature = "general_test")]
        async fn test_prover_init() {
            let _setup = SetupMocks::init();

            let prover_sm = _prover_sm_from_request();
            assert_match!(ProverFullState::PresentationRequestReceived(_), prover_sm.state);
        }

        #[tokio::test]
        #[cfg(feature = "general_test")]
        async fn test_prover_handle_presentation_proposal_send_from_initial_state() {
            let _setup = SetupMocks::init();

            let mut prover_sm = _prover_sm();
            prover_sm = prover_sm.step(ProverMessages::PresentationProposalSend(_presentation_proposal_data()), _send_message()).await.unwrap();

            assert_match!(ProverFullState::PresentationProposalSent(_), prover_sm.state);
        }

        #[tokio::test]
        #[cfg(feature = "general_test")]
        async fn test_prover_handle_presentation_request_received_from_presentation_proposal_sent_state() {
            let _setup = SetupMocks::init();

            let mut prover_sm = _prover_sm_from_request().to_presentation_proposal_sent_state().await;
            prover_sm = prover_sm.step(ProverMessages::PresentationRequestReceived(_presentation_request()), _send_message()).await.unwrap();

            assert_match!(ProverFullState::PresentationRequestReceived(_), prover_sm.state);
        }

        #[tokio::test]
        #[cfg(feature = "general_test")]
        async fn test_prover_handle_presentation_reject_received_from_presentation_proposal_sent_state() {
            let _setup = SetupMocks::init();

            let mut prover_sm = _prover_sm_from_request().to_presentation_proposal_sent_state().await;
            prover_sm = prover_sm.step(ProverMessages::PresentationRejectReceived(_problem_report()), _send_message()).await.unwrap();

            assert_match!(ProverFullState::Finished(_), prover_sm.state);
            assert_eq!(Status::Declined(ProblemReport::default()).code(), prover_sm.presentation_status());
        }

        #[tokio::test]
        #[cfg(feature = "general_test")]
        async fn test_prover_handle_presentation_proposal_send_from_presentation_request_received_state() {
            let _setup = SetupMocks::init();

            let mut prover_sm = _prover_sm_from_request();
            prover_sm = prover_sm.step(ProverMessages::PresentationProposalSend(_presentation_proposal_data()), _send_message()).await.unwrap();

            assert_match!(ProverFullState::PresentationProposalSent(_), prover_sm.state);
        }

        #[tokio::test]
        #[cfg(feature = "general_test")]
        async fn test_prover_handle_set_presentation_from_presentation_request_received_state() {
            let _setup = SetupMocks::init();

            let mut prover_sm = _prover_sm_from_request();
            prover_sm = prover_sm.step(ProverMessages::SetPresentation(_presentation()), _send_message()).await.unwrap();

            assert_match!(ProverFullState::PresentationPrepared(_), prover_sm.state);
        }

        #[tokio::test]
        #[cfg(feature = "general_test")]
        async fn test_prover_handle_prepare_presentation_message_from_presentation_request_received_state_for_invalid_credentials() {
            let _setup = SetupMocks::init();
            let _mock_builder = MockBuilder::init().
                set_mock_creds_retrieved_for_proof_request(CREDS_FROM_PROOF_REQ);

            let mut prover_sm = _prover_sm_from_request();
            prover_sm = prover_sm.step(ProverMessages::PreparePresentation(("invalid".to_string(), _self_attested())), _send_message()).await.unwrap();

            assert_match!(ProverFullState::PresentationPreparationFailed(_), prover_sm.state);
        }

        #[tokio::test]
        #[cfg(feature = "general_test")]
        async fn test_prover_handle_reject_presentation_request_message_from_presentation_request_received_state() {
            let _setup = SetupMocks::init();

            let mut prover_sm = _prover_sm_from_request();
            prover_sm = prover_sm.step(ProverMessages::RejectPresentationRequest(String::from("reject request")), _send_message()).await.unwrap();

            assert_match!(ProverFullState::Finished(_), prover_sm.state);
        }

        #[tokio::test]
        #[cfg(feature = "general_test")]
        async fn test_prover_handle_propose_presentation_message_from_presentation_request_received_state() {
            let _setup = SetupMocks::init();

            let mut prover_sm = _prover_sm_from_request();
            prover_sm = prover_sm.step(ProverMessages::ProposePresentation(_presentation_preview()), _send_message()).await.unwrap();

            assert_match!(ProverFullState::Finished(_), prover_sm.state);
        }

        #[tokio::test]
        #[cfg(feature = "general_test")]
        async fn test_prover_handle_other_messages_from_presentation_request_received_state() {
            let _setup = SetupMocks::init();

            let mut prover_sm = _prover_sm_from_request();

            prover_sm = prover_sm.step(ProverMessages::SendPresentation, _send_message()).await.unwrap();
            assert_match!(ProverFullState::PresentationRequestReceived(_), prover_sm.state);

            prover_sm = prover_sm.step(ProverMessages::PresentationAckReceived(_ack()), _send_message()).await.unwrap();
            assert_match!(ProverFullState::PresentationRequestReceived(_), prover_sm.state);
        }

        #[tokio::test]
        #[cfg(feature = "general_test")]
        async fn test_prover_handle_send_presentation_message_from_presentation_prepared_state() {
            let _setup = SetupMocks::init();

            let mut prover_sm = _prover_sm_from_request();

            prover_sm = prover_sm.step(ProverMessages::PreparePresentation((_credentials(), _self_attested())), _send_message()).await.unwrap();
            prover_sm = prover_sm.step(ProverMessages::SendPresentation, _send_message()).await.unwrap();

            assert_match!(ProverFullState::PresentationSent(_), prover_sm.state);
        }

        #[tokio::test]
        #[cfg(feature = "general_test")]
        async fn test_prover_handle_other_messages_from_presentation_prepared_state() {
            let _setup = SetupMocks::init();

            let mut prover_sm = _prover_sm_from_request().to_presentation_prepared_state().await;

            prover_sm = prover_sm.step(ProverMessages::PresentationRejectReceived(_problem_report()), _send_message()).await.unwrap();
            assert_match!(ProverFullState::PresentationPrepared(_), prover_sm.state);

            prover_sm = prover_sm.step(ProverMessages::PresentationAckReceived(_ack()), _send_message()).await.unwrap();
            assert_match!(ProverFullState::PresentationPrepared(_), prover_sm.state);
        }

        #[tokio::test]
        #[cfg(feature = "general_test")]
        async fn test_prover_handle_reject_presentation_request_message_from_presentation_prepared_state() {
            let _setup = SetupMocks::init();

            let mut prover_sm = _prover_sm_from_request().to_presentation_prepared_state().await;

            prover_sm = prover_sm.step(ProverMessages::RejectPresentationRequest(String::from("reject request")), _send_message()).await.unwrap();

            assert_match!(ProverFullState::Finished(_), prover_sm.state);
        }

        #[tokio::test]
        #[cfg(feature = "general_test")]
        async fn test_prover_handle_propose_presentation_message_from_presentation_prepared_state() {
            let _setup = SetupMocks::init();

            let mut prover_sm = _prover_sm_from_request().to_presentation_prepared_state().await;
            prover_sm = prover_sm.step(ProverMessages::ProposePresentation(_presentation_preview()), _send_message()).await.unwrap();

            assert_match!(ProverFullState::Finished(_), prover_sm.state);
        }

        #[tokio::test]
        #[cfg(feature = "general_test")]
        async fn test_prover_handle_send_presentation_message_from_presentation_preparation_failed_state() {
            let _setup = SetupMocks::init();
            let _mock_builder = MockBuilder::init().
                set_mock_creds_retrieved_for_proof_request(CREDS_FROM_PROOF_REQ);

            let mut prover_sm = _prover_sm_from_request();
            prover_sm = prover_sm.step(ProverMessages::PreparePresentation(("invalid".to_string(), _self_attested())), _send_message()).await.unwrap();
            assert_match!(ProverFullState::PresentationPreparationFailed(_), prover_sm.state);

            prover_sm = prover_sm.step(ProverMessages::SendPresentation, _send_message()).await.unwrap();
            assert_match!(ProverFullState::Finished(_), prover_sm.state);
            assert_eq!(Status::Failed(ProblemReport::default()).code(), prover_sm.presentation_status());
        }

        #[tokio::test]
        #[cfg(feature = "general_test")]
        async fn test_prover_handle_other_messages_from_presentation_preparation_failed_state() {
            let _setup = SetupMocks::init();
            let _mock_builder = MockBuilder::init().
                set_mock_creds_retrieved_for_proof_request(CREDS_FROM_PROOF_REQ);

            let mut prover_sm = _prover_sm_from_request();
            prover_sm = prover_sm.step(ProverMessages::PreparePresentation(("invalid".to_string(), _self_attested())), _send_message()).await.unwrap();

            prover_sm = prover_sm.step(ProverMessages::PresentationRejectReceived(_problem_report()), _send_message()).await.unwrap();
            assert_match!(ProverFullState::PresentationPreparationFailed(_), prover_sm.state);

            prover_sm = prover_sm.step(ProverMessages::PresentationAckReceived(_ack()), _send_message()).await.unwrap();
            assert_match!(ProverFullState::PresentationPreparationFailed(_), prover_sm.state);
        }

        #[tokio::test]
        #[cfg(feature = "general_test")]
        async fn test_prover_handle_ack_message_from_presentation_sent_state() {
            let _setup = SetupMocks::init();

            let mut prover_sm = _prover_sm_from_request();
            prover_sm = prover_sm.step(ProverMessages::PreparePresentation((_credentials(), _self_attested())), _send_message()).await.unwrap();
            prover_sm = prover_sm.step(ProverMessages::SendPresentation, _send_message()).await.unwrap();
            prover_sm = prover_sm.step(ProverMessages::PresentationAckReceived(_ack()), _send_message()).await.unwrap();

            assert_match!(ProverFullState::Finished(_), prover_sm.state);
            assert_eq!(Status::Success.code(), prover_sm.presentation_status());
        }

        #[tokio::test]
        #[cfg(feature = "general_test")]
        async fn test_prover_handle_reject_presentation_request_message_from_presentation_sent_state() {
            let _setup = SetupMocks::init();

            let prover_sm = _prover_sm_from_request().to_presentation_sent_state().await;
            let err = prover_sm.step(ProverMessages::RejectPresentationRequest(String::from("reject")), _send_message()).await.unwrap_err();
            assert_eq!(VcxErrorKind::ActionNotSupported, err.kind());
        }

        #[tokio::test]
        #[cfg(feature = "general_test")]
        async fn test_prover_handle_presentation_reject_message_from_presentation_sent_state() {
            let _setup = SetupMocks::init();

            let mut prover_sm = _prover_sm_from_request();
            prover_sm = prover_sm.step(ProverMessages::PreparePresentation((_credentials(), _self_attested())), _send_message()).await.unwrap();
            prover_sm = prover_sm.step(ProverMessages::SendPresentation, _send_message()).await.unwrap();
            prover_sm = prover_sm.step(ProverMessages::PresentationRejectReceived(_problem_report()), _send_message()).await.unwrap();

            assert_match!(ProverFullState::Finished(_), prover_sm.state);
            assert_eq!(Status::Failed(ProblemReport::create()).code(), prover_sm.presentation_status());
        }

        #[tokio::test]
        #[cfg(feature = "general_test")]
        async fn test_prover_handle_other_messages_from_presentation_sent_state() {
            let _setup = SetupMocks::init();

            let mut prover_sm = _prover_sm_from_request();
            prover_sm = prover_sm.step(ProverMessages::PreparePresentation((_credentials(), _self_attested())), _send_message()).await.unwrap();
            prover_sm = prover_sm.step(ProverMessages::SendPresentation, _send_message()).await.unwrap();

            prover_sm = prover_sm.step(ProverMessages::PreparePresentation((_credentials(), _self_attested())), _send_message()).await.unwrap();
            assert_match!(ProverFullState::PresentationSent(_), prover_sm.state);

            prover_sm = prover_sm.step(ProverMessages::SendPresentation, _send_message()).await.unwrap();
            assert_match!(ProverFullState::PresentationSent(_), prover_sm.state);
        }

        #[tokio::test]
        #[cfg(feature = "general_test")]
        async fn test_prover_handle_messages_from_finished_state() {
            let _setup = SetupMocks::init();

            let mut prover_sm = _prover_sm_from_request();
            prover_sm = prover_sm.step(ProverMessages::PreparePresentation((_credentials(), _self_attested())), _send_message()).await.unwrap();
            prover_sm = prover_sm.step(ProverMessages::SendPresentation, _send_message()).await.unwrap();
            prover_sm = prover_sm.step(ProverMessages::PresentationAckReceived(_ack()), _send_message()).await.unwrap();

            prover_sm = prover_sm.step(ProverMessages::PresentationAckReceived(_ack()), _send_message()).await.unwrap();
            assert_match!(ProverFullState::Finished(_), prover_sm.state);

            prover_sm = prover_sm.step(ProverMessages::PresentationRejectReceived(_problem_report()), _send_message()).await.unwrap();
            assert_match!(ProverFullState::Finished(_), prover_sm.state);
        }
    }

    mod find_message_to_handle {
        use super::*;

        #[test]
        #[cfg(feature = "general_test")]
        fn test_prover_find_message_to_handle_from_intial_state() {
            let _setup = SetupMocks::init();
            let prover = _prover_sm();
            {
                let messages = map!(
                    "key_1".to_string() => A2AMessage::PresentationProposal(_presentation_proposal()),
                    "key_2".to_string() => A2AMessage::Presentation(_presentation()),
                    "key_3".to_string() => A2AMessage::PresentationRequest(_presentation_request()),
                    "key_4".to_string() => A2AMessage::PresentationAck(_ack()),
                    "key_5".to_string() => A2AMessage::CommonProblemReport(_problem_report())
                );

                assert!(prover.find_message_to_handle(messages).is_none());
            }
        }

        #[test]
        #[cfg(feature = "general_test")]
        fn test_prover_find_message_to_handle_from_presentation_request_received_state() {
            let _setup = SetupMocks::init();

            let prover = _prover_sm_from_request();

            // No messages
            {
                let messages = map!(
                    "key_1".to_string() => A2AMessage::PresentationProposal(_presentation_proposal()),
                    "key_2".to_string() => A2AMessage::Presentation(_presentation()),
                    "key_3".to_string() => A2AMessage::PresentationRequest(_presentation_request()),
                    "key_4".to_string() => A2AMessage::PresentationAck(_ack()),
                    "key_5".to_string() => A2AMessage::CommonProblemReport(_problem_report())
                );

                assert!(prover.find_message_to_handle(messages).is_none());
            }
        }

        #[tokio::test]
        #[cfg(feature = "general_test")]
        async fn test_prover_find_message_to_handle_from_presentation_prepared_state() {
            let _setup = SetupMocks::init();

            let prover = _prover_sm_from_request().to_presentation_prepared_state().await;

            // No messages
            {
                let messages = map!(
                    "key_1".to_string() => A2AMessage::PresentationProposal(_presentation_proposal()),
                    "key_2".to_string() => A2AMessage::Presentation(_presentation()),
                    "key_3".to_string() => A2AMessage::PresentationRequest(_presentation_request()),
                    "key_4".to_string() => A2AMessage::PresentationAck(_ack()),
                    "key_5".to_string() => A2AMessage::CommonProblemReport(_problem_report())
                );

                assert!(prover.find_message_to_handle(messages).is_none());
            }
        }

        #[tokio::test]
        #[cfg(feature = "general_test")]
        async fn test_prover_find_message_to_handle_from_presentation_sent_state() {
            let _setup = SetupMocks::init();

            let prover = _prover_sm_from_request().to_presentation_sent_state().await;

            // Ack
            {
                let messages = map!(
                    "key_1".to_string() => A2AMessage::PresentationProposal(_presentation_proposal()),
                    "key_2".to_string() => A2AMessage::Presentation(_presentation()),
                    "key_3".to_string() => A2AMessage::PresentationAck(_ack())
                );

                let (uid, message) = prover.find_message_to_handle(messages).unwrap();
                assert_eq!("key_3", uid);
                assert_match!(A2AMessage::PresentationAck(_), message);
            }

            // Problem Report
            {
                let messages = map!(
                    "key_1".to_string() => A2AMessage::PresentationProposal(_presentation_proposal()),
                    "key_2".to_string() => A2AMessage::PresentationRequest(_presentation_request()),
                    "key_3".to_string() => A2AMessage::CommonProblemReport(_problem_report())
                );

                let (uid, message) = prover.find_message_to_handle(messages).unwrap();
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

                assert!(prover.find_message_to_handle(messages).is_none());
            }

            // No messages
            {
                let messages = map!(
                    "key_1".to_string() => A2AMessage::PresentationProposal(_presentation_proposal()),
                    "key_2".to_string() => A2AMessage::PresentationRequest(_presentation_request())
                );

                assert!(prover.find_message_to_handle(messages).is_none());
            }
        }

        #[tokio::test]
        #[cfg(feature = "general_test")]
        async fn test_prover_find_message_to_handle_from_finished_state() {
            let _setup = SetupMocks::init();

            let prover = _prover_sm_from_request().to_finished_state().await;

            // No messages
            {
                let messages = map!(
                    "key_1".to_string() => A2AMessage::PresentationProposal(_presentation_proposal()),
                    "key_2".to_string() => A2AMessage::Presentation(_presentation()),
                    "key_3".to_string() => A2AMessage::PresentationRequest(_presentation_request()),
                    "key_4".to_string() => A2AMessage::PresentationAck(_ack()),
                    "key_5".to_string() => A2AMessage::CommonProblemReport(_problem_report())
                );

                assert!(prover.find_message_to_handle(messages).is_none());
            }
        }
    }

    mod get_state {
        use super::*;

        #[tokio::test]
        #[cfg(feature = "general_test")]
        async fn test_get_state() {
            let _setup = SetupMocks::init();

            assert_eq!(ProverState::PresentationRequestReceived, _prover_sm_from_request().get_state());
            assert_eq!(ProverState::PresentationPrepared, _prover_sm_from_request().to_presentation_prepared_state().await.get_state());
            assert_eq!(ProverState::PresentationSent, _prover_sm_from_request().to_presentation_sent_state().await.get_state());
            assert_eq!(ProverState::Finished, _prover_sm_from_request().to_finished_state().await.get_state());
        }
    }
}
