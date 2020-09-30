use v3::handlers::issuance::issuer::state_machine::RevocationInfoV1;
use v3::handlers::issuance::issuer::states::finished::FinishedState;
use v3::messages::status::Status;

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct CredentialSentState {
    pub connection_handle: u32,
    pub revocation_info_v1: Option<RevocationInfoV1>,
    pub thread_id: String,
}


impl From<CredentialSentState> for FinishedState {
    fn from(state: CredentialSentState) -> Self {
        trace!("SM is now in Finished state");
        FinishedState {
            cred_id: None,
            thread_id: state.thread_id,
            revocation_info_v1: state.revocation_info_v1,
            status: Status::Success,
        }
    }
}