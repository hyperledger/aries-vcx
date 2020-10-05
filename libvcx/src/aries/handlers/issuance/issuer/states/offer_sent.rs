use aries::handlers::issuance::issuer::state_machine::RevocationInfoV1;
use aries::handlers::issuance::issuer::states::finished::FinishedState;
use aries::handlers::issuance::issuer::states::requested_received::RequestReceivedState;
use aries::messages::error::ProblemReport;
use aries::messages::issuance::credential_request::CredentialRequest;
use aries::messages::status::Status;

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct OfferSentState {
    pub offer: String,
    pub cred_data: String,
    pub rev_reg_id: Option<String>,
    pub tails_file: Option<String>,
    pub connection_handle: u32,
    pub thread_id: String,
}

impl From<OfferSentState> for FinishedState {
    fn from(state: OfferSentState) -> Self {
        trace!("SM is now in Finished state");
        FinishedState {
            cred_id: None,
            thread_id: state.thread_id,
            revocation_info_v1: Some(RevocationInfoV1 {
                cred_rev_id: None,
                rev_reg_id: state.rev_reg_id,
                tails_file: state.tails_file,
            }),
            status: Status::Undefined,
        }
    }
}

impl From<(OfferSentState, CredentialRequest)> for RequestReceivedState {
    fn from((state, request): (OfferSentState, CredentialRequest)) -> Self {
        trace!("SM is now in Request Received state");
        RequestReceivedState {
            offer: state.offer,
            cred_data: state.cred_data,
            rev_reg_id: state.rev_reg_id,
            tails_file: state.tails_file,
            connection_handle: state.connection_handle,
            request,
            thread_id: state.thread_id,
        }
    }
}

impl From<(OfferSentState, ProblemReport)> for FinishedState {
    fn from((state, err): (OfferSentState, ProblemReport)) -> Self {
        trace!("SM is now in Finished state");
        FinishedState {
            cred_id: None,
            thread_id: state.thread_id,
            revocation_info_v1: Some(RevocationInfoV1 {
                cred_rev_id: None,
                rev_reg_id: state.rev_reg_id,
                tails_file: state.tails_file,
            }),
            status: Status::Failed(err),
        }
    }
}