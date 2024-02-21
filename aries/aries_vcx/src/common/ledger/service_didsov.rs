use url::Url;

pub const SERVICE_SUFFIX: &str = "indy";

pub const SERVICE_TYPE: &str = "IndyAgent";

// https://sovrin-foundation.github.io/sovrin/spec/did-method-spec-template.html
#[derive(Debug, Deserialize, Serialize, Clone)]
#[cfg_attr(test, derive(PartialEq, Eq))]
#[serde(rename_all = "camelCase")]
pub struct EndpointDidSov {
    pub endpoint: Url,
    #[serde(default)]
    pub routing_keys: Option<Vec<String>>,
    #[serde(default)]
    pub types: Option<Vec<String>>,
}

impl EndpointDidSov {
    pub fn create() -> Self {
        Self::default()
    }

    pub fn set_service_endpoint(mut self, service_endpoint: Url) -> Self {
        self.endpoint = service_endpoint;
        self
    }

    pub fn set_routing_keys(mut self, routing_keys: Option<Vec<String>>) -> Self {
        self.routing_keys = routing_keys;
        self
    }

    pub fn set_types(mut self, types: Option<Vec<String>>) -> Self {
        self.types = types;
        self
    }
}

impl Default for EndpointDidSov {
    fn default() -> EndpointDidSov {
        EndpointDidSov {
            endpoint: "https://dummy.dummy/dummy".parse().expect("valid url"),
            routing_keys: Some(Vec::new()),
            types: None,
        }
    }
}

#[cfg(test)]
mod unit_tests {
    use did_doc::schema::service::typed::ServiceType;
    use diddoc_legacy::aries::diddoc::test_utils::{_routing_keys, _service_endpoint};
    use test_utils::devsetup::SetupMocks;

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
            .set_service_endpoint("http://bogus_endpoint.com".parse().expect("valid url"))
            .set_routing_keys(Some(_routing_keys()));

        let service4 = EndpointDidSov::create()
            .set_service_endpoint(_service_endpoint())
            .set_routing_keys(Some(_routing_keys()));

        assert_eq!(service1, service2);
        assert_eq!(service1, service4);
        assert_ne!(service1, service3);
    }

    #[test]
    fn test_didsov_service_serialization() {
        SetupMocks::init();
        let service1 = EndpointDidSov::create()
            .set_service_endpoint(_service_endpoint())
            .set_routing_keys(Some(_routing_keys()))
            .set_types(Some(vec![
                ServiceType::AIP1.to_string(),
                ServiceType::DIDCommV1.to_string(),
                ServiceType::DIDCommV2.to_string(),
                "foobar".to_string(),
            ]));

        let expected = json!({
            "endpoint": "http://localhost:8080/",
            "routingKeys": [
                "Hezce2UWMZ3wUhVkh2LfKSs8nDzWwzs2Win7EzNN3YaR",
                "3LYuxJBJkngDbvJj4zjx13DBUdZ2P96eNybwd2n9L9AU"
            ],
            "types": ["endpoint", "did-communication", "DIDCommMessaging", "foobar"]
        });
        assert_eq!(expected, json!(&service1));
    }

    #[test]
    fn test_didsov_service_deserialization() {
        SetupMocks::init();
        let data = json!({
            "endpoint": "http://localhost:8080",
            "routingKeys": [
                "Hezce2UWMZ3wUhVkh2LfKSs8nDzWwzs2Win7EzNN3YaR",
                "3LYuxJBJkngDbvJj4zjx13DBUdZ2P96eNybwd2n9L9AU"
            ],
            "types": ["endpoint", "did-communication", "DIDCommMessaging", "foobar"]
        })
        .to_string();

        let deserialized: EndpointDidSov = serde_json::from_str(&data).unwrap();
        assert_eq!(deserialized.endpoint, _service_endpoint());
        assert_eq!(deserialized.routing_keys, Some(_routing_keys()));
        assert_eq!(
            deserialized.types,
            Some(vec![
                ServiceType::AIP1.to_string(),
                ServiceType::DIDCommV1.to_string(),
                ServiceType::DIDCommV2.to_string(),
                "foobar".to_string()
            ])
        );
    }
}
