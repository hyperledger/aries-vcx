use std::collections::HashMap;

use crate::api::VcxStateType;
use crate::connection;
use crate::error::prelude::*;
use crate::aries::handlers::proof_presentation::prover::messages::ProverMessages;
use crate::aries::messages::a2a::A2AMessage;
use crate::aries::messages::error::ProblemReport;
use crate::aries::messages::proof_presentation::presentation::Presentation;
use crate::aries::messages::proof_presentation::presentation_proposal::{PresentationPreview, PresentationProposal};
use crate::aries::messages::proof_presentation::presentation_request::PresentationRequest;
use crate::aries::messages::status::Status;
use crate::aries::handlers::proof_presentation::prover::states::initial::InitialState;
use crate::aries::handlers::proof_presentation::prover::states::presentation_prepared::PresentationPreparedState;
use crate::aries::handlers::proof_presentation::prover::states::presentation_prepared_failed::PresentationPreparationFailedState;
use crate::aries::handlers::proof_presentation::prover::states::presentation_sent::PresentationSentState;
use crate::aries::handlers::proof_presentation::prover::states::finished::FinishedState;

/// A state machine that tracks the evolution of states for a Prover during
/// the Present Proof protocol.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct ProverSM {
    source_id: String,
    thread_id: String,
    state: ProverState,
}

impl ProverSM {
    pub fn new(presentation_request: PresentationRequest, source_id: String) -> ProverSM {
        ProverSM { source_id, thread_id: presentation_request.id.0.clone(), state: ProverState::Initiated(InitialState { presentation_request }) }
    }
}

// Possible Transitions:
//
// Initial -> PresentationPrepared, PresentationPreparationFailedState, Finished
// PresentationPrepared -> PresentationSent, Finished
// PresentationPreparationFailedState -> Finished
// PresentationSent -> Finished
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum ProverState {
    Initiated(InitialState),
    PresentationPrepared(PresentationPreparedState),
    PresentationPreparationFailed(PresentationPreparationFailedState),
    PresentationSent(PresentationSentState),
    Finished(FinishedState),
}

impl ProverSM {
    pub fn find_message_to_handle(&self, messages: HashMap<String, A2AMessage>) -> Option<(String, A2AMessage)> {
        trace!("Prover::find_message_to_handle >>> messages: {:?}", messages);

        for (uid, message) in messages {
            match self.state {
                ProverState::Initiated(_) => {
                    match message {
                        A2AMessage::PresentationRequest(_) => {
                            // ignore it here??
                        }
                        _ => {}
                    }
                }
                ProverState::PresentationPrepared(_) => {
                    // do not process messages
                }
                ProverState::PresentationPreparationFailed(_) => {
                    // do not process messages
                }
                ProverState::PresentationSent(_) => {
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
                ProverState::Finished(_) => {
                    // do not process messages
                }
            };
        }

        None
    }

    pub fn step(self, message: ProverMessages, connection_handle: Option<u32>) -> VcxResult<ProverSM> {
        trace!("ProverSM::step >>> message: {:?}", message);

        let ProverSM { source_id, state, thread_id } = self;

        let state = match state {
            ProverState::Initiated(state) => {
                match message {
                    ProverMessages::SetPresentation(presentation) => {
                        let presentation = presentation.set_thread_id(&thread_id);
                        ProverState::PresentationPrepared((state, presentation).into())
                    }
                    ProverMessages::PreparePresentation((credentials, self_attested_attrs)) => {
                        match state.build_presentation(&credentials, &self_attested_attrs) {
                            Ok(presentation) => {
                                let presentation = Presentation::create()
                                    .ask_for_ack()
                                    .set_thread_id(&thread_id)
                                    .set_presentations_attach(presentation)?;

                                ProverState::PresentationPrepared((state, presentation).into())
                            }
                            Err(err) => {
                                let problem_report =
                                    ProblemReport::create()
                                        .set_comment(err.to_string())
                                        .set_thread_id(&thread_id);

                                ProverState::PresentationPreparationFailed((state, problem_report).into())
                            }
                        }
                    }
                    ProverMessages::RejectPresentationRequest((reason)) => {
                        let connection_handle = connection_handle
                            .ok_or(VcxError::from_msg(VcxErrorKind::ActionNotSupported, "Presentation is already sent"))?;
                        Self::_handle_reject_presentation_request(connection_handle, &reason, &state.presentation_request, &thread_id)?;
                        ProverState::Finished(state.into())
                    }
                    ProverMessages::ProposePresentation((preview)) => {
                        let connection_handle = connection_handle
                            .ok_or(VcxError::from_msg(VcxErrorKind::ActionNotSupported, "Presentation is already sent"))?;
                        Self::_handle_presentation_proposal(connection_handle, preview, &state.presentation_request, &thread_id)?;
                        ProverState::Finished(state.into())
                    }
                    _ => {
                        ProverState::Initiated(state)
                    }
                }
            }
            ProverState::PresentationPrepared(state) => {
                match message {
                    ProverMessages::SendPresentation => {
                        match state.presentation_request.service.clone() {
                            None => {
                                let connection_handle = connection_handle
                                    .ok_or(VcxError::from_msg(VcxErrorKind::ActionNotSupported, "Presentation is already sent"))?;
                                connection::send_message(connection_handle, state.presentation.to_a2a_message())?;
                                ProverState::PresentationSent((state).into())
                            }
                            Some(service) => {
                                connection::send_message_to_self_endpoint(state.presentation.to_a2a_message(), &service.into())?;
                                ProverState::Finished(state.into())
                            }
                        }
                    }
                    ProverMessages::RejectPresentationRequest((reason)) => {
                        let connection_handle = connection_handle
                            .ok_or(VcxError::from_msg(VcxErrorKind::ActionNotSupported, "Presentation is already sent"))?;
                        Self::_handle_reject_presentation_request(connection_handle, &reason, &state.presentation_request, &thread_id)?;
                        ProverState::Finished(state.into())
                    }
                    ProverMessages::ProposePresentation((preview)) => {
                        let connection_handle = connection_handle
                            .ok_or(VcxError::from_msg(VcxErrorKind::ActionNotSupported, "Presentation is already sent"))?;
                        Self::_handle_presentation_proposal(connection_handle, preview, &state.presentation_request, &thread_id)?;
                        ProverState::Finished(state.into())
                    }
                    _ => {
                        ProverState::PresentationPrepared(state)
                    }
                }
            }
            ProverState::PresentationPreparationFailed(state) => {
                match message {
                    ProverMessages::SendPresentation => {
                        match state.presentation_request.service.clone() {
                            None => {
                                let connection_handle = connection_handle
                                    .ok_or(VcxError::from_msg(VcxErrorKind::ActionNotSupported, "Presentation is already sent"))?;
                                connection::send_message(connection_handle, state.problem_report.to_a2a_message())?;
                            }
                            Some(service) => {
                                connection::send_message_to_self_endpoint(state.problem_report.to_a2a_message(), &service.into())?;
                            }
                        }

                        ProverState::Finished((state).into())
                    }
                    _ => {
                        ProverState::PresentationPreparationFailed(state)
                    }
                }
            }
            ProverState::PresentationSent(state) => {
                match message {
                    ProverMessages::PresentationAckReceived(ack) => {
                        ProverState::Finished((state, ack).into())
                    }
                    ProverMessages::PresentationRejectReceived(problem_report) => {
                        ProverState::Finished((state, problem_report).into())
                    }
                    ProverMessages::RejectPresentationRequest(_) => {
                        return Err(VcxError::from_msg(VcxErrorKind::ActionNotSupported, "Presentation is already sent"));
                    }
                    _ => {
                        ProverState::PresentationSent(state)
                    }
                }
            }
            ProverState::Finished(state) => ProverState::Finished(state)
        };

        Ok(ProverSM { source_id, state, thread_id })
    }

    fn _handle_reject_presentation_request(connection_handle: u32, reason: &str, presentation_request: &PresentationRequest, thread_id: &str) -> VcxResult<()> {
        let problem_report = ProblemReport::create()
            .set_comment(reason.to_string())
            .set_thread_id(thread_id);

        match presentation_request.service.clone() {
            None => connection::send_message(connection_handle, problem_report.to_a2a_message())?,
            Some(service) => connection::send_message_to_self_endpoint(problem_report.to_a2a_message(), &service.into())?
        }

        Ok(())
    }

    fn _handle_presentation_proposal(connection_handle: u32, preview: PresentationPreview, presentation_request: &PresentationRequest, thread_id: &str) -> VcxResult<()> {
        let proposal = PresentationProposal::create()
            .set_presentation_preview(preview)
            .set_thread_id(thread_id);

        match presentation_request.service.clone() {
            None => connection::send_message(connection_handle, proposal.to_a2a_message())?,
            Some(service) => connection::send_message_to_self_endpoint(proposal.to_a2a_message(), &service.into())?
        }

        Ok(())
    }

    pub fn source_id(&self) -> String { self.source_id.clone() }

    pub fn state(&self) -> u32 {
        match self.state {
            ProverState::Initiated(_) => VcxStateType::VcxStateRequestReceived as u32,
            ProverState::PresentationPrepared(_) => VcxStateType::VcxStateRequestReceived as u32,
            ProverState::PresentationPreparationFailed(_) => VcxStateType::VcxStateRequestReceived as u32,
            ProverState::PresentationSent(_) => VcxStateType::VcxStateOfferSent as u32, // TODO: maybe VcxStateType::VcxStateAccepted
            ProverState::Finished(ref status) => {
                match status.status {
                    Status::Success => VcxStateType::VcxStateAccepted as u32,
                    _ => VcxStateType::VcxStateNone as u32,
                }
            }
        }
    }

    pub fn has_transitions(&self) -> bool {
        trace!("Prover::states::has_transitions >> state: {:?}", self.state);
        match self.state {
            ProverState::Initiated(_) => false,
            ProverState::PresentationPrepared(_) => true,
            ProverState::PresentationPreparationFailed(_) => true,
            ProverState::PresentationSent(_) => true,
            ProverState::Finished(_) => false,
        }
    }

    pub fn presentation_status(&self) -> u32 {
        match self.state {
            ProverState::Finished(ref state) => state.status.code(),
            _ => Status::Undefined.code()
        }
    }

    pub fn presentation_request(&self) -> &PresentationRequest {
        match self.state {
            ProverState::Initiated(ref state) => &state.presentation_request,
            ProverState::PresentationPrepared(ref state) => &state.presentation_request,
            ProverState::PresentationPreparationFailed(ref state) => &state.presentation_request,
            ProverState::PresentationSent(ref state) => &state.presentation_request,
            ProverState::Finished(ref state) => &state.presentation_request,
        }
    }

    pub fn presentation(&self) -> VcxResult<&Presentation> {
        match self.state {
            ProverState::Initiated(_) => Err(VcxError::from_msg(VcxErrorKind::NotReady, "Presentation is not created yet")),
            ProverState::PresentationPrepared(ref state) => Ok(&state.presentation),
            ProverState::PresentationPreparationFailed(_) => Err(VcxError::from_msg(VcxErrorKind::NotReady, "Presentation is not created yet")),
            ProverState::PresentationSent(ref state) => Ok(&state.presentation),
            ProverState::Finished(ref state) => Ok(&state.presentation),
        }
    }
}

#[cfg(test)]
pub mod test {
    use crate::utils::devsetup::SetupMocks;
    use crate::aries::handlers::connection::tests::mock_connection;
    use crate::aries::messages::proof_presentation::presentation::tests::_presentation;
    use crate::aries::messages::proof_presentation::presentation_proposal::tests::{_presentation_preview, _presentation_proposal};
    use crate::aries::messages::proof_presentation::presentation_request::tests::{_presentation_request, _presentation_request_with_service};
    use crate::aries::messages::proof_presentation::test::{_ack, _problem_report};
    use crate::aries::test::source_id;

    use super::*;

    pub fn _prover_sm() -> ProverSM {
        ProverSM::new(_presentation_request(), source_id())
    }

    impl ProverSM {
        fn to_presentation_prepared_state(mut self) -> ProverSM {
            self = self.step(ProverMessages::PreparePresentation((_credentials(), _self_attested())), None).unwrap();
            self
        }

        fn to_presentation_sent_state(mut self) -> ProverSM {
            let connection_handle = Some(mock_connection());
            self = self.step(ProverMessages::PreparePresentation((_credentials(), _self_attested())), None).unwrap();
            self = self.step(ProverMessages::SendPresentation, connection_handle).unwrap();
            self
        }

        fn to_finished_state(mut self) -> ProverSM {
            let connection_handle = Some(mock_connection());
            self = self.step(ProverMessages::PreparePresentation((_credentials(), _self_attested())), None).unwrap();
            self = self.step(ProverMessages::SendPresentation, connection_handle).unwrap();
            self = self.step(ProverMessages::PresentationAckReceived(_ack()), connection_handle).unwrap();
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

        #[test]
        #[cfg(feature = "general_test")]
        fn test_prover_new() {
            let _setup = SetupMocks::init();

            let prover_sm = _prover_sm();

            assert_match!(ProverState::Initiated(_), prover_sm.state);
            assert_eq!(source_id(), prover_sm.source_id());
        }
    }

    mod step {
        use super::*;
        use crate::utils::constants::CREDS_FROM_PROOF_REQ;
        use crate::utils::mockdata::mock_settings::MockBuilder;

        #[test]
        #[cfg(feature = "general_test")]
        fn test_prover_init() {
            let _setup = SetupMocks::init();

            let prover_sm = _prover_sm();
            assert_match!(ProverState::Initiated(_), prover_sm.state);
        }

        #[test]
        #[cfg(feature = "general_test")]
        fn test_prover_handle_prepare_presentation_message_from_initiated_state() {
            let _setup = SetupMocks::init();

            let connection_handle = Some(mock_connection());
            let mut prover_sm = _prover_sm();
            prover_sm = prover_sm.step(ProverMessages::PreparePresentation((_credentials(), _self_attested())), connection_handle).unwrap();

            assert_match!(ProverState::PresentationPrepared(_), prover_sm.state);
        }

        #[test]
        #[cfg(feature = "general_test")]
        fn test_prover_handle_prepare_presentation_message_from_initiated_state_for_invalid_credentials() {
            let _setup = SetupMocks::init();
            let _mock_builder = MockBuilder::init().
                set_mock_creds_retrieved_for_proof_request(CREDS_FROM_PROOF_REQ);

            let connection_handle = Some(mock_connection());
            let mut prover_sm = _prover_sm();
            prover_sm = prover_sm.step(ProverMessages::PreparePresentation(("invalid".to_string(), _self_attested())), connection_handle).unwrap();

            assert_match!(ProverState::PresentationPreparationFailed(_), prover_sm.state);
        }

        #[test]
        #[cfg(feature = "general_test")]
        fn test_prover_handle_reject_presentation_request_message_from_initiated_state() {
            let _setup = SetupMocks::init();

            let connection_handle = Some(mock_connection());
            let mut prover_sm = _prover_sm();
            prover_sm = prover_sm.step(ProverMessages::RejectPresentationRequest((String::from("reject request"))), connection_handle).unwrap();

            assert_match!(ProverState::Finished(_), prover_sm.state);
        }

        #[test]
        #[cfg(feature = "general_test")]
        fn test_prover_handle_propose_presentation_message_from_initiated_state() {
            let _setup = SetupMocks::init();

            let connection_handle = Some(mock_connection());
            let mut prover_sm = _prover_sm();
            prover_sm = prover_sm.step(ProverMessages::ProposePresentation((_presentation_preview())), connection_handle).unwrap();

            assert_match!(ProverState::Finished(_), prover_sm.state);
        }

        #[test]
        #[cfg(feature = "general_test")]
        fn test_prover_handle_other_messages_from_initiated_state() {
            let _setup = SetupMocks::init();

            let connection_handle = Some(mock_connection());
            let mut prover_sm = _prover_sm();

            prover_sm = prover_sm.step(ProverMessages::SendPresentation, connection_handle).unwrap();
            assert_match!(ProverState::Initiated(_), prover_sm.state);

            prover_sm = prover_sm.step(ProverMessages::PresentationAckReceived(_ack()), connection_handle).unwrap();
            assert_match!(ProverState::Initiated(_), prover_sm.state);
        }

        #[test]
        #[cfg(feature = "general_test")]
        fn test_prover_handle_send_presentation_message_from_presentation_prepared_state() {
            let _setup = SetupMocks::init();

            let connection_handle = Some(mock_connection());
            let mut prover_sm = _prover_sm();

            prover_sm = prover_sm.step(ProverMessages::PreparePresentation((_credentials(), _self_attested())), connection_handle).unwrap();
            prover_sm = prover_sm.step(ProverMessages::SendPresentation, connection_handle).unwrap();

            assert_match!(ProverState::PresentationSent(_), prover_sm.state);
        }

        #[test]
        #[cfg(feature = "general_test")]
        fn test_prover_handle_send_presentation_message_from_presentation_prepared_state_for_presentation_request_contains_service_decorator() {
            let _setup = SetupMocks::init();

            let connection_handle = Some(mock_connection());
            let mut prover_sm = ProverSM::new(_presentation_request_with_service(), source_id());

            prover_sm = prover_sm.step(ProverMessages::PreparePresentation((_credentials(), _self_attested())), connection_handle).unwrap();
            prover_sm = prover_sm.step(ProverMessages::SendPresentation, connection_handle).unwrap();

            assert_match!(ProverState::Finished(_), prover_sm.state);
        }

        #[test]
        #[cfg(feature = "general_test")]
        fn test_prover_handle_other_messages_from_presentation_prepared_state() {
            let _setup = SetupMocks::init();

            let connection_handle = Some(mock_connection());
            let mut prover_sm = _prover_sm().to_presentation_prepared_state();

            prover_sm = prover_sm.step(ProverMessages::PresentationRejectReceived(_problem_report()), connection_handle).unwrap();
            assert_match!(ProverState::PresentationPrepared(_), prover_sm.state);

            prover_sm = prover_sm.step(ProverMessages::PresentationAckReceived(_ack()), connection_handle).unwrap();
            assert_match!(ProverState::PresentationPrepared(_), prover_sm.state);
        }

        #[test]
        #[cfg(feature = "general_test")]
        fn test_prover_handle_reject_presentation_request_message_from_presentation_prepared_state() {
            let _setup = SetupMocks::init();

            let connection_handle = Some(mock_connection());
            let mut prover_sm = _prover_sm().to_presentation_prepared_state();

            prover_sm = prover_sm.step(ProverMessages::RejectPresentationRequest((String::from("reject request"))), connection_handle).unwrap();

            assert_match!(ProverState::Finished(_), prover_sm.state);
        }

        #[test]
        #[cfg(feature = "general_test")]
        fn test_prover_handle_propose_presentation_message_from_presentation_prepared_state() {
            let _setup = SetupMocks::init();

            let connection_handle = Some(mock_connection());
            let mut prover_sm = _prover_sm().to_presentation_prepared_state();
            prover_sm = prover_sm.step(ProverMessages::ProposePresentation((_presentation_preview())), connection_handle).unwrap();

            assert_match!(ProverState::Finished(_), prover_sm.state);
        }

        #[test]
        #[cfg(feature = "general_test")]
        fn test_prover_handle_send_presentation_message_from_presentation_preparation_failed_state() {
            let _setup = SetupMocks::init();
            let _mock_builder = MockBuilder::init().
                set_mock_creds_retrieved_for_proof_request(CREDS_FROM_PROOF_REQ);

            let connection_handle = Some(mock_connection());
            let mut prover_sm = _prover_sm();
            prover_sm = prover_sm.step(ProverMessages::PreparePresentation(("invalid".to_string(), _self_attested())), connection_handle).unwrap();
            assert_match!(ProverState::PresentationPreparationFailed(_), prover_sm.state);

            prover_sm = prover_sm.step(ProverMessages::SendPresentation, connection_handle).unwrap();
            assert_match!(ProverState::Finished(_), prover_sm.state);
            assert_eq!(Status::Failed(ProblemReport::default()).code(), prover_sm.presentation_status());
        }

        #[test]
        #[cfg(feature = "general_test")]
        fn test_prover_handle_other_messages_from_presentation_preparation_failed_state() {
            let _setup = SetupMocks::init();
            let _mock_builder = MockBuilder::init().
                set_mock_creds_retrieved_for_proof_request(CREDS_FROM_PROOF_REQ);

            let connection_handle = Some(mock_connection());
            let mut prover_sm = _prover_sm();
            prover_sm = prover_sm.step(ProverMessages::PreparePresentation(("invalid".to_string(), _self_attested())), connection_handle).unwrap();

            prover_sm = prover_sm.step(ProverMessages::PresentationRejectReceived(_problem_report()), connection_handle).unwrap();
            assert_match!(ProverState::PresentationPreparationFailed(_), prover_sm.state);

            prover_sm = prover_sm.step(ProverMessages::PresentationAckReceived(_ack()), connection_handle).unwrap();
            assert_match!(ProverState::PresentationPreparationFailed(_), prover_sm.state);
        }

        #[test]
        #[cfg(feature = "general_test")]
        fn test_prover_handle_ack_message_from_presentation_sent_state() {
            let _setup = SetupMocks::init();

            let connection_handle = Some(mock_connection());
            let mut prover_sm = _prover_sm();
            prover_sm = prover_sm.step(ProverMessages::PreparePresentation((_credentials(), _self_attested())), connection_handle).unwrap();
            prover_sm = prover_sm.step(ProverMessages::SendPresentation, connection_handle).unwrap();
            prover_sm = prover_sm.step(ProverMessages::PresentationAckReceived(_ack()), connection_handle).unwrap();

            assert_match!(ProverState::Finished(_), prover_sm.state);
            assert_eq!(Status::Success.code(), prover_sm.presentation_status());
        }

        #[test]
        #[cfg(feature = "general_test")]
        fn test_prover_handle_reject_presentation_request_message_from_presentation_sent_state() {
            let _setup = SetupMocks::init();

            let connection_handle = Some(mock_connection());
            let prover_sm = _prover_sm().to_presentation_sent_state();
            let err = prover_sm.step(ProverMessages::RejectPresentationRequest((String::from("reject"))), connection_handle).unwrap_err();
            assert_eq!(VcxErrorKind::ActionNotSupported, err.kind());
        }

        #[test]
        #[cfg(feature = "general_test")]
        fn test_prover_handle_presentation_reject_message_from_presentation_sent_state() {
            let _setup = SetupMocks::init();

            let connection_handle = Some(mock_connection());
            let mut prover_sm = _prover_sm();
            prover_sm = prover_sm.step(ProverMessages::PreparePresentation((_credentials(), _self_attested())), connection_handle).unwrap();
            prover_sm = prover_sm.step(ProverMessages::SendPresentation, connection_handle).unwrap();
            prover_sm = prover_sm.step(ProverMessages::PresentationRejectReceived(_problem_report()), connection_handle).unwrap();

            assert_match!(ProverState::Finished(_), prover_sm.state);
            assert_eq!(Status::Failed(ProblemReport::create()).code(), prover_sm.presentation_status());
        }

        #[test]
        #[cfg(feature = "general_test")]
        fn test_prover_handle_other_messages_from_presentation_sent_state() {
            let _setup = SetupMocks::init();

            let connection_handle = Some(mock_connection());
            let mut prover_sm = _prover_sm();
            prover_sm = prover_sm.step(ProverMessages::PreparePresentation((_credentials(), _self_attested())), connection_handle).unwrap();
            prover_sm = prover_sm.step(ProverMessages::SendPresentation, connection_handle).unwrap();

            prover_sm = prover_sm.step(ProverMessages::PreparePresentation((_credentials(), _self_attested())), connection_handle).unwrap();
            assert_match!(ProverState::PresentationSent(_), prover_sm.state);

            prover_sm = prover_sm.step(ProverMessages::SendPresentation, connection_handle).unwrap();
            assert_match!(ProverState::PresentationSent(_), prover_sm.state);
        }

        #[test]
        #[cfg(feature = "general_test")]
        fn test_prover_handle_messages_from_finished_state() {
            let _setup = SetupMocks::init();

            let connection_handle = Some(mock_connection());
            let mut prover_sm = _prover_sm();
            prover_sm = prover_sm.step(ProverMessages::PreparePresentation((_credentials(), _self_attested())), connection_handle).unwrap();
            prover_sm = prover_sm.step(ProverMessages::SendPresentation, connection_handle).unwrap();
            prover_sm = prover_sm.step(ProverMessages::PresentationAckReceived(_ack()), connection_handle).unwrap();

            prover_sm = prover_sm.step(ProverMessages::PresentationAckReceived(_ack()), connection_handle).unwrap();
            assert_match!(ProverState::Finished(_), prover_sm.state);

            prover_sm = prover_sm.step(ProverMessages::PresentationRejectReceived(_problem_report()), connection_handle).unwrap();
            assert_match!(ProverState::Finished(_), prover_sm.state);
        }
    }

    mod find_message_to_handle {
        use super::*;

        #[test]
        #[cfg(feature = "general_test")]
        fn test_prover_find_message_to_handle_from_initiated_state() {
            let _setup = SetupMocks::init();

            let prover = _prover_sm();

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

        #[test]
        #[cfg(feature = "general_test")]
        fn test_prover_find_message_to_handle_from_presentation_prepared_state() {
            let _setup = SetupMocks::init();

            let prover = _prover_sm().to_presentation_prepared_state();

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

        #[test]
        #[cfg(feature = "general_test")]
        fn test_prover_find_message_to_handle_from_presentation_sent_state() {
            let _setup = SetupMocks::init();

            let prover = _prover_sm().to_presentation_sent_state();

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

        #[test]
        #[cfg(feature = "general_test")]
        fn test_prover_find_message_to_handle_from_finished_state() {
            let _setup = SetupMocks::init();

            let prover = _prover_sm().to_finished_state();

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

        #[test]
        #[cfg(feature = "general_test")]
        fn test_get_state() {
            let _setup = SetupMocks::init();

            assert_eq!(VcxStateType::VcxStateRequestReceived as u32, _prover_sm().state());
            assert_eq!(VcxStateType::VcxStateRequestReceived as u32, _prover_sm().to_presentation_prepared_state().state());
            assert_eq!(VcxStateType::VcxStateOfferSent as u32, _prover_sm().to_presentation_sent_state().state());
            assert_eq!(VcxStateType::VcxStateAccepted as u32, _prover_sm().to_finished_state().state());
        }
    }
}
