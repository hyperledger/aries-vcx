#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LegacyAgentInfo {
    pub pw_did: String,
    pub pw_vk: String,
    pub agent_did: String,
    pub agent_vk: String,
}
