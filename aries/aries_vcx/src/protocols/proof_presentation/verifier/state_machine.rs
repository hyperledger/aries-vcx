use std::fmt::Display;

use anoncreds_types::data_types::messages::pres_request::PresentationRequest;
use aries_vcx_core::{
    anoncreds::base_anoncreds::BaseAnonCreds, ledger::base_ledger::AnoncredsLedgerRead,
};
use chrono::Utc;
use messages::{
    decorators::{thread::Thread, timing::Timing},
    msg_fields::protocols::{
        notification::ack::{AckContent, AckDecorators, AckStatus},
        present_proof::v1::{
            ack::AckPresentationV1,
            present::PresentationV1,
            problem_report::PresentProofV1ProblemReport,
            propose::ProposePresentationV1,
            request::{
                RequestPresentationV1, RequestPresentationV1Content,
                RequestPresentationV1Decorators,
            },
        },
        report_problem::ProblemReport,
    },
    AriesMessage,
};
use uuid::Uuid;

use crate::{
    errors::error::prelude::*,
    handlers::util::{make_attach_from_str, verify_thread_id, AttachmentId, Status},
    protocols::{
        common::build_problem_report_msg,
        proof_presentation::verifier::{
            states::{
                finished::FinishedState, initial::InitialVerifierState,
                presentation_proposal_received::PresentationProposalReceivedState,
                presentation_request_sent::PresentationRequestSentState,
                presentation_request_set::PresentationRequestSetState,
            },
            verification_status::PresentationVerificationStatus,
        },
    },
};

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
            VerifierFullState::PresentationProposalReceived(_) => {
                f.write_str("PresentationProposalReceived")
            }
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

pub fn build_verification_ack(thread_id: &str) -> AckPresentationV1 {
    let content = AckContent::builder().status(AckStatus::Ok).build();

    let decorators = AckDecorators::builder()
        .thread(Thread::builder().thid(thread_id.to_owned()).build())
        .timing(Timing::builder().out_time(Utc::now()).build())
        .build();

    AckPresentationV1::builder()
        .id(Uuid::new_v4().to_string())
        .content(content)
        .decorators(decorators)
        .build()
}

pub fn build_starting_presentation_request(
    thread_id: &str,
    request_data: &PresentationRequest,
    comment: Option<String>,
) -> VcxResult<RequestPresentationV1> {
    let id = thread_id.to_owned();

    let content = RequestPresentationV1Content::builder().request_presentations_attach(vec![
        make_attach_from_str!(
            &json!(request_data).to_string(),
            AttachmentId::PresentationRequest.as_ref().to_string()
        ),
    ]);

    let content = if let Some(comment) = comment {
        content.comment(comment).build()
    } else {
        content.build()
    };

    let decorators = RequestPresentationV1Decorators::builder()
        .timing(Timing::builder().out_time(Utc::now()).build())
        .build();

    Ok(RequestPresentationV1::builder()
        .id(id)
        .content(content)
        .decorators(decorators)
        .build())
}

impl VerifierSM {
    pub fn new(source_id: &str) -> Self {
        Self {
            thread_id: String::new(),
            source_id: source_id.to_string(),
            state: VerifierFullState::Initial(InitialVerifierState {}),
        }
    }

    // todo: eliminate VcxResult (follow set_request err chain and eliminate possibility of err at
    // the bottom)
    pub fn from_request(
        source_id: &str,
        presentation_request_data: &PresentationRequest,
    ) -> VcxResult<Self> {
        let sm = Self {
            source_id: source_id.to_string(),
            thread_id: Uuid::new_v4().to_string(),
            state: VerifierFullState::Initial(InitialVerifierState {}),
        };
        sm.set_presentation_request(presentation_request_data, None)
    }

    pub fn from_proposal(source_id: &str, presentation_proposal: &ProposePresentationV1) -> Self {
        Self {
            source_id: source_id.to_string(),
            thread_id: presentation_proposal.id.clone(),
            state: VerifierFullState::PresentationProposalReceived(
                PresentationProposalReceivedState::new(presentation_proposal.clone()),
            ),
        }
    }

    pub fn receive_presentation_proposal(self, proposal: ProposePresentationV1) -> VcxResult<Self> {
        verify_thread_id(&self.thread_id, &proposal.clone().into())?;
        let (state, thread_id) = match self.state {
            VerifierFullState::Initial(_) => {
                let thread_id = match proposal.decorators.thread {
                    Some(ref thread) => thread.thid.clone(),
                    None => proposal.id.clone(),
                };
                (
                    VerifierFullState::PresentationProposalReceived(
                        PresentationProposalReceivedState::new(proposal),
                    ),
                    thread_id,
                )
            }
            VerifierFullState::PresentationRequestSent(_) => (
                VerifierFullState::PresentationProposalReceived(
                    PresentationProposalReceivedState::new(proposal),
                ),
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

    pub fn receive_presentation_request_reject(
        self,
        problem_report: ProblemReport,
    ) -> VcxResult<Self> {
        verify_thread_id(
            &self.thread_id,
            &AriesMessage::ReportProblem(problem_report.clone()),
        )?;
        let state = match self.state {
            VerifierFullState::PresentationRequestSent(state) => {
                VerifierFullState::Finished((state, problem_report).into())
            }
            s => {
                warn!(
                    "Unable to receive presentation request reject in state {}",
                    s
                );
                s
            }
        };
        Ok(Self { state, ..self })
    }

    pub async fn reject_presentation_proposal(
        self,
        problem_report: ProblemReport,
    ) -> VcxResult<Self> {
        let (state, thread_id) = match self.state {
            VerifierFullState::PresentationProposalReceived(state) => {
                let thread_id = match state.presentation_proposal.decorators.thread {
                    Some(thread) => thread.thid,
                    None => state.presentation_proposal.id,
                };
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
        ledger: &'a impl AnoncredsLedgerRead,
        anoncreds: &'a impl BaseAnonCreds,
        presentation: PresentationV1,
    ) -> VcxResult<Self> {
        verify_thread_id(&self.thread_id, &presentation.clone().into())?;
        let state = match self.state {
            VerifierFullState::PresentationRequestSent(state) => {
                let verification_result = state
                    .verify_presentation(ledger, anoncreds, &presentation, &self.thread_id)
                    .await;

                match verification_result {
                    Ok(()) => VerifierFullState::Finished(
                        (state, presentation, PresentationVerificationStatus::Valid).into(),
                    ),
                    Err(err) => {
                        let problem_report =
                            build_problem_report_msg(Some(err.to_string()), &self.thread_id);

                        match err.kind() {
                            AriesVcxErrorKind::InvalidProof => VerifierFullState::Finished(
                                (state, presentation, PresentationVerificationStatus::Invalid)
                                    .into(),
                            ),
                            _ => VerifierFullState::Finished((state, problem_report).into()),
                        }
                    }
                }
            }
            s => {
                warn!("Unable to verify presentation in state {}", s);
                s
            }
        };
        Ok(Self { state, ..self })
    }

    pub fn get_final_message(&self) -> VcxResult<AriesMessage> {
        match &self.state {
            VerifierFullState::Finished(ref state) => match &state.verification_status {
                PresentationVerificationStatus::Valid => {
                    Ok(build_verification_ack(&self.thread_id).into())
                }
                PresentationVerificationStatus::Invalid
                | PresentationVerificationStatus::Unavailable => match &state.status {
                    Status::Undefined => Err(AriesVcxError::from_msg(
                        AriesVcxErrorKind::InvalidState,
                        "Cannot get final message in this state: finished, status undefined",
                    )),
                    Status::Success => Ok(build_problem_report_msg(None, &self.thread_id).into()),
                    Status::Failed(problem_report) | Status::Declined(problem_report) => {
                        let problem_report = PresentProofV1ProblemReport::builder()
                            .id(problem_report.id.clone())
                            .content(problem_report.content.clone().into())
                            .decorators(problem_report.decorators.clone())
                            .build();

                        Ok(problem_report)
                    }
                },
            },
            s => Err(AriesVcxError::from_msg(
                AriesVcxErrorKind::InvalidState,
                format!("Cannot get final message in this state: {:?}", s),
            )),
        }
    }

    pub fn set_presentation_request(
        self,
        request_data: &PresentationRequest,
        comment: Option<String>,
    ) -> VcxResult<Self> {
        let Self {
            source_id,
            thread_id,
            state,
        } = self;
        let state = match state {
            VerifierFullState::Initial(_)
            | VerifierFullState::PresentationRequestSet(_)
            | VerifierFullState::PresentationProposalReceived(_) => {
                let presentation_request =
                    build_starting_presentation_request(&thread_id, request_data, comment)?;
                VerifierFullState::PresentationRequestSet(PresentationRequestSetState::new(
                    presentation_request,
                ))
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

    pub fn mark_presentation_request_sent(self) -> VcxResult<Self> {
        let Self {
            state,
            source_id,
            thread_id,
        } = self;
        let state = match state {
            VerifierFullState::PresentationRequestSet(state) => {
                VerifierFullState::PresentationRequestSent(state.into())
            }
            VerifierFullState::PresentationRequestSent(state) => {
                VerifierFullState::PresentationRequestSent(state)
            }
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
            VerifierFullState::PresentationProposalReceived(_) => {
                VerifierState::PresentationProposalReceived
            }
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

    pub fn presentation_request_msg(&self) -> VcxResult<RequestPresentationV1> {
        match self.state {
            VerifierFullState::Initial(_) => Err(AriesVcxError::from_msg(
                AriesVcxErrorKind::InvalidState,
                "Presentation request not set yet",
            )),
            VerifierFullState::PresentationRequestSet(ref state) => {
                Ok(state.presentation_request.clone())
            }
            VerifierFullState::PresentationProposalReceived(ref state) => state
                .presentation_request
                .clone()
                .ok_or(AriesVcxError::from_msg(
                    AriesVcxErrorKind::InvalidState,
                    "No presentation request set",
                )),
            VerifierFullState::PresentationRequestSent(ref state) => {
                Ok(state.presentation_request.clone())
            }
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

    pub fn get_presentation_msg(&self) -> VcxResult<PresentationV1> {
        match self.state {
            VerifierFullState::Finished(ref state) => {
                state.presentation.clone().ok_or(AriesVcxError::from_msg(
                    AriesVcxErrorKind::InvalidState,
                    "State machine is final state, but presentation is not available".to_string(),
                ))
            }
            _ => Err(AriesVcxError::from_msg(
                AriesVcxErrorKind::InvalidState,
                "Presentation not received yet",
            )),
        }
    }

    pub fn presentation_proposal(&self) -> VcxResult<ProposePresentationV1> {
        match self.state {
            VerifierFullState::PresentationProposalReceived(ref state) => {
                Ok(state.presentation_proposal.clone())
            }
            _ => Err(AriesVcxError::from_msg(
                AriesVcxErrorKind::InvalidState,
                "Presentation proposal not received yet",
            )),
        }
    }
}
