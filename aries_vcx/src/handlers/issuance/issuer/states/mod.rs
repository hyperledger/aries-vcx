pub(super) mod credential_sent;
pub(super) mod finished;
pub(super) mod initial;
pub(super) mod offer_sent;
pub(super) mod requested_received;
pub(super) mod proposal_received;

#[derive(Serialize, Deserialize, Debug, Clone, Default)]
pub struct OfferInfo {
    pub credential_json: String,
    pub cred_def_id: String
}
