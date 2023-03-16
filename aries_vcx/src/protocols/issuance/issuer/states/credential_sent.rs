use crate::protocols::issuance::issuer::state_machine::RevocationInfoV1;
use crate::protocols::issuance::issuer::states::finished::FinishedState;
use messages::status::Status;

// todo: should Sent state be merged into Finished state - it only differs by whether "ack" was received or not - could be just flag
// todo:
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
