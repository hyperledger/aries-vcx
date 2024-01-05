use std::collections::HashMap;

use display_as_json::Display;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use service_accept_type::ServiceAcceptType;
use service_key_kind::ServiceKeyKind;
use url::Url;

use crate::{
    error::DidDocumentBuilderError,
    schema::{service::typed::ServiceType, types::uri::Uri, utils::OneOrList},
};

pub mod service_accept_type;
pub mod service_key_kind;
pub mod typed;

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Display)]
#[serde(rename_all = "camelCase")]
pub struct Service {
    id: Uri,
    #[serde(rename = "type")]
    service_type: OneOrList<ServiceType>,
    service_endpoint: Url,
    #[serde(flatten)]
    #[serde(skip_serializing_if = "HashMap::is_empty")]
    extra: HashMap<String, Value>,
}

impl Service {
    pub fn new(
        id: Uri,
        service_endpoint: Url,
        service_type: OneOrList<ServiceType>,
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

    pub fn contains_service_type(&self, tested_service_type: &ServiceType) -> bool {
        match &self.service_type {
            OneOrList::One(service_type) => tested_service_type == service_type,
            OneOrList::List(service_types) => service_types.contains(tested_service_type),
        }
    }

    pub fn service_type(&self) -> &OneOrList<ServiceType> {
        &self.service_type
    }

    pub fn service_types(&self) -> Vec<ServiceType> {
        match &self.service_type {
            OneOrList::One(service_type) => vec![service_type.clone()],
            OneOrList::List(service_types) => service_types.clone(),
        }
    }

    pub fn service_endpoint(&self) -> &Url {
        &self.service_endpoint
    }

    pub fn extra(&self) -> &HashMap<String, Value> {
        &self.extra
    }

    pub fn extra_field_priority(&self) -> Result<u32, DidDocumentBuilderError> {
        self._expected_extra_field_type::<u32>("priority")
    }

    pub fn extra_field_routing_keys(&self) -> Result<Vec<ServiceKeyKind>, DidDocumentBuilderError> {
        self._expected_extra_field_type::<Vec<ServiceKeyKind>>("routingKeys")
    }

    pub fn extra_field_recipient_keys(
        &self,
    ) -> Result<Vec<ServiceKeyKind>, DidDocumentBuilderError> {
        self._expected_extra_field_type::<Vec<ServiceKeyKind>>("recipientKeys")
    }

    pub fn extra_field_accept(&self) -> Result<Vec<ServiceAcceptType>, DidDocumentBuilderError> {
        self._expected_extra_field_type::<Vec<ServiceAcceptType>>("accept")
    }

    fn _expected_extra_field_type<T: for<'de> serde::Deserialize<'de>>(
        &self,
        key: &'static str,
    ) -> Result<T, DidDocumentBuilderError> {
        match self.extra_field_as_as::<T>(key) {
            None => Err(DidDocumentBuilderError::MissingField(key)),
            Some(value) => value,
        }
    }

    pub fn extra_field_as_as<T: for<'de> serde::Deserialize<'de>>(
        &self,
        key: &str,
    ) -> Option<Result<T, DidDocumentBuilderError>> {
        match self.extra.get(key) {
            Some(value) => {
                let result = serde_json::from_value::<T>(value.clone()).map_err(|_err| {
                    DidDocumentBuilderError::CustomError(format!(
                        "Extra field {} is not of type {}",
                        key,
                        std::any::type_name::<T>()
                    ))
                });
                Some(result)
            }
            None => None,
        }
    }

    pub fn add_extra_field_as<T: serde::Serialize>(
        &mut self,
        key: &str,
        value: T,
    ) -> Result<(), DidDocumentBuilderError> {
        let value = serde_json::to_value(value).map_err(|_err| {
            DidDocumentBuilderError::CustomError(format!(
                "Failed to serialize extra field {} as {}",
                key,
                std::any::type_name::<T>()
            ))
        })?;
        self.extra.insert(key.to_string(), value);
        Ok(())
    }

    pub fn add_extra_field_routing_keys(
        &mut self,
        routing_keys: Vec<ServiceKeyKind>,
    ) -> Result<(), DidDocumentBuilderError> {
        self.add_extra_field_as("routingKeys", routing_keys)
    }

    pub fn add_extra_field_recipient_keys(
        &mut self,
        recipient_keys: Vec<ServiceKeyKind>,
    ) -> Result<(), DidDocumentBuilderError> {
        self.add_extra_field_as("recipientKeys", recipient_keys)
    }

    pub fn add_extra_field_accept(
        &mut self,
        accept: Vec<ServiceAcceptType>,
    ) -> Result<(), DidDocumentBuilderError> {
        self.add_extra_field_as("accept", accept)
    }

    pub fn add_extra_field_priority(
        &mut self,
        priority: u32,
    ) -> Result<(), DidDocumentBuilderError> {
        self.add_extra_field_as("priority", priority)
    }
}

#[cfg(test)]
mod tests {
    use std::collections::HashMap;

    use did_parser::DidUrl;
    use serde_json::json;

    use crate::schema::{
        service::{
            service_accept_type::ServiceAcceptType, service_key_kind::ServiceKeyKind,
            typed::ServiceType, Service,
        },
        types::uri::Uri,
        utils::OneOrList,
    };

    #[test]
    fn test_service_builder() {
        let uri_id = Uri::new("http://example.com").unwrap();
        let service_endpoint = "http://example.com/endpoint";
        let service_type = ServiceType::DIDCommV2;

        let service = Service::new(
            uri_id.clone(),
            service_endpoint.try_into().unwrap(),
            OneOrList::One(service_type.clone()),
            HashMap::default(),
        );

        assert_eq!(service.id(), &uri_id);
        assert_eq!(service.service_endpoint().as_ref(), service_endpoint);
        assert_eq!(service.service_types(), vec!(service_type.clone()));
        assert_eq!(service.service_type(), &OneOrList::One(service_type));
    }

    #[test]
    fn test_serde_service_aip1() {
        let service_aip1 = json!({
            "id": "service-0",
            "type": "endpoint",
            "serviceEndpoint": "https://example.com/endpoint"
        })
        .to_string();
        let service = serde_json::from_str::<Service>(&service_aip1).unwrap();

        assert_eq!(service.id().to_string(), "service-0");
        assert_eq!(
            service.service_endpoint().to_string(),
            "https://example.com/endpoint"
        );
        assert_eq!(service.service_types().first().unwrap(), &ServiceType::AIP1);
    }

    #[test]
    fn test_serde_service_didcomm1() {
        let service_didcomm1 = json!({
            "id": "service-0",
            "type": "did-communication",
            "priority": 0,
            "recipientKeys": ["did:sov:HR6vs6GEZ8rHaVgjg2WodM#key-agreement-1"],
            "routingKeys": [ "did:sov:HR6vs6GEZ8rHaVgjg2WodM#key-agreement-2"],
            "accept": ["didcomm/aip2;env=rfc19"],
            "serviceEndpoint": "https://example.com/endpoint"
        })
        .to_string();
        let service = serde_json::from_str::<Service>(&service_didcomm1).unwrap();

        assert_eq!(service.id().to_string(), "service-0");
        assert_eq!(
            service.service_types().first().unwrap(),
            &ServiceType::DIDCommV1
        );
        assert_eq!(
            service.service_endpoint().to_string(),
            "https://example.com/endpoint"
        );

        let recipient_keys = service.extra_field_recipient_keys().unwrap();
        assert_eq!(recipient_keys.len(), 1);
        assert_eq!(
            recipient_keys.first().unwrap(),
            &ServiceKeyKind::Reference(
                DidUrl::parse(String::from(
                    "did:sov:HR6vs6GEZ8rHaVgjg2WodM#key-agreement-1"
                ))
                .unwrap()
            )
        );

        let routing_keys = service.extra_field_routing_keys().unwrap();
        assert_eq!(routing_keys.len(), 1);
        assert_eq!(
            routing_keys.first().unwrap(),
            &ServiceKeyKind::Reference(
                DidUrl::parse(String::from(
                    "did:sov:HR6vs6GEZ8rHaVgjg2WodM#key-agreement-2"
                ))
                .unwrap()
            )
        );

        let accept = service.extra_field_accept().unwrap();
        assert_eq!(accept.len(), 1);
        assert_eq!(accept.first().unwrap(), &ServiceAcceptType::DIDCommV1);

        let priority = service.extra_field_priority().unwrap();
        assert_eq!(priority, 0);
    }

    #[test]
    fn test_serde_service_didcomm2() {
        let service_didcomm2 = json!({
          "id": "service-0",
          "type": "DIDCommMessaging",
          "accept": [ "didcomm/v2"],
          "routingKeys": [],
          "serviceEndpoint": "https://example.com/endpoint"
        })
        .to_string();
        let service = serde_json::from_str::<Service>(&service_didcomm2).unwrap();

        assert_eq!(service.id().to_string(), "service-0");
        assert_eq!(
            service.service_types().first().unwrap(),
            &ServiceType::DIDCommV2
        );
        assert_eq!(
            service.service_endpoint().to_string(),
            "https://example.com/endpoint"
        );

        let accept = service.extra_field_accept().unwrap();
        assert_eq!(accept.len(), 1);
        assert_eq!(accept.first().unwrap(), &ServiceAcceptType::DIDCommV2);
    }
}
