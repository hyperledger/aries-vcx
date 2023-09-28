use messages::msg_fields::protocols::cred_issuance::v1::issue_credential::IssueCredentialV1;

use crate::{
    handlers::util::Status,
    protocols::issuance::issuer::{
        state_machine::RevocationInfoV1, states::finished::FinishedState,
    },
};

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct CredentialSetState {
    pub revocation_info_v1: Option<RevocationInfoV1>,
    pub msg_issue_credential: IssueCredentialV1,
}

impl FinishedState {
    pub fn from_credential_set_state(state: CredentialSetState) -> Self {
        trace!("SM is now in Finished state");
        FinishedState {
            cred_id: None,
            revocation_info_v1: state.revocation_info_v1,
            status: Status::Success,
        }
    }
}
