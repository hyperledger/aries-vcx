use did_doc::schema::service::Service;
use did_resolver_sov::resolution::ExtraFieldsSov;
use url::Url;

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
    pub service_endpoint: Url,
}

impl AriesService {
    pub fn create() -> Self {
        Self::default()
    }

    pub fn set_service_endpoint(mut self, service_endpoint: Url) -> Self {
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
            id: format!("did:example:123456789abcdefghi;{SERVICE_SUFFIX}"),
            type_: String::from(SERVICE_TYPE),
            priority: 0,
            service_endpoint: "https://dummy.dummy/dummy"
                .parse()
                .expect("dummy url should get parsed"),
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

impl From<Service<ExtraFieldsSov>> for AriesService {
    fn from(service: Service<ExtraFieldsSov>) -> Self {
        let mut aries_service = AriesService::default();
        aries_service.id = service.id().to_string();
        aries_service.type_ = service.service_type().to_string();
        aries_service.priority = 0;
        let aries_service = aries_service
            .set_service_endpoint(service.service_endpoint().to_owned().into())
            .set_routing_keys(service.extra().routing_keys().to_vec())
            .set_recipient_keys(service.extra().recipient_keys().to_vec());
        aries_service
    }
}

#[cfg(test)]
mod unit_tests {
    use crate::aries::diddoc::test_utils::{_recipient_keys, _routing_keys, _routing_keys_1, _service_endpoint};
    use crate::aries::service::AriesService;

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
            .set_service_endpoint("https://dummy.dummy/dummy".parse().expect("valid url"))
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
