use std::collections::HashMap;

use display_as_json::Display;
use serde::{Deserialize, Serialize};
use serde_json::Value;

use super::{
    types::{uri::Uri, url::Url},
    utils::OneOrList,
};
use crate::error::DidDocumentBuilderError;

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Display)]
#[serde(rename_all = "camelCase")]
pub struct Service {
    id: Uri,
    #[serde(rename = "type")]
    service_type: OneOrList<String>,
    service_endpoint: Url,
    #[serde(flatten)]
    #[serde(skip_serializing_if = "HashMap::is_empty")]
    extra: HashMap<String, Value>,
}

impl Service {
    pub fn new(
        id: Uri,
        service_endpoint: Url,
        service_type: OneOrList<String>,
        extra: HashMap<String, Value>,
    ) -> Service {
        Service {
            id,
            service_endpoint,
            service_type,
            extra,
        }
    }

    pub fn id(&self) -> &Uri {
        &self.id
    }

    pub fn service_type(&self) -> &OneOrList<String> {
        &self.service_type
    }

    pub fn service_endpoint(&self) -> &Url {
        &self.service_endpoint
    }

    pub fn extra(&self) -> &HashMap<String, Value> {
        &self.extra
    }

    pub fn extra_field_as_as<T: for<'de> serde::Deserialize<'de>>(
        &self,
        key: &str,
    ) -> Result<T, DidDocumentBuilderError> {
        match self.extra.get(key) {
            None => Err(DidDocumentBuilderError::CustomError(format!(
                "Extra field {} not found",
                key
            ))),
            Some(value) => serde_json::from_value::<T>(value.clone()).map_err(|_err| {
                DidDocumentBuilderError::CustomError(format!(
                    "Extra field {} is not of type {}",
                    key,
                    std::any::type_name::<T>()
                ))
            }),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn create_valid_uri() -> Uri {
        Uri::new("http://example.com").unwrap()
    }

    #[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Default)]
    #[serde(rename_all = "camelCase")]
    pub struct ExtraSov {
        pub priority: u32,
        pub recipient_keys: Vec<String>,
        pub routing_keys: Vec<String>,
    }

    #[test]
    fn test_service_builder_basic() {
        let id = create_valid_uri();
        let service_endpoint = "http://example.com/endpoint";
        let service_type = "DIDCommMessaging".to_string();

        let service = Service::new(
            id.clone(),
            service_endpoint.try_into().unwrap(),
            OneOrList::One(service_type.clone()),
            HashMap::default(),
        );

        assert_eq!(service.id(), &id);
        assert_eq!(service.service_endpoint().as_ref(), service_endpoint);
        assert_eq!(service.service_type(), &OneOrList::One(service_type));
    }

    #[test]
    fn test_service_serde() {
        let service_serialized = r#"{
          "id": "did:sov:HR6vs6GEZ8rHaVgjg2WodM#did-communication",
          "type": "did-communication",
          "priority": 0,
          "recipientKeys": [
            "did:sov:HR6vs6GEZ8rHaVgjg2WodM#key-agreement-1"
          ],
          "routingKeys": [],
          "accept": [
            "didcomm/aip2;env=rfc19"
          ],
          "serviceEndpoint": "https://example.com/endpoint"
        }"#;

        let service: Service = serde_json::from_str(service_serialized).unwrap();
        assert_eq!(
            service.id(),
            &Uri::new("did:sov:HR6vs6GEZ8rHaVgjg2WodM#did-communication").unwrap()
        );
        assert_eq!(
            service.service_type(),
            &OneOrList::One("did-communication".to_string())
        );
        assert_eq!(
            service.service_endpoint().as_ref(),
            "https://example.com/endpoint"
        );
        assert_eq!(service.extra_field_as_as::<u32>("priority").unwrap(), 0);
        assert_eq!(
            service
                .extra_field_as_as::<Vec<String>>("recipientKeys")
                .unwrap(),
            vec!["did:sov:HR6vs6GEZ8rHaVgjg2WodM#key-agreement-1".to_string()]
        );
    }
}
