use aries::messages::issuance::credential::Credential;
use aries::messages::status::Status;

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct FinishedHolderState {
    pub cred_id: Option<String>,
    pub credential: Option<Credential>,
    pub status: Status,
    pub rev_reg_def_json: Option<String>,
}
