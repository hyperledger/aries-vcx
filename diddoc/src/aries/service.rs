pub const SERVICE_SUFFIX: &str = "indy";

pub const SERVICE_TYPE: &str = "IndyAgent";

// Service object as defined https://github.com/hyperledger/aries-rfcs/blob/main/features/0434-outofband/README.md#the-services-item
// Note that is divergence from w3c spec https://w3c.github.io/did-core/#service-properties
#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct AriesService {
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

impl AriesService {
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

impl Default for AriesService {
    fn default() -> AriesService {
        AriesService {
            id: format!("did:example:123456789abcdefghi;{}", SERVICE_SUFFIX),
            type_: String::from(SERVICE_TYPE),
            priority: 0,
            service_endpoint: String::new(),
            recipient_keys: Vec::new(),
            routing_keys: Vec::new(),
        }
    }
}

impl PartialEq for AriesService {
    fn eq(&self, other: &Self) -> bool {
        self.recipient_keys == other.recipient_keys && self.routing_keys == other.routing_keys
    }
}

#[cfg(test)]
#[cfg(feature = "general_test")]
mod unit_tests {
    use crate::aries::{
        diddoc::test_utils::{_recipient_keys, _routing_keys, _routing_keys_1, _service_endpoint},
        service::AriesService,
    };

    #[test]
    fn test_service_comparison() {
        let service1 = AriesService::create()
            .set_service_endpoint(_service_endpoint())
            .set_recipient_keys(_recipient_keys())
            .set_routing_keys(_routing_keys());

        let service2 = AriesService::create()
            .set_service_endpoint(_service_endpoint())
            .set_recipient_keys(_recipient_keys())
            .set_routing_keys(_routing_keys());

        let service3 = AriesService::create()
            .set_service_endpoint("bogus_endpoint".to_string())
            .set_recipient_keys(_recipient_keys())
            .set_routing_keys(_routing_keys());

        let service4 = AriesService::create()
            .set_service_endpoint(_service_endpoint())
            .set_recipient_keys(_recipient_keys())
            .set_routing_keys(_routing_keys_1());

        assert_eq!(service1, service2);
        assert_eq!(service1, service3);
        assert_ne!(service1, service4);
    }
}
