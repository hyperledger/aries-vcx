pub const SERVICE_SUFFIX: &str = "indy";
pub const SERVICE_TYPE: &str = "IndyAgent";

// Service object as defined https://github.com/hyperledger/aries-rfcs/blob/main/features/0434-outofband/README.md#the-services-item
// Note that is divergence from w3c spec https://w3c.github.io/did-core/#service-properties
#[derive(Serialize, Deserialize)]
pub struct EndpointService {
    pub endpoint: String,
    #[serde(rename = "recipientKeys")]
    pub recipient_keys: Option<Vec<String>>,
    #[serde(default)]
    #[serde(rename = "routingKeys")]
    pub routing_keys: Option<Vec<String>>,
}

impl EndpointService {
    pub fn create() -> Self {
        Self::default()
    }

    pub fn set_service_endpoint(mut self, service_endpoint: String) -> Self {
        self.endpoint = service_endpoint;
        self
    }

    pub fn set_routing_keys(mut self, routing_keys: Option<Vec<String>>) -> Self {
        self.routing_keys = routing_keys;
        self
    }

    pub fn set_recipient_keys(mut self, recipient_keys: Option<Vec<String>>) -> Self {
        self.recipient_keys = recipient_keys;
        self
    }
}

impl Default for EndpointService {
    fn default() -> EndpointService {
        EndpointService {
            endpoint: String::new(),
            recipient_keys: Some(Vec::new()),
            routing_keys: Some(Vec::new()),
        }
    }
}

impl PartialEq for EndpointService {
    fn eq(&self, other: &Self) -> bool {
        self.recipient_keys == other.recipient_keys && self.routing_keys == other.routing_keys
    }
}

#[cfg(test)]
#[cfg(feature = "general_test")]
mod unit_tests {
    use crate::did_doc::service_aries_public::EndpointService;
    use crate::did_doc::test_utils::{_recipient_keys, _routing_keys, _routing_keys_1, _service_endpoint};

    #[test]
    fn test_service_comparison() {
        let service1 = EndpointService::create()
            .set_service_endpoint(_service_endpoint())
            .set_recipient_keys(Some(_recipient_keys()))
            .set_routing_keys(Some(_routing_keys()));

        let service2 = EndpointService::create()
            .set_service_endpoint(_service_endpoint())
            .set_recipient_keys(Some(_recipient_keys()))
            .set_routing_keys(Some(_routing_keys()));

        let service3 = EndpointService::create()
            .set_service_endpoint("bogus_endpoint".to_string())
            .set_recipient_keys(Some(_recipient_keys()))
            .set_routing_keys(Some(_routing_keys()));

        let service4 = EndpointService::create()
            .set_service_endpoint(_service_endpoint())
            .set_recipient_keys(Some(_recipient_keys()))
            .set_routing_keys(Some(_routing_keys()));

        assert_eq!(service1, service2);
        assert_eq!(service1, service3);
        assert_ne!(service1, service4);
    }
}
