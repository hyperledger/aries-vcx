use std::collections::{HashMap, HashSet};

use serde::{Deserialize, Serialize};
use serde_json::Value;

use crate::error::DIDDocumentBuilderError;

use super::{
    types::{uri::Uri, url::Url},
    utils::OneOrList,
};

type ServiceTypeAlias = OneOrList<String>;

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct Service {
    id: Uri,
    #[serde(rename = "type")]
    service_type: ServiceTypeAlias,
    service_endpoint: Url,
    #[serde(flatten)]
    #[serde(skip_serializing_if = "HashMap::is_empty")]
    #[serde(default)]
    extra: HashMap<String, Value>,
}

impl Service {
    pub fn builder(
        id: Uri,
        service_endpoint: Url,
    ) -> Result<ServiceBuilder, DIDDocumentBuilderError> {
        ServiceBuilder::new(id, service_endpoint)
    }

    pub fn id(&self) -> &Uri {
        &self.id
    }

    pub fn service_type(&self) -> &ServiceTypeAlias {
        &self.service_type
    }

    pub fn service_endpoint(&self) -> &str {
        self.service_endpoint.as_ref()
    }

    pub fn extra_field(&self, key: &str) -> Option<&Value> {
        self.extra.get(key)
    }
}

#[derive(Debug)]
pub struct ServiceBuilder {
    id: Uri,
    service_type: HashSet<String>,
    service_endpoint: Url,
    extra: HashMap<String, Value>,
}

impl ServiceBuilder {
    pub fn new(id: Uri, service_endpoint: Url) -> Result<Self, DIDDocumentBuilderError> {
        Ok(Self {
            id,
            service_endpoint,
            service_type: HashSet::new(),
            extra: HashMap::new(),
        })
    }

    pub fn add_service_type(
        mut self,
        service_type: String,
    ) -> Result<Self, DIDDocumentBuilderError> {
        if service_type.is_empty() {
            return Err(DIDDocumentBuilderError::MissingField("type"));
        }
        self.service_type.insert(service_type);
        Ok(self)
    }

    pub fn add_extra_field(mut self, key: String, value: Value) -> Self {
        self.extra.insert(key, value);
        self
    }

    pub fn build(self) -> Result<Service, DIDDocumentBuilderError> {
        if self.service_type.is_empty() {
            Err(DIDDocumentBuilderError::MissingField("type"))
        } else {
            Ok(Service {
                id: self.id,
                service_type: OneOrList::List(self.service_type.into_iter().collect()),
                service_endpoint: self.service_endpoint,
                extra: self.extra,
            })
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn create_valid_uri() -> Uri {
        Uri::new("http://example.com").unwrap()
    }

    #[test]
    fn test_service_builder_basic() {
        let id = create_valid_uri();
        let service_endpoint = "http://example.com/endpoint";
        let service_type = "DIDCommMessaging".to_string();

        let service = ServiceBuilder::new(id.clone(), service_endpoint.try_into().unwrap())
            .unwrap()
            .add_service_type(service_type.clone())
            .unwrap()
            .build()
            .unwrap();

        assert_eq!(service.id(), &id);
        assert_eq!(service.service_endpoint(), service_endpoint);
        assert_eq!(service.service_type(), &OneOrList::List(vec![service_type]));
    }

    #[test]
    fn test_service_builder_add_extra() {
        let id = create_valid_uri();
        let service_endpoint = "http://example.com/endpoint";
        let service_type = "DIDCommMessaging".to_string();
        let extra_key = "foo".to_string();
        let extra_value = Value::String("bar".to_string());

        let service = ServiceBuilder::new(id, service_endpoint.try_into().unwrap())
            .unwrap()
            .add_service_type(service_type)
            .unwrap()
            .add_extra_field(extra_key.clone(), extra_value.clone())
            .build()
            .unwrap();

        assert_eq!(service.extra_field(&extra_key).unwrap(), &extra_value);
    }

    #[test]
    fn test_service_builder_add_duplicate_types() {
        let id = create_valid_uri();
        let service_endpoint = "http://example.com/endpoint";
        let service_type = "DIDCommMessaging".to_string();

        let service = ServiceBuilder::new(id, service_endpoint.try_into().unwrap())
            .unwrap()
            .add_service_type(service_type.clone())
            .unwrap()
            .add_service_type(service_type.clone())
            .unwrap()
            .build()
            .unwrap();

        assert_eq!(service.service_type(), &OneOrList::List(vec![service_type]));
    }

    #[test]
    fn test_service_builder_add_type_missing_type() {
        let id = create_valid_uri();
        let service_endpoint = "http://example.com/endpoint";

        let res = ServiceBuilder::new(id, service_endpoint.try_into().unwrap())
            .unwrap()
            .add_service_type("".to_string());
        assert!(res.is_err());
    }

    #[test]
    fn test_service_serde() {
        let service_serialized = r##"{
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
        }"##;

        let service: Service = serde_json::from_str(service_serialized).unwrap();
        assert_eq!(
            service.id(),
            &Uri::new("did:sov:HR6vs6GEZ8rHaVgjg2WodM#did-communication").unwrap()
        );
        assert_eq!(
            service.service_type(),
            &OneOrList::One("did-communication".to_string())
        );
        assert_eq!(service.service_endpoint(), "https://example.com/endpoint");
        assert_eq!(
            service.extra_field("priority").unwrap(),
            &Value::Number(0.into())
        );
        assert_eq!(
            service.extra_field("recipientKeys").unwrap(),
            &Value::Array(vec![Value::String(
                "did:sov:HR6vs6GEZ8rHaVgjg2WodM#key-agreement-1".to_string()
            )])
        );
    }
}
