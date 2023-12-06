pub mod didcommv1;
pub mod didcommv2;

use std::{fmt, fmt::Display};

use serde::{de, de::Visitor, Deserialize, Deserializer, Serialize, Serializer};
use url::Url;

use crate::schema::{types::uri::Uri, utils::OneOrList};

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
pub struct TypedService<E> {
    id: Uri,
    #[serde(rename = "type")]
    service_type: OneOrList<String>,
    service_endpoint: Url,
    #[serde(flatten)]
    extra: E,
}

impl<E> TypedService<E> {
    pub fn id(&self) -> &Uri {
        &self.id
    }

    pub fn service_type(&self) -> &OneOrList<String> {
        &self.service_type
    }

    pub fn service_endpoint(&self) -> &Url {
        &self.service_endpoint
    }

    pub fn extra(&self) -> &E {
        &self.extra
    }
}

const SERVICE_TYPE_AIP1: &str = "endpoint";
const SERVICE_TYPE_DIDCOMMV1: &str = "did-communication";
const SERVICE_TYPE_DIDCOMMV2: &str = "DIDCommMessaging";
const SERVICE_TYPE_LEGACY: &str = "IndyAgent";

#[derive(Clone, Debug, PartialEq)]
pub enum ServiceType {
    AIP1,
    DIDCommV1,
    DIDCommV2,
    Legacy,
    Other(String),
}

impl Display for ServiceType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ServiceType::AIP1 => write!(f, "endpoint"),
            ServiceType::DIDCommV1 => write!(f, "did-communication"),
            // Interop note: AFJ useses DIDComm, Acapy uses DIDCommMessaging
            // Not matching spec:
            // * did:sov method - https://sovrin-foundation.github.io/sovrin/spec/did-method-spec-template.html#crud-operation-definitions
            // Matching spec:
            // * did:peer method - https://identity.foundation/peer-did-method-spec/#multi-key-creation
            // * did core - https://www.w3.org/TR/did-spec-registries/#didcommmessaging
            // * didcommv2 - https://identity.foundation/didcomm-messaging/spec/#service-endpoint
            ServiceType::DIDCommV2 => write!(f, "DIDCommMessaging"),
            ServiceType::Legacy => write!(f, "IndyAgent"),
            ServiceType::Other(other) => write!(f, "{}", other),
        }
    }
}

impl Serialize for ServiceType {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        match self {
            ServiceType::AIP1 => serializer.serialize_str(SERVICE_TYPE_AIP1),
            ServiceType::DIDCommV1 => serializer.serialize_str(SERVICE_TYPE_DIDCOMMV1),
            ServiceType::DIDCommV2 => serializer.serialize_str(SERVICE_TYPE_DIDCOMMV2),
            ServiceType::Legacy => serializer.serialize_str(SERVICE_TYPE_LEGACY),
            ServiceType::Other(ref value) => serializer.serialize_str(value),
        }
    }
}

impl<'de> Deserialize<'de> for ServiceType {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        struct StringVisitor;

        impl<'de> Visitor<'de> for StringVisitor {
            type Value = ServiceType;

            fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
                formatter.write_str("a string")
            }

            fn visit_str<E>(self, value: &str) -> Result<Self::Value, E>
            where
                E: de::Error,
            {
                match value {
                    SERVICE_TYPE_AIP1 => Ok(ServiceType::AIP1),
                    SERVICE_TYPE_DIDCOMMV1 => Ok(ServiceType::DIDCommV1),
                    SERVICE_TYPE_DIDCOMMV2 => Ok(ServiceType::DIDCommV2),
                    SERVICE_TYPE_LEGACY => Ok(ServiceType::Legacy),
                    _ => Ok(ServiceType::Other(value.to_owned())),
                }
            }
        }

        deserializer.deserialize_str(StringVisitor)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_service_type_serialize() {
        let service_type = ServiceType::AIP1;
        let serialized = serde_json::to_string(&service_type).unwrap();
        assert_eq!(serialized, "\"endpoint\"");

        let service_type = ServiceType::DIDCommV1;
        let serialized = serde_json::to_string(&service_type).unwrap();
        assert_eq!(serialized, "\"did-communication\"");

        let service_type = ServiceType::DIDCommV2;
        let serialized = serde_json::to_string(&service_type).unwrap();
        assert_eq!(serialized, "\"DIDCommMessaging\"");

        let service_type = ServiceType::Legacy;
        let serialized = serde_json::to_string(&service_type).unwrap();
        assert_eq!(serialized, "\"IndyAgent\"");

        let service_type = ServiceType::Other("Other".to_string());
        let serialized = serde_json::to_string(&service_type).unwrap();
        assert_eq!(serialized, "\"Other\"");
    }

    #[test]
    fn test_service_type_deserialize() {
        let service_type = ServiceType::AIP1;
        let deserialized: ServiceType = serde_json::from_str("\"endpoint\"").unwrap();
        assert_eq!(deserialized, service_type);

        let service_type = ServiceType::DIDCommV1;
        let deserialized: ServiceType = serde_json::from_str("\"did-communication\"").unwrap();
        assert_eq!(deserialized, service_type);

        let service_type = ServiceType::DIDCommV2;
        let deserialized: ServiceType = serde_json::from_str("\"DIDCommMessaging\"").unwrap();
        assert_eq!(deserialized, service_type);

        let service_type = ServiceType::Legacy;
        let deserialized: ServiceType = serde_json::from_str("\"IndyAgent\"").unwrap();
        assert_eq!(deserialized, service_type);

        let service_type = ServiceType::Other("Other".to_string());
        let deserialized: ServiceType = serde_json::from_str("\"Other\"").unwrap();
        assert_eq!(deserialized, service_type);
    }
}
