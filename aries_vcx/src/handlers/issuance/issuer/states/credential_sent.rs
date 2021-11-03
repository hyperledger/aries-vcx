use crate::handlers::issuance::issuer::state_machine::RevocationInfoV1;
use crate::handlers::issuance::issuer::states::finished::FinishedState;
use crate::messages::status::Status;

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
