pub(super) mod initial;
pub(super) mod proposal_received;
pub(super) mod offer_set;
pub(super) mod offer_sent;
pub(super) mod requested_received;
pub(super) mod credential_sent;
pub(super) mod finished;

#[derive(Serialize, Deserialize, Debug, Clone, Default)]
pub struct OfferInfo {
    pub credential_json: String,
    pub cred_def_id: String,
    pub rev_reg_id: Option<String>,
    pub tails_file: Option<String>
}

impl OfferInfo {
    pub fn new(credential_json: String, cred_def_id: String, rev_reg_id: Option<String>, tails_file: Option<String>) -> Self {
        Self {
            credential_json,
            cred_def_id,
            rev_reg_id,
            tails_file
        }
    }
}
