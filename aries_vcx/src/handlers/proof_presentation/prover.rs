use std::collections::HashMap;
use std::sync::Arc;

use aries_vcx_core::anoncreds::base_anoncreds::BaseAnonCreds;
use aries_vcx_core::ledger::base_ledger::AnoncredsLedgerRead;
use messages::msg_fields::protocols::notification::Notification;
use messages::msg_fields::protocols::present_proof::ack::AckPresentation;
use messages::msg_fields::protocols::present_proof::present::Presentation;
use messages::msg_fields::protocols::present_proof::propose::PresentationPreview;
use messages::msg_fields::protocols::present_proof::request::RequestPresentation;
use messages::msg_fields::protocols::present_proof::PresentProof;
use messages::msg_fields::protocols::report_problem::ProblemReport;
use messages::msg_parts::MsgParts;
use messages::AriesMessage;

use crate::errors::error::prelude::*;
use crate::handlers::util::{get_attach_as_string, PresentationProposalData};
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

    pub fn set_presentation(&mut self, presentation: Presentation) -> VcxResult<()> {
        trace!("Prover::set_presentation >>>");
        self.prover_sm = self.prover_sm.clone().set_presentation(presentation)?;
        Ok(())
    }

    pub async fn send_proposal(
        &mut self,
        proposal_data: PresentationProposalData,
        send_message: SendClosure,
    ) -> VcxResult<()> {
        trace!("Prover::send_proposal >>>");
        self.prover_sm = self
            .prover_sm
            .clone()
            .send_presentation_proposal(proposal_data, send_message)
            .await?;
        Ok(())
    }

    pub async fn send_presentation(&mut self, send_message: SendClosure) -> VcxResult<()> {
        trace!("Prover::send_presentation >>>");
        self.prover_sm = self.prover_sm.clone().send_presentation(send_message).await?;
        Ok(())
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
                self.prover_sm
                    .clone()
                    .decline_presentation_request(reason, send_message)
                    .await?
            }
            (None, Some(proposal)) => {
                let presentation_preview: PresentationPreview = serde_json::from_str(&proposal).map_err(|err| {
                    AriesVcxError::from_msg(
                        AriesVcxErrorKind::InvalidJson,
                        format!("Cannot serialize Presentation Preview: {:?}", err),
                    )
                })?;
                self.prover_sm
                    .clone()
                    .negotiate_presentation(presentation_preview, send_message)
                    .await?
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
