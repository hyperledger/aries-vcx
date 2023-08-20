use crate::handlers::util::Status;
use crate::protocols::issuance::issuer::state_machine::RevocationInfoV1;
use crate::protocols::issuance::issuer::states::finished::FinishedState;
use messages::msg_fields::protocols::cred_issuance::issue_credential::IssueCredential;

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct CredentialSetState {
    pub revocation_info_v1: Option<RevocationInfoV1>,
    pub msg_issue_credential: IssueCredential,
}

impl From<CredentialSetState> for FinishedState {
    fn from(state: CredentialSetState) -> Self {
        trace!("SM is now in Finished state");
        FinishedState {
            cred_id: None,
            revocation_info_v1: state.revocation_info_v1,
            status: Status::Success,
        }
    }
}
