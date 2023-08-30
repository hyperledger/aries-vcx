use std::collections::HashMap;
use std::sync::Arc;

use aries_vcx_core::anoncreds::base_anoncreds::BaseAnonCreds;
use aries_vcx_core::ledger::base_ledger::AnoncredsLedgerRead;
use chrono::Utc;
use messages::decorators::thread::Thread;
use messages::decorators::timing::Timing;
use messages::msg_fields::protocols::notification::Notification;
use messages::msg_fields::protocols::present_proof::ack::AckPresentation;
use messages::msg_fields::protocols::present_proof::present::Presentation;
use messages::msg_fields::protocols::present_proof::propose::{
    PresentationPreview, ProposePresentation, ProposePresentationContent, ProposePresentationDecorators,
};
use messages::msg_fields::protocols::present_proof::request::RequestPresentation;
use messages::msg_fields::protocols::present_proof::PresentProof;
use messages::msg_fields::protocols::report_problem::ProblemReport;
use messages::msg_parts::MsgParts;
use messages::AriesMessage;
use uuid::Uuid;

use crate::errors::error::prelude::*;
use crate::handlers::util::{get_attach_as_string, PresentationProposalData};
use crate::protocols::common::build_problem_report_msg;
use crate::protocols::proof_presentation::prover::state_machine::{ProverSM, ProverState};
use crate::protocols::SendClosure;

use super::types::{RetrievedCredentials, SelectedCredentials};

#[derive(Serialize, Deserialize, Debug, Clone, Default)]
pub struct Prover {
    prover_sm: ProverSM,
}

impl Prover {
    pub fn create(source_id: &str) -> VcxResult<Prover> {
        trace!("Prover::create >>> source_id: {}", source_id);
        Ok(Prover {
            prover_sm: ProverSM::new(source_id.to_string()),
        })
    }

    pub fn create_from_request(source_id: &str, presentation_request: RequestPresentation) -> VcxResult<Prover> {
        trace!(
            "Prover::create_from_request >>> source_id: {}, presentation_request: {:?}",
            source_id,
            presentation_request
        );
        Ok(Prover {
            prover_sm: ProverSM::from_request(presentation_request, source_id.to_string()),
        })
    }

    pub fn get_state(&self) -> ProverState {
        self.prover_sm.get_state()
    }

    pub fn presentation_status(&self) -> u32 {
        self.prover_sm.get_presentation_status()
    }

    pub async fn retrieve_credentials(&self, anoncreds: &Arc<dyn BaseAnonCreds>) -> VcxResult<RetrievedCredentials> {
        trace!("Prover::retrieve_credentials >>>");
        let presentation_request = self.presentation_request_data()?;
        let json_retrieved_credentials = anoncreds
            .prover_get_credentials_for_proof_req(&presentation_request)
            .await?;
        trace!("Prover::retrieve_credentials >>> presentation_request: {presentation_request}, json_retrieved_credentials: {json_retrieved_credentials}");
        Ok(serde_json::from_str(&json_retrieved_credentials)?)
    }

    pub async fn generate_presentation(
        &mut self,
        ledger: &Arc<dyn AnoncredsLedgerRead>,
        anoncreds: &Arc<dyn BaseAnonCreds>,
        credentials: SelectedCredentials,
        self_attested_attrs: HashMap<String, String>,
    ) -> VcxResult<()> {
        trace!(
            "Prover::generate_presentation >>> credentials: {:?}, self_attested_attrs: {:?}",
            credentials,
            self_attested_attrs
        );
        self.prover_sm = self
            .prover_sm
            .clone()
            .generate_presentation(ledger, anoncreds, credentials, self_attested_attrs)
            .await?;
        Ok(())
    }

    pub fn get_presentation_msg(&self) -> VcxResult<Presentation> {
        Ok(self.prover_sm.get_presentation_msg()?.to_owned())
    }

    pub async fn build_proposal(&mut self, proposal_data: PresentationProposalData) -> VcxResult<ProposePresentation> {
        trace!("Prover::build_proposal >>>");
        self.prover_sm = self
            .prover_sm
            .clone()
            .build_presentation_proposal(proposal_data)
            .await?;
        self.prover_sm.get_proposal()
    }

    pub async fn send_presentation(&mut self, send_message: SendClosure) -> VcxResult<()> {
        trace!("Prover::send_presentation >>>");
        self.prover_sm = self.prover_sm.clone().set_presentation().await?;
        match self.prover_sm.get_state() {
            ProverState::PresentationSet => {
                let presentation = self.prover_sm.get_presentation_msg()?;
                send_message(presentation.clone().into()).await?;
                Ok(())
            }
            ProverState::PresentationPreparationFailed => {
                let problem_report = self.prover_sm.get_problem_report()?;
                send_message(problem_report.into()).await?;
                Ok(())
            }
            _ => Err(AriesVcxError::from_msg(
                AriesVcxErrorKind::NotReady,
                "Cannot send presentation",
            )),
        }
    }

    pub fn process_presentation_ack(&mut self, ack: AckPresentation) -> VcxResult<()> {
        trace!("Prover::process_presentation_ack >>>");
        self.prover_sm = self.prover_sm.clone().receive_presentation_ack(ack)?;
        Ok(())
    }

    pub fn progressable_by_message(&self) -> bool {
        self.prover_sm.progressable_by_message()
    }

    pub fn presentation_request_data(&self) -> VcxResult<String> {
        Ok(get_attach_as_string!(
            &self
                .prover_sm
                .get_presentation_request()?
                .content
                .request_presentations_attach
        ))
    }

    pub fn get_proof_request_attachment(&self) -> VcxResult<String> {
        let data = get_attach_as_string!(
            &self
                .prover_sm
                .get_presentation_request()?
                .content
                .request_presentations_attach
        );

        let proof_request_data: serde_json::Value = serde_json::from_str(&data).map_err(|err| {
            AriesVcxError::from_msg(
                AriesVcxErrorKind::InvalidJson,
                format!("Cannot deserialize {:?} into PresentationRequestData: {:?}", data, err),
            )
        })?;
        Ok(proof_request_data.to_string())
    }

    pub fn get_source_id(&self) -> String {
        self.prover_sm.source_id()
    }

    pub fn get_thread_id(&self) -> VcxResult<String> {
        self.prover_sm.get_thread_id()
    }

    pub async fn process_aries_msg(&mut self, message: AriesMessage) -> VcxResult<()> {
        let prover_sm = match message {
            AriesMessage::PresentProof(PresentProof::RequestPresentation(request)) => {
                self.prover_sm.clone().receive_presentation_request(request)?
            }
            AriesMessage::PresentProof(PresentProof::Ack(ack)) => {
                self.prover_sm.clone().receive_presentation_ack(ack)?
            }
            AriesMessage::ReportProblem(report) => self.prover_sm.clone().receive_presentation_reject(report)?,
            AriesMessage::Notification(Notification::ProblemReport(report)) => {
                let MsgParts {
                    id,
                    content,
                    decorators,
                } = report;
                let report = ProblemReport::with_decorators(id, content.0, decorators);
                self.prover_sm.clone().receive_presentation_reject(report)?
            }
            AriesMessage::PresentProof(PresentProof::ProblemReport(report)) => {
                let MsgParts {
                    id,
                    content,
                    decorators,
                } = report;
                let report = ProblemReport::with_decorators(id, content.0, decorators);
                self.prover_sm.clone().receive_presentation_reject(report)?
            }
            _ => self.prover_sm.clone(),
        };
        self.prover_sm = prover_sm;
        Ok(())
    }

    // TODO: Can we delete this (please)?
    pub async fn decline_presentation_request(
        &mut self,
        send_message: SendClosure,
        reason: Option<String>,
        proposal: Option<String>,
    ) -> VcxResult<()> {
        trace!(
            "Prover::decline_presentation_request >>> reason: {:?}, proposal: {:?}",
            reason,
            proposal
        );
        self.prover_sm = match (reason, proposal) {
            (Some(reason), None) => {
                let thread_id = self.prover_sm.get_thread_id()?;
                let problem_report = build_problem_report_msg(Some(reason), &thread_id);
                send_message(problem_report.clone().into()).await?;
                self.prover_sm
                    .clone()
                    .decline_presentation_request(problem_report)
                    .await?
            }
            (None, Some(proposal)) => {
                let presentation_preview: PresentationPreview = serde_json::from_str(&proposal).map_err(|err| {
                    AriesVcxError::from_msg(
                        AriesVcxErrorKind::InvalidJson,
                        format!("Cannot serialize Presentation Preview: {:?}", err),
                    )
                })?;
                let thread_id = self.prover_sm.get_thread_id()?;
                let id = Uuid::new_v4().to_string();
                let content = ProposePresentationContent::new(presentation_preview);
                let mut decorators = ProposePresentationDecorators::default();
                let thread = Thread::new(thread_id.to_owned());
                let mut timing = Timing::default();
                timing.out_time = Some(Utc::now());
                decorators.thread = Some(thread);
                decorators.timing = Some(timing);

                let proposal = ProposePresentation::with_decorators(id, content, decorators);
                send_message(proposal.into()).await?;
                self.prover_sm.clone().negotiate_presentation().await?
            }
            (None, None) => {
                return Err(AriesVcxError::from_msg(
                    AriesVcxErrorKind::InvalidOption,
                    "Either `reason` or `proposal` parameter must be specified.",
                ));
            }
            (Some(_), Some(_)) => {
                return Err(AriesVcxError::from_msg(
                    AriesVcxErrorKind::InvalidOption,
                    "Only one of `reason` or `proposal` parameters must be specified.",
                ));
            }
        };
        Ok(())
    }
}
