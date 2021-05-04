use crate::aries::handlers::issuance::issuer::state_machine::RevocationInfoV1;
use crate::aries::messages::status::Status;

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct FinishedState {
    pub cred_id: Option<String>,
    pub thread_id: String,
    pub revocation_info_v1: Option<RevocationInfoV1>,
    pub status: Status,
}
