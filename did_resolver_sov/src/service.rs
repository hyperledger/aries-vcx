use std::collections::HashSet;
use std::fmt::Display;

use serde::{Deserialize, Deserializer};

#[derive(Debug, Deserialize, Clone, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct EndpointDidSov {
    pub endpoint: String,
    #[serde(default)]
    pub routing_keys: Vec<String>,
    #[serde(
        default = "default_didsov_service_types",
        deserialize_with = "deserialize_didsov_service_types"
    )]
    pub types: HashSet<DidSovServiceType>,
}

#[derive(Debug, Clone, Deserialize, PartialEq, Eq, Hash)]
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

impl Display for DidSovServiceType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            DidSovServiceType::Endpoint => write!(f, "endpoint"),
            DidSovServiceType::DidCommunication => write!(f, "did-communication"),
            DidSovServiceType::DIDComm => write!(f, "DIDComm"),
            DidSovServiceType::Unknown => write!(f, "Unknown"),
        }
    }
}

fn default_didsov_service_types() -> HashSet<DidSovServiceType> {
    vec![
        DidSovServiceType::Endpoint,
        DidSovServiceType::DidCommunication,
    ]
    .into_iter()
    .collect()
}

fn deserialize_didsov_service_types<'de, D>(
    deserializer: D,
) -> Result<HashSet<DidSovServiceType>, D::Error>
where
    D: Deserializer<'de>,
{
    let mut types: HashSet<DidSovServiceType> = Deserialize::deserialize(deserializer)?;
    if types.is_empty() || types.iter().all(|t| *t == DidSovServiceType::Unknown) {
        types = default_didsov_service_types();
    } else {
        types.retain(|t| *t != DidSovServiceType::Unknown);
    }
    Ok(types)
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::from_str;
    use std::iter::FromIterator;

    #[test]
    fn test_deserialize_endpoint_did_sov() {
        let json = r#"{
            "endpoint": "https://example.com",
            "routingKeys": ["key1", "key2"],
            "types": ["endpoint", "did-communication"]
        }"#;
        let endpoint_did_sov: EndpointDidSov = from_str(json).unwrap();
        assert_eq!(endpoint_did_sov.endpoint, "https://example.com");
        assert_eq!(endpoint_did_sov.routing_keys, vec!["key1", "key2"]);
        assert_eq!(
            endpoint_did_sov.types,
            HashSet::from_iter(vec![
                DidSovServiceType::Endpoint,
                DidSovServiceType::DidCommunication,
            ])
        );

        let json = r#"{
            "endpoint": "https://example.com",
            "routingKeys": ["key1", "key2"],
            "types": ["endpoint", "DIDComm"]
        }"#;
        let endpoint_did_sov: EndpointDidSov = from_str(json).unwrap();
        assert_eq!(endpoint_did_sov.endpoint, "https://example.com");
        assert_eq!(endpoint_did_sov.routing_keys, vec!["key1", "key2"]);
        assert_eq!(
            endpoint_did_sov.types,
            HashSet::from_iter(vec![
                DidSovServiceType::Endpoint,
                DidSovServiceType::DIDComm,
            ])
        );

        let json = r#"{
            "endpoint": "https://example.com",
            "routingKeys": ["key1", "key2"],
            "types": ["endpoint", "endpoint"]
        }"#;
        let endpoint_did_sov: EndpointDidSov = from_str(json).unwrap();
        assert_eq!(endpoint_did_sov.endpoint, "https://example.com");
        assert_eq!(endpoint_did_sov.routing_keys, vec!["key1", "key2"]);
        assert_eq!(
            endpoint_did_sov.types,
            HashSet::from_iter(vec![DidSovServiceType::Endpoint,])
        );

        let json = r#"{
            "endpoint": "https://example.com",
            "routingKeys": ["key1", "key2"],
            "types": ["invalid"]
        }"#;
        let endpoint_did_sov: EndpointDidSov = from_str(json).unwrap();
        assert_eq!(endpoint_did_sov.endpoint, "https://example.com");
        assert_eq!(endpoint_did_sov.routing_keys, vec!["key1", "key2"]);
        assert_eq!(endpoint_did_sov.types, default_didsov_service_types());

        let json = r#"{
            "endpoint": "https://example.com",
            "routingKeys": ["key1", "key2"]
        }"#;
        let endpoint_did_sov: EndpointDidSov = from_str(json).unwrap();
        assert_eq!(endpoint_did_sov.endpoint, "https://example.com");
        assert_eq!(endpoint_did_sov.routing_keys, vec!["key1", "key2"]);
        assert_eq!(endpoint_did_sov.types, default_didsov_service_types());

        let json = r#"{
            "endpoint": "https://example.com"
        }"#;
        let endpoint_did_sov: EndpointDidSov = from_str(json).unwrap();
        assert_eq!(endpoint_did_sov.endpoint, "https://example.com");
        assert!(endpoint_did_sov.routing_keys.is_empty());
        assert_eq!(endpoint_did_sov.types, default_didsov_service_types());
    }
}
