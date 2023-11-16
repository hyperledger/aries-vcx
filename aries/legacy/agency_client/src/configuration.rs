use url::Url;

#[derive(Serialize, Deserialize, Clone)]
pub struct AgentProvisionConfig {
    pub agency_did: String,
    pub agency_verkey: String,
    pub agency_endpoint: Url,
    pub agent_seed: Option<String>,
}

impl Default for AgentProvisionConfig {
    fn default() -> Self {
        Self {
            agency_did: Default::default(),
            agency_verkey: Default::default(),
            agency_endpoint: "http://127.0.0.1:8080"
                .parse()
                .expect("should be valid url"),
            agent_seed: Default::default(),
        }
    }
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct AgencyClientConfig {
    pub agency_did: String,
    pub agency_endpoint: Url,
    pub agency_verkey: String,
    pub remote_to_sdk_did: String,
    pub remote_to_sdk_verkey: String,
    pub sdk_to_remote_did: String,
    pub sdk_to_remote_verkey: String,
}

impl Default for AgencyClientConfig {
    fn default() -> Self {
        Self {
            agency_did: Default::default(),
            agency_endpoint: "http://127.0.0.1:8080"
                .parse()
                .expect("should be valid url"),
            agency_verkey: Default::default(),
            remote_to_sdk_did: Default::default(),
            remote_to_sdk_verkey: Default::default(),
            sdk_to_remote_did: Default::default(),
            sdk_to_remote_verkey: Default::default(),
        }
    }
}
