use std::collections::HashMap;

use anoncreds_types::data_types::messages::cred_selection::SelectedCredentials;
use aries_vcx_core::{
    anoncreds::base_anoncreds::BaseAnonCreds, ledger::base_ledger::AnoncredsLedgerRead,
    wallet::base_wallet::BaseWallet,
};
use messages::msg_fields::protocols::{
    present_proof::v1::{
        present::PresentationV1,
        request::{RequestPresentationV1, RequestPresentationV1Content},
    },
    report_problem::ProblemReport,
};
use uuid::Uuid;

use crate::{
    common::proofs::prover::generate_indy_proof,
    errors::error::prelude::*,
    handlers::util::{get_attach_as_string, Status},
    protocols::proof_presentation::prover::states::{
        finished::FinishedState,
        presentation_preparation_failed::PresentationPreparationFailedState,
        presentation_prepared::PresentationPreparedState,
    },
};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct PresentationRequestReceived {
    pub presentation_request: RequestPresentationV1,
}

impl Default for PresentationRequestReceived {
    fn default() -> Self {
        let id = Uuid::new_v4().to_string();
        let content = RequestPresentationV1Content::builder()
            .request_presentations_attach(Vec::new())
            .build();

        Self {
            presentation_request: RequestPresentationV1::builder()
                .id(id)
                .content(content)
                .build(),
        }
    }
}

impl PresentationRequestReceived {
    pub fn new(presentation_request: RequestPresentationV1) -> Self {
        Self {
            presentation_request,
        }
    }

    pub async fn build_presentation(
        &self,
        wallet: &impl BaseWallet,
        ledger: &impl AnoncredsLedgerRead,
        anoncreds: &impl BaseAnonCreds,
        credentials: &SelectedCredentials,
        self_attested_attrs: &HashMap<String, String>,
    ) -> VcxResult<String> {
        let proof_req_data_json = get_attach_as_string!(
            &self
                .presentation_request
                .content
                .request_presentations_attach
        );

        generate_indy_proof(
            wallet,
            ledger,
            anoncreds,
            credentials,
            self_attested_attrs,
            &proof_req_data_json,
        )
        .await
    }
}

impl From<(PresentationRequestReceived, ProblemReport)> for PresentationPreparationFailedState {
    fn from((state, problem_report): (PresentationRequestReceived, ProblemReport)) -> Self {
        trace!(
            "transit state from PresentationRequestReceived to PresentationPreparationFailedState"
        );
        PresentationPreparationFailedState {
            presentation_request: state.presentation_request,
            problem_report,
        }
    }
}

impl From<(PresentationRequestReceived, PresentationV1)> for PresentationPreparedState {
    fn from((state, presentation): (PresentationRequestReceived, PresentationV1)) -> Self {
        trace!("transit state from PresentationRequestReceived to PresentationPreparedState");
        PresentationPreparedState {
            presentation_request: state.presentation_request,
            presentation,
        }
    }
}

impl From<PresentationRequestReceived> for FinishedState {
    fn from(state: PresentationRequestReceived) -> Self {
        trace!("Prover: transit state from PresentationRequestReceived to FinishedState");
        FinishedState {
            presentation_request: Some(state.presentation_request),
            presentation: None,
            status: Status::Success,
        }
    }
}

impl From<(PresentationRequestReceived, ProblemReport)> for FinishedState {
    fn from((state, problem_report): (PresentationRequestReceived, ProblemReport)) -> Self {
        trace!("Prover: transit state from PresentationRequestReceived to FinishedState");
        FinishedState {
            presentation_request: Some(state.presentation_request),
            presentation: None,
            status: Status::Declined(problem_report),
        }
    }
}
