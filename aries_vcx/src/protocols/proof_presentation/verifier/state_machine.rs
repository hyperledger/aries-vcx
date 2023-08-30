use std::fmt::Display;
use std::sync::Arc;

use chrono::Utc;
use uuid::Uuid;

use aries_vcx_core::anoncreds::base_anoncreds::BaseAnonCreds;
use aries_vcx_core::ledger::base_ledger::AnoncredsLedgerRead;
use messages::decorators::thread::Thread;
use messages::decorators::timing::Timing;
use messages::msg_fields::protocols::notification::ack::{AckDecorators, AckStatus};
use messages::msg_fields::protocols::present_proof::ack::{AckPresentation, AckPresentationContent};
use messages::msg_fields::protocols::present_proof::problem_report::{
    PresentProofProblemReport, PresentProofProblemReportContent,
};
use messages::msg_fields::protocols::present_proof::request::{
    RequestPresentation, RequestPresentationContent, RequestPresentationDecorators,
};
use messages::msg_fields::protocols::present_proof::{
    present::Presentation, propose::ProposePresentation, PresentProof,
};
use messages::msg_fields::protocols::report_problem::ProblemReport;
use messages::msg_parts::MsgParts;
use messages::AriesMessage;

use crate::common::proofs::proof_request::PresentationRequestData;
use crate::errors::error::prelude::*;
use crate::handlers::util::{make_attach_from_str, verify_thread_id, AttachmentId, Status};
use crate::protocols::common::build_problem_report_msg;
use crate::protocols::proof_presentation::verifier::states::finished::FinishedState;
use crate::protocols::proof_presentation::verifier::states::initial::InitialVerifierState;
use crate::protocols::proof_presentation::verifier::states::presentation_proposal_received::PresentationProposalReceivedState;
use crate::protocols::proof_presentation::verifier::states::presentation_request_sent::PresentationRequestSentState;
use crate::protocols::proof_presentation::verifier::states::presentation_request_set::PresentationRequestSetState;
use crate::protocols::proof_presentation::verifier::verification_status::PresentationVerificationStatus;
use crate::protocols::SendClosure;

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

pub fn build_verification_ack(thread_id: &str) -> AckPresentation {
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
    let id = thread_id.to_owned();

    let mut content = RequestPresentationContent::new(vec![make_attach_from_str!(
        &json!(request_data).to_string(),
        AttachmentId::PresentationRequest.as_ref().to_string()
    )]);
    content.comment = comment;

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
            &AriesMessage::PresentProof(PresentProof::ProposePresentation(proposal.clone())),
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
            VerifierFullState::PresentationRequestSent(_) => (
                VerifierFullState::PresentationProposalReceived(PresentationProposalReceivedState::new(proposal)),
                self.thread_id.clone(),
            ),
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
        verify_thread_id(&self.thread_id, &AriesMessage::ReportProblem(problem_report.clone()))?;
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

    pub async fn verify_presentation<'a>(
        self,
        ledger: &'a Arc<dyn AnoncredsLedgerRead>,
        anoncreds: &'a Arc<dyn BaseAnonCreds>,
        presentation: Presentation,
        send_message: SendClosure,
    ) -> VcxResult<Self> {
        verify_thread_id(
            &self.thread_id,
            &AriesMessage::PresentProof(PresentProof::Presentation(presentation.clone())),
        )?;
        let state = match self.state {
            VerifierFullState::PresentationRequestSent(state) => {
                let verification_result = state
                    .verify_presentation(ledger, anoncreds, &presentation, &self.thread_id)
                    .await;

                let (sm, message) = match verification_result {
                    Ok(()) => {
                        let sm = VerifierFullState::Finished(
                            (state, presentation, PresentationVerificationStatus::Valid).into(),
                        );
                        let ack = build_verification_ack(&self.thread_id).into();
                        (sm, ack)
                    }
                    Err(err) => {
                        let problem_report = build_problem_report_msg(Some(err.to_string()), &self.thread_id);

                        let sm = match err.kind() {
                            AriesVcxErrorKind::InvalidProof => VerifierFullState::Finished(
                                (state, presentation, PresentationVerificationStatus::Invalid).into(),
                            ),
                            _ => VerifierFullState::Finished((state, problem_report.clone()).into()),
                        };

                        let MsgParts {
                            id,
                            content,
                            decorators,
                        } = problem_report;

                        let problem_report = PresentProofProblemReport::with_decorators(
                            id,
                            PresentProofProblemReportContent(content),
                            decorators,
                        );

                        (sm, AriesMessage::from(problem_report))
                    }
                };
                send_message(message).await?;
                sm
            }
            s => {
                warn!("Unable to verify presentation in state {}", s);
                s
            }
        };
        Ok(Self { state, ..self })
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
                ));
            }
        };
        Ok(Self {
            source_id,
            thread_id,
            state,
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
