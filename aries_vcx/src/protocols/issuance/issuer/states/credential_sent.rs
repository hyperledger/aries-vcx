use messages::status::Status;

use crate::protocols::issuance::issuer::{state_machine::RevocationInfoV1, states::finished::FinishedState};

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct CredentialSentState {
    pub revocation_info_v1: Option<RevocationInfoV1>,
}

impl From<CredentialSentState> for FinishedState {
    fn from(state: CredentialSentState) -> Self {
        trace!("SM is now in Finished state");
        FinishedState {
            cred_id: None,
            revocation_info_v1: state.revocation_info_v1,
            status: Status::Success,
        }
    }
}
