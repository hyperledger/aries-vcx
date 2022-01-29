use crate::protocols::issuance::issuer::state_machine::RevocationInfoV1;
use crate::messages::status::Status;

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct FinishedState {
    pub cred_id: Option<String>,
    pub revocation_info_v1: Option<RevocationInfoV1>,
    pub status: Status,
}
