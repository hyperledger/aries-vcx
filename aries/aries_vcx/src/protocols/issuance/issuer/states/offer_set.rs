use anoncreds_types::data_types::identifiers::cred_def_id::CredentialDefinitionId;
use messages::msg_fields::protocols::{
    cred_issuance::v1::{
        offer_credential::OfferCredentialV1, request_credential::RequestCredentialV1,
    },
    report_problem::ProblemReport,
};

use crate::{
    handlers::util::Status,
    protocols::issuance::issuer::{
        state_machine::RevocationInfoV1,
        states::{finished::FinishedState, requested_received::RequestReceivedState},
    },
};

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct OfferSetState {
    pub offer: OfferCredentialV1,
    pub credential_json: String,
    pub cred_def_id: CredentialDefinitionId,
    pub rev_reg_id: Option<String>,
    pub tails_file: Option<String>,
}

impl OfferSetState {
    pub fn new(
        cred_offer_msg: OfferCredentialV1,
        credential_json: &str,
        cred_def_id: CredentialDefinitionId,
        rev_reg_id: Option<String>,
        tails_file: Option<String>,
    ) -> Self {
        OfferSetState {
            offer: cred_offer_msg,
            credential_json: credential_json.into(),
            cred_def_id,
            rev_reg_id,
            tails_file,
        }
    }
}

impl RequestReceivedState {
    pub fn from_offer_set_and_request(state: OfferSetState, request: RequestCredentialV1) -> Self {
        trace!("SM is now in Request Received state");
        RequestReceivedState {
            offer: state.offer,
            cred_data: state.credential_json,
            rev_reg_id: state.rev_reg_id,
            tails_file: state.tails_file,
            request,
        }
    }
}

impl FinishedState {
    pub fn from_offer_set_and_error(state: OfferSetState, err: ProblemReport) -> Self {
        trace!("SM is now in Finished state");
        FinishedState {
            cred_id: None,
            revocation_info_v1: Some(RevocationInfoV1 {
                cred_rev_id: None,
                rev_reg_id: state.rev_reg_id,
                tails_file: state.tails_file,
            }),
            status: Status::Failed(err),
        }
    }
}
