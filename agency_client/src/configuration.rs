#[derive(Serialize, Deserialize, Default, Clone)]
pub struct AgentProvisionConfig {
    pub agency_did: String,
    pub agency_verkey: String,
    pub agency_endpoint: String,
    pub agent_seed: Option<String>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct AgencyClientConfig {
    pub agency_did: String,
    pub agency_endpoint: String,
    pub agency_verkey: String,
    pub remote_to_sdk_did: String,
    pub remote_to_sdk_verkey: String,
    pub sdk_to_remote_did: String,
    pub sdk_to_remote_verkey: String,
}
