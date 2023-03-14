pub const SERVICE_SUFFIX: &str = "indy";

pub const SERVICE_TYPE: &str = "IndyAgent";

// https://sovrin-foundation.github.io/sovrin/spec/did-method-spec-template.html
#[derive(Debug, Deserialize, Serialize, Clone)]
#[cfg_attr(test, derive(PartialEq, Eq))]
#[serde(rename_all = "camelCase")]
pub struct EndpointDidSov {
    pub endpoint: String,
    #[serde(default)]
    pub routing_keys: Option<Vec<String>>,
    #[serde(default)]
    pub types: Option<Vec<DidSovServiceType>>,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
#[cfg_attr(test, derive(PartialEq, Eq))]
pub enum DidSovServiceType {
    #[serde(rename = "endpoint")] // AIP 1.0
    Endpoint,
    #[serde(rename = "did-communication")] // AIP 2.0
    DidCommunication,
    #[serde(rename = "DIDComm")] // DIDComm V2
    DIDComm,
    #[serde(other)]
    Unknown,
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

    pub fn set_types(mut self, types: Option<Vec<DidSovServiceType>>) -> Self {
        self.types = types;
        self
    }
}

impl Default for EndpointDidSov {
    fn default() -> EndpointDidSov {
        EndpointDidSov {
            endpoint: String::new(),
            routing_keys: Some(Vec::new()),
            types: None,
        }
    }
}

#[cfg(test)]
#[cfg(feature = "general_test")]
mod unit_tests {
    use messages::diddoc::aries::diddoc::test_utils::{_routing_keys, _service_endpoint};

    use crate::{
        common::ledger::service_didsov::{DidSovServiceType, EndpointDidSov},
        utils::devsetup::SetupEmpty,
    };

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

    #[test]
    fn test_didsov_service_serialization() {
        SetupEmpty::init();
        let service1 = EndpointDidSov::create()
            .set_service_endpoint(_service_endpoint())
            .set_routing_keys(Some(_routing_keys()))
            .set_types(Some(vec![
                DidSovServiceType::Endpoint,
                DidSovServiceType::DidCommunication,
                DidSovServiceType::DIDComm,
            ]));

        let expected = json!({
            "endpoint": "http://localhost:8080",
            "routingKeys": [
                "Hezce2UWMZ3wUhVkh2LfKSs8nDzWwzs2Win7EzNN3YaR",
                "3LYuxJBJkngDbvJj4zjx13DBUdZ2P96eNybwd2n9L9AU"
            ],
            "types": ["endpoint", "did-communication", "DIDComm"]
        });
        assert_eq!(expected, json!(&service1));
    }

    #[test]
    fn test_didsov_service_deserialization() {
        SetupEmpty::init();

        let data = json!({
            "endpoint": "http://localhost:8080",
            "routingKeys": [
                "Hezce2UWMZ3wUhVkh2LfKSs8nDzWwzs2Win7EzNN3YaR",
                "3LYuxJBJkngDbvJj4zjx13DBUdZ2P96eNybwd2n9L9AU"
            ],
            "types": ["endpoint", "did-communication", "DIDComm", "foobar"]
        })
        .to_string();

        let deserialized: EndpointDidSov = serde_json::from_str(&data).unwrap();
        assert_eq!(deserialized.endpoint, _service_endpoint());
        assert_eq!(deserialized.routing_keys, Some(_routing_keys()));
        assert_eq!(
            deserialized.types,
            Some(vec![
                DidSovServiceType::Endpoint,
                DidSovServiceType::DidCommunication,
                DidSovServiceType::DIDComm,
                DidSovServiceType::Unknown
            ])
        );
    }
}
