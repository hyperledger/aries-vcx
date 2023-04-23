use std::collections::{HashMap, HashSet};

use serde::{Deserialize, Serialize};
use serde_json::Value;

use crate::error::DIDDocumentBuilderError;

use super::{types::uri::Uri, utils::OneOrList};

type ServiceTypeAlias = OneOrList<String>;

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct Service {
    id: Uri,
    r#type: ServiceTypeAlias,
    service_endpoint: String,
    #[serde(flatten)]
    #[serde(skip_serializing_if = "HashMap::is_empty")]
    #[serde(default)]
    extra: HashMap<String, Value>,
}

impl Service {
    pub fn builder(
        id: Uri,
        service_endpoint: String,
    ) -> Result<ServiceBuilder, DIDDocumentBuilderError> {
        ServiceBuilder::new(id, service_endpoint)
    }

    pub fn id(&self) -> &Uri {
        &self.id
    }

    pub fn r#type(&self) -> &ServiceTypeAlias {
        &self.r#type
    }

    pub fn service_endpoint(&self) -> &str {
        self.service_endpoint.as_ref()
    }

    pub fn extra(&self, key: &str) -> Option<&Value> {
        self.extra.get(key)
    }
}

#[derive(Debug, Default)]
pub struct ServiceBuilder {
    id: Uri,
    r#type: HashSet<String>,
    service_endpoint: String,
    extra: HashMap<String, Value>,
}

impl ServiceBuilder {
    pub fn new(id: Uri, service_endpoint: String) -> Result<Self, DIDDocumentBuilderError> {
        if id.is_empty() {
            return Err(DIDDocumentBuilderError::MissingField("id"));
        }
        if service_endpoint.is_empty() {
            return Err(DIDDocumentBuilderError::MissingField("serviceEndpoint"));
        }
        Ok(Self {
            id,
            service_endpoint,
            ..Default::default()
        })
    }

    pub fn add_type(mut self, r#type: String) -> Result<Self, DIDDocumentBuilderError> {
        if r#type.is_empty() {
            return Err(DIDDocumentBuilderError::MissingField("type"));
        }
        self.r#type.insert(r#type);
        Ok(self)
    }

    pub fn add_extra(mut self, key: String, value: Value) -> Self {
        self.extra.insert(key, value);
        self
    }

    pub fn build(self) -> Result<Service, DIDDocumentBuilderError> {
        if self.r#type.is_empty() {
            Err(DIDDocumentBuilderError::MissingField("type"))
        } else {
            Ok(Service {
                id: self.id,
                r#type: OneOrList::List(self.r#type.into_iter().collect()),
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
        Uri::new("http://example.com".to_string()).unwrap()
    }

    #[test]
    fn test_service_builder_basic() {
        let id = create_valid_uri();
        let service_endpoint = "http://example.com/endpoint".to_string();
        let r#type = "DIDCommMessaging".to_string();

        let service = ServiceBuilder::new(id.clone(), service_endpoint.clone())
            .unwrap()
            .add_type(r#type.clone())
            .unwrap()
            .build()
            .unwrap();

        assert_eq!(service.id(), &id);
        assert_eq!(service.service_endpoint(), &service_endpoint);
        assert_eq!(service.r#type(), &OneOrList::List(vec![r#type]));
    }

    #[test]
    fn test_service_builder_add_extra() {
        let id = create_valid_uri();
        let service_endpoint = "http://example.com/endpoint".to_string();
        let r#type = "DIDCommMessaging".to_string();
        let extra_key = "foo".to_string();
        let extra_value = Value::String("bar".to_string());

        let service = ServiceBuilder::new(id.clone(), service_endpoint.clone())
            .unwrap()
            .add_type(r#type.clone())
            .unwrap()
            .add_extra(extra_key.clone(), extra_value.clone())
            .build()
            .unwrap();

        assert_eq!(service.extra(&extra_key).unwrap(), &extra_value);
    }

    #[test]
    fn test_service_builder_add_duplicate_types() {
        let id = create_valid_uri();
        let service_endpoint = "http://example.com/endpoint".to_string();
        let r#type = "DIDCommMessaging".to_string();

        let service = ServiceBuilder::new(id.clone(), service_endpoint.clone())
            .unwrap()
            .add_type(r#type.clone())
            .unwrap()
            .add_type(r#type.clone())
            .unwrap()
            .build()
            .unwrap();

        assert_eq!(service.r#type(), &OneOrList::List(vec![r#type]));
    }

    #[test]
    fn test_service_builder_constructor_missing_service_endpoint() {
        let id = create_valid_uri();

        let res = ServiceBuilder::new(id.clone(), "".to_string());
        assert!(res.is_err());
    }

    #[test]
    fn test_service_builder_add_type_missing_type() {
        let id = create_valid_uri();
        let service_endpoint = "http://example.com/endpoint".to_string();

        let res = ServiceBuilder::new(id.clone(), service_endpoint.clone())
            .unwrap()
            .add_type("".to_string());
        assert!(res.is_err());
    }
}
