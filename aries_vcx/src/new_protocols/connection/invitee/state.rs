use diddoc_legacy::aries::diddoc::AriesDidDoc;
use url::Url;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BootstrapInfo {
    pub service_endpoint: Url,
    pub recipient_keys: Vec<String>,
    #[serde(default)]
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub routing_keys: Vec<String>,
    pub did: Option<String>,
    pub service_endpoint_did: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InviteeRequested {
    pub(crate) bootstrap_info: BootstrapInfo,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InviteeComplete {
    pub(crate) bootstrap_info: BootstrapInfo,
    pub(crate) did_doc: AriesDidDoc,
}
