use std::collections::HashMap;
use std::fmt;
use std::sync::Arc;

use crate::errors::error::prelude::*;
use crate::handlers::proof_presentation::types::SelectedCredentials;
use crate::handlers::util::{make_attach_from_str, AttachmentId, PresentationProposalData, Status};
use crate::protocols::common::build_problem_report_msg;
use crate::protocols::proof_presentation::prover::states::finished::FinishedState;
use crate::protocols::proof_presentation::prover::states::initial::InitialProverState;
use crate::protocols::proof_presentation::prover::states::presentation_preparation_failed::PresentationPreparationFailedState;
use crate::protocols::proof_presentation::prover::states::presentation_prepared::PresentationPreparedState;
use crate::protocols::proof_presentation::prover::states::presentation_proposal_sent::PresentationProposalSent;
use crate::protocols::proof_presentation::prover::states::presentation_request_received::PresentationRequestReceived;
use crate::protocols::proof_presentation::prover::states::presentation_sent::PresentationSentState;

use aries_vcx_core::anoncreds::base_anoncreds::BaseAnonCreds;
use aries_vcx_core::ledger::base_ledger::AnoncredsLedgerRead;
use chrono::Utc;
use messages::decorators::thread::Thread;
use messages::decorators::timing::Timing;
use messages::msg_fields::protocols::present_proof::ack::AckPresentation;
use messages::msg_fields::protocols::present_proof::present::{
    Presentation, PresentationContent, PresentationDecorators,
};
use messages::msg_fields::protocols::present_proof::propose::{
    PresentationPreview, ProposePresentation, ProposePresentationContent, ProposePresentationDecorators,
};
use messages::msg_fields::protocols::present_proof::request::RequestPresentation;
use messages::msg_fields::protocols::report_problem::ProblemReport;
use uuid::Uuid;

/// A state machine that tracks the evolution of states for a Prover during
/// the Present Proof protocol.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Default)]
pub struct ProverSM {
    source_id: String,
    thread_id: String,
    state: ProverFullState,
}

#[derive(Debug, PartialEq, Eq)]
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

impl fmt::Display for ProverFullState {
    fn fmt(&self, f: &mut fmt::Formatter) -> Result<(), fmt::Error> {
        match *self {
            ProverFullState::Initial(_) => f.write_str("Initial"),
            ProverFullState::PresentationProposalSent(_) => f.write_str("PresentationProposalSent"),
            ProverFullState::PresentationRequestReceived(_) => f.write_str("PresentationRequestReceived"),
            ProverFullState::PresentationPrepared(_) => f.write_str("PresentationPrepared"),
            ProverFullState::PresentationPreparationFailed(_) => f.write_str("PresentationPreparationFailed"),
            ProverFullState::PresentationSent(_) => f.write_str("PresentationSent"),
            ProverFullState::Finished(_) => f.write_str("Finished"),
        }
    }
}

fn build_presentation_msg(thread_id: &str, presentation_attachment: String) -> VcxResult<Presentation> {
    let id = Uuid::new_v4().to_string();

    let content = PresentationContent::new(vec![make_attach_from_str!(
        &presentation_attachment,
        AttachmentId::Presentation.as_ref().to_string()
    )]);
    let mut decorators = PresentationDecorators::new(Thread::new(thread_id.to_owned()));
    let mut timing = Timing::default();
    timing.out_time = Some(Utc::now());
    decorators.timing = Some(timing);

    Ok(Presentation::with_decorators(id, content, decorators))
}

impl Default for ProverFullState {
    fn default() -> Self {
        Self::PresentationRequestReceived(PresentationRequestReceived::default())
    }
}

impl ProverSM {
    pub fn new(source_id: String) -> ProverSM {
        ProverSM {
            source_id,
            thread_id: Uuid::new_v4().to_string(),
            state: ProverFullState::Initial(InitialProverState {}),
        }
    }

    pub fn from_request(presentation_request: RequestPresentation, source_id: String) -> ProverSM {
        ProverSM {
            source_id,
            thread_id: presentation_request.id.clone(),
            state: ProverFullState::PresentationRequestReceived(PresentationRequestReceived { presentation_request }),
        }
    }

    pub async fn build_presentation_proposal(self, proposal_data: PresentationProposalData) -> VcxResult<Self> {
        let state = match self.state {
            ProverFullState::Initial(_) => {
                let id = self.thread_id.clone();
                let preview = PresentationPreview::new(proposal_data.attributes, proposal_data.predicates);
                let mut content = ProposePresentationContent::new(preview);
                content.comment = proposal_data.comment;

                let decorators = ProposePresentationDecorators::default();

                let proposal = ProposePresentation::with_decorators(id, content, decorators);
                ProverFullState::PresentationProposalSent(PresentationProposalSent::new(proposal))
            }
            ProverFullState::PresentationRequestReceived(_) => {
                let id = Uuid::new_v4().to_string();
                let preview = PresentationPreview::new(proposal_data.attributes, proposal_data.predicates);
                let mut content = ProposePresentationContent::new(preview);
                content.comment = proposal_data.comment;

                let mut decorators = ProposePresentationDecorators::default();
                decorators.thread = Some(Thread::new(self.thread_id.clone()));

                let proposal = ProposePresentation::with_decorators(id, content, decorators);
                ProverFullState::PresentationProposalSent(PresentationProposalSent::new(proposal))
            }
            s => {
                warn!("Unable to set presentation proposal in state {}", s);
                s
            }
        };
        Ok(Self { state, ..self })
    }

    pub async fn decline_presentation_request(self, problem_report: ProblemReport) -> VcxResult<Self> {
        let state = match self.state {
            ProverFullState::PresentationRequestReceived(state) => {
                ProverFullState::Finished((state, problem_report).into())
            }
            ProverFullState::PresentationPrepared(_) => {
                ProverFullState::Finished(FinishedState::declined(problem_report))
            }
            s => {
                warn!("Unable to decline presentation request in state {}", s);
                s
            }
        };
        Ok(Self { state, ..self })
    }

    pub async fn negotiate_presentation(self) -> VcxResult<Self> {
        let state = match self.state {
            ProverFullState::PresentationRequestReceived(state) => ProverFullState::Finished(state.into()),
            ProverFullState::PresentationPrepared(state) => ProverFullState::Finished(state.into()),
            s => {
                warn!("Unable to send handle presentation proposal in state {}", s);
                s
            }
        };
        Ok(Self { state, ..self })
    }

    pub async fn generate_presentation(
        self,
        ledger: &Arc<dyn AnoncredsLedgerRead>,
        anoncreds: &Arc<dyn BaseAnonCreds>,
        credentials: SelectedCredentials,
        self_attested_attrs: HashMap<String, String>,
    ) -> VcxResult<Self> {
        let state = match self.state {
            ProverFullState::PresentationRequestReceived(state) => {
                match state
                    .build_presentation(ledger, anoncreds, &credentials, &self_attested_attrs)
                    .await
                {
                    Ok(presentation) => {
                        let presentation = build_presentation_msg(&self.thread_id, presentation)?;
                        ProverFullState::PresentationPrepared((state, presentation).into())
                    }
                    Err(err) => {
                        let problem_report = build_problem_report_msg(Some(err.to_string()), &self.thread_id);
                        error!(
                            "Failed bo build presentation, sending problem report: {:?}",
                            problem_report
                        );
                        ProverFullState::PresentationPreparationFailed((state, problem_report).into())
                    }
                }
            }
            s => {
                warn!("Unable to send generate presentation in state {}", s);
                s
            }
        };
        Ok(Self { state, ..self })
    }

    pub async fn mark_presentation_sent(self) -> VcxResult<Self> {
        let state = match self.state {
            ProverFullState::PresentationPrepared(state) => ProverFullState::PresentationSent((state).into()),
            ProverFullState::PresentationPreparationFailed(state) => ProverFullState::Finished((state).into()),
            s => {
                warn!("Unable to send send presentation in state {}", s);
                s
            }
        };
        Ok(Self { state, ..self })
    }

    pub fn get_problem_report(&self) -> VcxResult<ProblemReport> {
        match &self.state {
            ProverFullState::Finished(state) => match &state.status {
                Status::Failed(problem_report) | Status::Declined(problem_report) => Ok(problem_report.clone()),
                _ => Err(AriesVcxError::from_msg(
                    AriesVcxErrorKind::NotReady,
                    "Cannot get problem report",
                )),
            },
            _ => Err(AriesVcxError::from_msg(
                AriesVcxErrorKind::NotReady,
                "Cannot get problem report",
            )),
        }
    }

    pub fn receive_presentation_request(self, request: RequestPresentation) -> VcxResult<ProverSM> {
        let prover_sm = match &self.state {
            ProverFullState::PresentationProposalSent(_) => {
                let state = ProverFullState::PresentationRequestReceived(PresentationRequestReceived::new(request));
                ProverSM { state, ..self }
            }
            _ => {
                warn!("Not supported in this state");
                self
            }
        };
        Ok(prover_sm)
    }

    pub fn receive_presentation_reject(self, problem_report: ProblemReport) -> VcxResult<ProverSM> {
        let prover_sm = match &self.state {
            ProverFullState::PresentationProposalSent(_) => {
                let state = ProverFullState::Finished(FinishedState::declined(problem_report));
                ProverSM { state, ..self }
            }
            ProverFullState::PresentationSent(state) => {
                let state = ProverFullState::Finished((state.clone(), problem_report).into());
                ProverSM { state, ..self }
            }
            _ => {
                warn!("Not supported in this state");
                self
            }
        };
        Ok(prover_sm)
    }

    pub fn receive_presentation_ack(self, ack: AckPresentation) -> VcxResult<Self> {
        let state = match self.state {
            ProverFullState::PresentationSent(state) => ProverFullState::Finished((state, ack).into()),
            s => {
                warn!("Unable to process presentation ack in state {}", s);
                s
            }
        };
        Ok(Self { state, ..self })
    }

    pub fn source_id(&self) -> String {
        self.source_id.clone()
    }

    pub fn get_thread_id(&self) -> VcxResult<String> {
        Ok(self.thread_id.clone())
    }

    pub fn get_state(&self) -> ProverState {
        match self.state {
            ProverFullState::Initial(_) => ProverState::Initial,
            ProverFullState::PresentationProposalSent(_) => ProverState::PresentationProposalSent,
            ProverFullState::PresentationRequestReceived(_) => ProverState::PresentationRequestReceived,
            ProverFullState::PresentationPrepared(_) => ProverState::PresentationPrepared,
            ProverFullState::PresentationPreparationFailed(_) => ProverState::PresentationPreparationFailed,
            ProverFullState::PresentationSent(_) => ProverState::PresentationSent,
            ProverFullState::Finished(ref status) => match status.status {
                Status::Success => ProverState::Finished,
                _ => ProverState::Failed,
            },
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

    pub fn get_presentation_status(&self) -> u32 {
        match self.state {
            ProverFullState::Finished(ref state) => state.status.code(),
            _ => Status::Undefined.code(),
        }
    }

    pub fn get_presentation_request(&self) -> VcxResult<&RequestPresentation> {
        match self.state {
            ProverFullState::Initial(_) => Err(AriesVcxError::from_msg(
                AriesVcxErrorKind::NotReady,
                "Presentation request is not available",
            )),
            ProverFullState::PresentationProposalSent(_) => Err(AriesVcxError::from_msg(
                AriesVcxErrorKind::NotReady,
                "Presentation request is not available",
            )),
            ProverFullState::PresentationRequestReceived(ref state) => Ok(&state.presentation_request),
            ProverFullState::PresentationPrepared(ref state) => Ok(&state.presentation_request),
            ProverFullState::PresentationPreparationFailed(ref state) => Ok(&state.presentation_request),
            ProverFullState::PresentationSent(ref state) => Ok(&state.presentation_request),
            ProverFullState::Finished(ref state) => Ok(state.presentation_request.as_ref().ok_or(
                AriesVcxError::from_msg(AriesVcxErrorKind::NotReady, "Presentation request is not available"),
            )?),
        }
    }

    pub fn get_presentation_msg(&self) -> VcxResult<&Presentation> {
        match self.state {
            ProverFullState::Initial(_) => Err(AriesVcxError::from_msg(
                AriesVcxErrorKind::NotReady,
                "Presentation is not created yet",
            )),
            ProverFullState::PresentationProposalSent(_) => Err(AriesVcxError::from_msg(
                AriesVcxErrorKind::NotReady,
                "Presentation is not created yet",
            )),
            ProverFullState::PresentationRequestReceived(_) => Err(AriesVcxError::from_msg(
                AriesVcxErrorKind::NotReady,
                "Presentation is not created yet",
            )),
            ProverFullState::PresentationPrepared(ref state) => Ok(&state.presentation),
            ProverFullState::PresentationPreparationFailed(_) => Err(AriesVcxError::from_msg(
                AriesVcxErrorKind::NotReady,
                "Presentation is not created yet",
            )),
            ProverFullState::PresentationSent(ref state) => Ok(&state.presentation),
            ProverFullState::Finished(ref state) => Ok(state.presentation.as_ref().ok_or(AriesVcxError::from_msg(
                AriesVcxErrorKind::NotReady,
                "Presentation is not available in Finished state",
            ))?),
        }
    }

    pub fn get_proposal(&self) -> VcxResult<ProposePresentation> {
        match &self.state {
            ProverFullState::PresentationProposalSent(state) => Ok(state.proposal.clone()),
            _ => Err(AriesVcxError::from_msg(
                AriesVcxErrorKind::NotReady,
                "Cannot get proposal",
            )),
        }
    }
}
