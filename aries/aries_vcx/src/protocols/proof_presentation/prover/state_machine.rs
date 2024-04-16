use std::{collections::HashMap, fmt};

use anoncreds_types::data_types::messages::{
    cred_selection::SelectedCredentials, presentation::Presentation,
};
use aries_vcx_core::anoncreds::base_anoncreds::BaseAnonCreds;
use aries_vcx_ledger::ledger::base_ledger::AnoncredsLedgerRead;
use aries_vcx_wallet::wallet::base_wallet::BaseWallet;
use chrono::Utc;
use messages::{
    decorators::{thread::Thread, timing::Timing},
    msg_fields::protocols::{
        present_proof::v1::{
            ack::AckPresentationV1,
            present::{PresentationV1, PresentationV1Content, PresentationV1Decorators},
            propose::{
                PresentationPreview, ProposePresentationV1, ProposePresentationV1Content,
                ProposePresentationV1Decorators,
            },
            request::RequestPresentationV1,
        },
        report_problem::ProblemReport,
    },
};
use uuid::Uuid;

use crate::{
    errors::error::prelude::*,
    handlers::util::{make_attach_from_str, AttachmentId, PresentationProposalData, Status},
    protocols::{
        common::build_problem_report_msg,
        proof_presentation::prover::states::{
            finished::FinishedState, initial::InitialProverState,
            presentation_preparation_failed::PresentationPreparationFailedState,
            presentation_prepared::PresentationPreparedState,
            presentation_proposal_sent::PresentationProposalSent,
            presentation_request_received::PresentationRequestReceived,
            presentation_sent::PresentationSentState,
        },
    },
};

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
            ProverFullState::PresentationRequestReceived(_) => {
                f.write_str("PresentationRequestReceived")
            }
            ProverFullState::PresentationPrepared(_) => f.write_str("PresentationPrepared"),
            ProverFullState::PresentationPreparationFailed(_) => {
                f.write_str("PresentationPreparationFailed")
            }
            ProverFullState::PresentationSent(_) => f.write_str("PresentationSent"),
            ProverFullState::Finished(_) => f.write_str("Finished"),
        }
    }
}

fn build_presentation_msg(
    thread_id: &str,
    presentation: Presentation,
) -> VcxResult<PresentationV1> {
    let id = Uuid::new_v4().to_string();

    let content = PresentationV1Content::builder()
        .presentations_attach(vec![make_attach_from_str!(
            &serde_json::to_string(&presentation)?,
            AttachmentId::Presentation.as_ref().to_string()
        )])
        .build();

    let decorators = PresentationV1Decorators::builder()
        .thread(Thread::builder().thid(thread_id.to_owned()).build())
        .timing(Timing::builder().out_time(Utc::now()).build())
        .build();

    Ok(PresentationV1::builder()
        .id(id)
        .content(content)
        .decorators(decorators)
        .build())
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

    pub fn from_request(
        presentation_request: RequestPresentationV1,
        source_id: String,
    ) -> ProverSM {
        ProverSM {
            source_id,
            thread_id: presentation_request.id.clone(),
            state: ProverFullState::PresentationRequestReceived(PresentationRequestReceived {
                presentation_request,
            }),
        }
    }

    pub async fn build_presentation_proposal(
        self,
        proposal_data: PresentationProposalData,
    ) -> VcxResult<Self> {
        let state = match self.state {
            ProverFullState::Initial(_) => {
                let id = self.thread_id.clone();
                let preview =
                    PresentationPreview::new(proposal_data.attributes, proposal_data.predicates);
                let content =
                    ProposePresentationV1Content::builder().presentation_proposal(preview);

                let content = if let Some(comment) = proposal_data.comment {
                    content.comment(comment).build()
                } else {
                    content.build()
                };

                let proposal = ProposePresentationV1::builder()
                    .id(id)
                    .content(content)
                    .build();
                ProverFullState::PresentationProposalSent(PresentationProposalSent::new(proposal))
            }
            ProverFullState::PresentationRequestReceived(_) => {
                let id = Uuid::new_v4().to_string();
                let preview =
                    PresentationPreview::new(proposal_data.attributes, proposal_data.predicates);

                let content =
                    ProposePresentationV1Content::builder().presentation_proposal(preview);
                let content = if let Some(comment) = proposal_data.comment {
                    content.comment(comment).build()
                } else {
                    content.build()
                };

                let decorators = ProposePresentationV1Decorators::builder()
                    .thread(Thread::builder().thid(self.thread_id.clone()).build())
                    .build();

                let proposal = ProposePresentationV1::builder()
                    .id(id)
                    .content(content)
                    .decorators(decorators)
                    .build();
                ProverFullState::PresentationProposalSent(PresentationProposalSent::new(proposal))
            }
            s => {
                warn!("Unable to set presentation proposal in state {}", s);
                s
            }
        };
        Ok(Self { state, ..self })
    }

    pub async fn decline_presentation_request(
        self,
        problem_report: ProblemReport,
    ) -> VcxResult<Self> {
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
            ProverFullState::PresentationRequestReceived(state) => {
                ProverFullState::Finished(state.into())
            }
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
        wallet: &impl BaseWallet,
        ledger: &impl AnoncredsLedgerRead,
        anoncreds: &impl BaseAnonCreds,
        credentials: SelectedCredentials,
        self_attested_attrs: HashMap<String, String>,
    ) -> VcxResult<Self> {
        let state = match self.state {
            ProverFullState::PresentationRequestReceived(state) => {
                match state
                    .build_presentation(
                        wallet,
                        ledger,
                        anoncreds,
                        &credentials,
                        self_attested_attrs,
                    )
                    .await
                {
                    Ok(presentation) => {
                        let presentation = build_presentation_msg(&self.thread_id, presentation)?;
                        ProverFullState::PresentationPrepared((state, presentation).into())
                    }
                    Err(err) => {
                        let problem_report =
                            build_problem_report_msg(Some(err.to_string()), &self.thread_id);
                        error!(
                            "Failed bo build presentation, sending problem report: {:?}",
                            problem_report
                        );
                        ProverFullState::PresentationPreparationFailed(
                            (state, problem_report).into(),
                        )
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

    pub fn mark_presentation_sent(self) -> VcxResult<Self> {
        let state = match self.state {
            ProverFullState::PresentationPrepared(state) => {
                ProverFullState::PresentationSent((state).into())
            }
            ProverFullState::PresentationPreparationFailed(state) => {
                ProverFullState::Finished((state).into())
            }
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
                Status::Failed(problem_report) | Status::Declined(problem_report) => {
                    Ok(problem_report.clone())
                }
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

    pub fn receive_presentation_request(
        self,
        request: RequestPresentationV1,
    ) -> VcxResult<ProverSM> {
        let prover_sm = match &self.state {
            ProverFullState::PresentationProposalSent(_) => {
                let state = ProverFullState::PresentationRequestReceived(
                    PresentationRequestReceived::new(request),
                );
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

    pub fn receive_presentation_ack(self, ack: AckPresentationV1) -> VcxResult<Self> {
        let state = match self.state {
            ProverFullState::PresentationSent(state) => {
                ProverFullState::Finished((state, ack).into())
            }
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
            ProverFullState::PresentationRequestReceived(_) => {
                ProverState::PresentationRequestReceived
            }
            ProverFullState::PresentationPrepared(_) => ProverState::PresentationPrepared,
            ProverFullState::PresentationPreparationFailed(_) => {
                ProverState::PresentationPreparationFailed
            }
            ProverFullState::PresentationSent(_) => ProverState::PresentationSent,
            ProverFullState::Finished(ref status) => match status.status {
                Status::Success => ProverState::Finished,
                _ => ProverState::Failed,
            },
        }
    }

    pub fn progressable_by_message(&self) -> bool {
        trace!(
            "Prover::states::progressable_by_message >> state: {:?}",
            self.state
        );
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

    pub fn get_presentation_request(&self) -> VcxResult<&RequestPresentationV1> {
        match self.state {
            ProverFullState::Initial(_) => Err(AriesVcxError::from_msg(
                AriesVcxErrorKind::NotReady,
                "Presentation request is not available",
            )),
            ProverFullState::PresentationProposalSent(_) => Err(AriesVcxError::from_msg(
                AriesVcxErrorKind::NotReady,
                "Presentation request is not available",
            )),
            ProverFullState::PresentationRequestReceived(ref state) => {
                Ok(&state.presentation_request)
            }
            ProverFullState::PresentationPrepared(ref state) => Ok(&state.presentation_request),
            ProverFullState::PresentationPreparationFailed(ref state) => {
                Ok(&state.presentation_request)
            }
            ProverFullState::PresentationSent(ref state) => Ok(&state.presentation_request),
            ProverFullState::Finished(ref state) => {
                Ok(state
                    .presentation_request
                    .as_ref()
                    .ok_or(AriesVcxError::from_msg(
                        AriesVcxErrorKind::NotReady,
                        "Presentation request is not available",
                    ))?)
            }
        }
    }

    pub fn get_presentation_msg(&self) -> VcxResult<&PresentationV1> {
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
            ProverFullState::Finished(ref state) => {
                Ok(state.presentation.as_ref().ok_or(AriesVcxError::from_msg(
                    AriesVcxErrorKind::NotReady,
                    "Presentation is not available in Finished state",
                ))?)
            }
        }
    }

    pub fn get_presentation_proposal(&self) -> VcxResult<ProposePresentationV1> {
        match &self.state {
            ProverFullState::PresentationProposalSent(state) => Ok(state.proposal.clone()),
            _ => Err(AriesVcxError::from_msg(
                AriesVcxErrorKind::NotReady,
                "Cannot get proposal",
            )),
        }
    }
}
