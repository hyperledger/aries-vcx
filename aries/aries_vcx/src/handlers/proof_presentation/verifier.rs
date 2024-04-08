use anoncreds_types::data_types::messages::pres_request::PresentationRequest;
use aries_vcx_anoncreds::anoncreds::base_anoncreds::BaseAnonCreds;
use aries_vcx_ledger::ledger::base_ledger::AnoncredsLedgerRead;
use messages::{
    msg_fields::protocols::{
        notification::Notification,
        present_proof::{
            v1::{
                present::PresentationV1, propose::ProposePresentationV1,
                request::RequestPresentationV1, PresentProofV1,
            },
            PresentProof,
        },
        report_problem::ProblemReport,
    },
    AriesMessage,
};

use crate::{
    errors::error::prelude::*,
    handlers::util::get_attach_as_string,
    protocols::{
        common::build_problem_report_msg,
        proof_presentation::verifier::{
            state_machine::{VerifierSM, VerifierState},
            verification_status::PresentationVerificationStatus,
        },
    },
};

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Default)]
pub struct Verifier {
    verifier_sm: VerifierSM,
}

impl Verifier {
    pub fn create(source_id: &str) -> VcxResult<Self> {
        trace!("Verifier::create >>> source_id: {:?}", source_id);

        Ok(Self {
            verifier_sm: VerifierSM::new(source_id),
        })
    }

    pub fn create_from_request(
        source_id: String,
        presentation_request: &PresentationRequest,
    ) -> VcxResult<Self> {
        trace!(
            "Verifier::create_from_request >>> source_id: {:?}, presentation_request: {:?}",
            source_id,
            presentation_request
        );
        let verifier_sm = VerifierSM::from_request(&source_id, presentation_request)?;
        Ok(Self { verifier_sm })
    }

    pub fn create_from_proposal(
        source_id: &str,
        presentation_proposal: &ProposePresentationV1,
    ) -> VcxResult<Self> {
        trace!(
            "Issuer::create_from_proposal >>> source_id: {:?}, presentation_proposal: {:?}",
            source_id,
            presentation_proposal
        );
        Ok(Self {
            verifier_sm: VerifierSM::from_proposal(source_id, presentation_proposal),
        })
    }

    pub fn get_source_id(&self) -> String {
        self.verifier_sm.source_id()
    }

    pub fn get_state(&self) -> VerifierState {
        self.verifier_sm.get_state()
    }

    // TODO: Find a better name for this method
    pub fn mark_presentation_request_sent(&mut self) -> VcxResult<RequestPresentationV1> {
        if self.verifier_sm.get_state() == VerifierState::PresentationRequestSet {
            let request = self.verifier_sm.presentation_request_msg()?;
            self.verifier_sm = self.verifier_sm.clone().mark_presentation_request_sent()?;
            Ok(request)
        } else {
            Err(AriesVcxError::from_msg(
                AriesVcxErrorKind::NotReady,
                "Cannot send presentation request",
            ))
        }
    }

    // todo: verification and sending ack should be separate apis
    pub async fn verify_presentation(
        &mut self,
        ledger: &impl AnoncredsLedgerRead,
        anoncreds: &impl BaseAnonCreds,
        presentation: PresentationV1,
    ) -> VcxResult<AriesMessage> {
        trace!("Verifier::verify_presentation >>>");
        self.verifier_sm = self
            .verifier_sm
            .clone()
            .verify_presentation(ledger, anoncreds, presentation)
            .await?;
        self.verifier_sm.get_final_message()
    }

    pub fn set_presentation_request(
        &mut self,
        presentation_request_data: PresentationRequest,
        comment: Option<String>,
    ) -> VcxResult<()> {
        trace!(
            "Verifier::set_presentation_request >>> presentation_request_data: {:?}, comment: \
             ${:?}",
            presentation_request_data,
            comment
        );
        self.verifier_sm = self
            .verifier_sm
            .clone()
            .set_presentation_request(&presentation_request_data, comment)?;
        Ok(())
    }

    pub fn get_presentation_request_msg(&self) -> VcxResult<RequestPresentationV1> {
        self.verifier_sm.presentation_request_msg()
    }

    pub fn get_presentation_request_attachment(&self) -> VcxResult<String> {
        let pres_req = &self.verifier_sm.presentation_request_msg()?;
        Ok(get_attach_as_string!(
            pres_req.content.request_presentations_attach
        ))
    }

    pub fn get_presentation_msg(&self) -> VcxResult<PresentationV1> {
        self.verifier_sm.get_presentation_msg()
    }

    pub fn get_verification_status(&self) -> PresentationVerificationStatus {
        self.verifier_sm.get_verification_status()
    }

    pub fn get_presentation_attachment(&self) -> VcxResult<String> {
        let presentation = &self.verifier_sm.get_presentation_msg()?;
        Ok(get_attach_as_string!(
            presentation.content.presentations_attach
        ))
    }

    pub fn get_presentation_proposal(&self) -> VcxResult<ProposePresentationV1> {
        self.verifier_sm.presentation_proposal()
    }

    pub fn get_thread_id(&self) -> VcxResult<String> {
        Ok(self.verifier_sm.thread_id())
    }

    pub async fn process_aries_msg(
        &mut self,
        ledger: &impl AnoncredsLedgerRead,
        anoncreds: &impl BaseAnonCreds,
        message: AriesMessage,
    ) -> VcxResult<Option<AriesMessage>> {
        let (verifier_sm, message) = match message {
            AriesMessage::PresentProof(PresentProof::V1(PresentProofV1::ProposePresentation(
                proposal,
            ))) => (
                self.verifier_sm
                    .clone()
                    .receive_presentation_proposal(proposal)?,
                None,
            ),
            AriesMessage::PresentProof(PresentProof::V1(PresentProofV1::Presentation(
                presentation,
            ))) => {
                let sm = self
                    .verifier_sm
                    .clone()
                    .verify_presentation(ledger, anoncreds, presentation)
                    .await?;
                (sm.clone(), Some(sm.get_final_message()?))
            }
            AriesMessage::ReportProblem(report) => (
                self.verifier_sm
                    .clone()
                    .receive_presentation_request_reject(report)?,
                None,
            ),
            AriesMessage::Notification(Notification::ProblemReport(report)) => (
                self.verifier_sm
                    .clone()
                    .receive_presentation_request_reject(report.into())?,
                None,
            ),
            AriesMessage::PresentProof(PresentProof::V1(PresentProofV1::ProblemReport(report))) => {
                (
                    self.verifier_sm
                        .clone()
                        .receive_presentation_request_reject(report.into())?,
                    None,
                )
            }
            _ => (self.verifier_sm.clone(), None),
        };
        self.verifier_sm = verifier_sm;
        Ok(message)
    }

    pub fn progressable_by_message(&self) -> bool {
        self.verifier_sm.progressable_by_message()
    }

    pub async fn decline_presentation_proposal<'a>(
        &'a mut self,
        reason: &'a str,
    ) -> VcxResult<ProblemReport> {
        trace!(
            "Verifier::decline_presentation_proposal >>> reason: {:?}",
            reason
        );
        let state = self.verifier_sm.get_state();
        if state == VerifierState::PresentationProposalReceived {
            let proposal = self.verifier_sm.presentation_proposal()?;
            let thread_id = match proposal.decorators.thread {
                Some(thread) => thread.thid,
                None => proposal.id,
            };
            let problem_report = build_problem_report_msg(Some(reason.to_string()), &thread_id);
            self.verifier_sm = self
                .verifier_sm
                .clone()
                .reject_presentation_proposal(problem_report.clone())
                .await?;
            Ok(problem_report)
        } else {
            Err(AriesVcxError::from_msg(
                AriesVcxErrorKind::NotReady,
                format!(
                    "Unable to reject presentation proposal in state {:?}",
                    state
                ),
            ))
        }
    }
}
