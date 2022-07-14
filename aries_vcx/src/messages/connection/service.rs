use crate::error::prelude::*;
use crate::libindy::utils::ledger;
use crate::messages::connection::did::Did;

pub const SERVICE_SUFFIX: &str = "indy";
pub const SERVICE_TYPE: &str = "IndyAgent";

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(untagged)]
pub enum ServiceResolvable {
    FullService(FullService),
    Did(Did),
}

impl ServiceResolvable {
    pub async fn resolve(&self) -> VcxResult<FullService> {
        match self {
            ServiceResolvable::FullService(full_service) => Ok(full_service.clone()),
            ServiceResolvable::Did(did) => ledger::get_service(&did).await
        }
    }
}

#[derive(Debug, Deserialize, Serialize, Clone)]
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

impl FullService {
    pub fn create() -> Self {
        Self::default()
    }

    pub fn set_service_endpoint(mut self, service_endpoint: String) -> Self {
        self.service_endpoint = service_endpoint;
        self
    }

    pub fn set_routing_keys(mut self, routing_keys: Vec<String>) -> Self {
        self.routing_keys = routing_keys;
        self
    }

    pub fn set_recipient_keys(mut self, recipient_keys: Vec<String>) -> Self {
        self.recipient_keys = recipient_keys;
        self
    }
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

impl PartialEq for FullService {
    fn eq(&self, other: &Self) -> bool {
        self.recipient_keys == other.recipient_keys &&
            self.routing_keys == other.routing_keys
    }
}

#[cfg(test)]
pub mod tests {
    use crate::messages::connection::did_doc::test_utils::{_recipient_keys, _routing_keys, _routing_keys_1, _service_endpoint};

    use super::*;

    #[test]
    #[cfg(feature = "general_test")]
    fn test_service_comparison() {
        let service1 = FullService::create()
            .set_service_endpoint(_service_endpoint())
            .set_recipient_keys(_recipient_keys())
            .set_routing_keys(_routing_keys());

        let service2 = FullService::create()
            .set_service_endpoint(_service_endpoint())
            .set_recipient_keys(_recipient_keys())
            .set_routing_keys(_routing_keys());

        let service3 = FullService::create()
            .set_service_endpoint("bogus_endpoint".to_string())
            .set_recipient_keys(_recipient_keys())
            .set_routing_keys(_routing_keys());

        let service4 = FullService::create()
            .set_service_endpoint(_service_endpoint())
            .set_recipient_keys(_recipient_keys())
            .set_routing_keys(_routing_keys_1());

        assert!(service1 == service2);
        assert!(service1 == service3);
        assert!(service1 != service4);
    }
}
