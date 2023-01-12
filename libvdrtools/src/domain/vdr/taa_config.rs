#[derive(Debug, Serialize, Deserialize, Eq, PartialEq, Clone)]
pub struct TAAConfig {
    pub text: Option<String>,
    pub version: Option<String>,
    pub taa_digest: Option<String>,
    pub acc_mech_type: String,
    pub time: u64,
}
