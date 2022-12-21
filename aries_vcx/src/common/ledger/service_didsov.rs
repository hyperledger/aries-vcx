pub const SERVICE_SUFFIX: &str = "indy";

pub const SERVICE_TYPE: &str = "IndyAgent";

// Service object as defined https://github.com/hyperledger/aries-rfcs/blob/main/features/0434-outofband/README.md#the-services-item
// Note that is divergence from w3c spec https://w3c.github.io/did-core/#service-properties
#[derive(Debug, Deserialize, Serialize, Clone,PartialEq)]
pub struct EndpointDidSov {
    pub endpoint: String,
    #[serde(default)]
    #[serde(rename = "routingKeys")]
    pub routing_keys: Option<Vec<String>>,
}

impl EndpointDidSov {
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
}

impl Default for EndpointDidSov {
    fn default() -> EndpointDidSov {
        EndpointDidSov {
            endpoint: String::new(),
            routing_keys: Some(Vec::new()),
        }
    }
}

#[cfg(test)]
#[cfg(feature = "general_test")]
mod unit_tests {
    use messages::did_doc::aries::diddoc::test_utils::{_routing_keys, _service_endpoint};
    use crate::common::ledger::service_didsov::EndpointDidSov;

    #[test]
    fn test_service_comparison() {
        let service1 = EndpointDidSov::create()
            .set_service_endpoint(_service_endpoint())
            .set_routing_keys(Some(_routing_keys()));

        let service2 = EndpointDidSov::create()
            .set_service_endpoint(_service_endpoint())
            .set_routing_keys(Some(_routing_keys()));

        let service3 = EndpointDidSov::create()
            .set_service_endpoint("bogus_endpoint".to_string())
            .set_routing_keys(Some(_routing_keys()));

        let service4 = EndpointDidSov::create()
            .set_service_endpoint(_service_endpoint())
            .set_routing_keys(Some(_routing_keys()));

        assert_eq!(service1, service2);
        assert_eq!(service1, service4);
        assert_ne!(service1, service3);
    }
}
