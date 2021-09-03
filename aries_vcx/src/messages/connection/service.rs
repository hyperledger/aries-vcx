use crate::messages::connection::did_doc::Did;
use crate::libindy::utils::ledger;
use crate::error::prelude::*;

pub const SERVICE_SUFFIX: &str = "indy";
pub const SERVICE_TYPE: &str = "IndyAgent";

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(untagged)]
pub enum Service {
    FullService(FullService),
    Did(Did)
}

impl Service {
    pub fn resolve(&self) -> VcxResult<FullService> {
        match self {
            Service::FullService(full_service) => Ok(full_service.clone()),
            Service::Did(did) => ledger::get_service(&did)
        }
    }
}

#[derive(Debug, Deserialize, Serialize, Clone, PartialEq)]
pub struct FullService {
    pub id: String,
    #[serde(rename = "type")]
    pub type_: String,
    #[serde(default)]
    pub priority: u32,
    #[serde(default)]
    #[serde(rename = "recipientKeys")]
    pub recipient_keys: Vec<String>,
    #[serde(default)]
    #[serde(rename = "routingKeys")]
    pub routing_keys: Vec<String>,
    #[serde(rename = "serviceEndpoint")]
    pub service_endpoint: String,
}

impl Default for FullService {
    fn default() -> FullService {
        FullService {
            // TODO: FIXME Several services????
            id: format!("did:example:123456789abcdefghi;{}", SERVICE_SUFFIX),
            type_: String::from(SERVICE_TYPE),
            priority: 0,
            service_endpoint: String::new(),
            recipient_keys: Vec::new(),
            routing_keys: Vec::new(),
        }
    }
}
