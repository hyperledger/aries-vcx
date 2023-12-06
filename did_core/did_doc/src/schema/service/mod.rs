pub mod extra_fields;
pub mod typed;

use std::collections::HashMap;

use display_as_json::Display;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use url::Url;

use crate::{
    error::DidDocumentBuilderError,
    schema::{types::uri::Uri, utils::OneOrList},
};

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
    use std::collections::HashMap;

    use did_parser::DidUrl;

    use crate::schema::{
        service::{
            extra_fields::{ServiceAcceptType, ServiceKeyKind},
            Service,
        },
        types::uri::Uri,
        utils::OneOrList,
    };
    use crate::schema::service::typed::ServiceType;

    #[test]
    fn test_service_builder() {
        let uri_id = Uri::new("http://example.com").unwrap();
        let service_endpoint = "http://example.com/endpoint";
        let service_type = "DIDCommMessaging".to_string();

        let service = Service::new(
            uri_id.clone(),
            service_endpoint.try_into().unwrap(),
            OneOrList::One(service_type.clone()),
            HashMap::default(),
        );

        assert_eq!(service.id(), &uri_id);
        assert_eq!(service.service_endpoint().as_ref(), service_endpoint);
        assert_eq!(service.service_type(), &OneOrList::One(service_type));
    }

    #[test]
    fn test_serde_service_aip1() {
        let service_aip1: &str = r#"
        {
            "id": "service-0",
            "type": "endpoint",
            "serviceEndpoint": "https://example.com/endpoint"
        }"#;
        let service = serde_json::from_str::<Service>(service_aip1).unwrap();

        assert_eq!(service.id().to_string(), "service-0");
        assert_eq!(
            service.service_endpoint().to_string(),
            "https://example.com/endpoint"
        );
        assert_eq!(
            service.service_type().first().unwrap(),
            ServiceType::AIP1.to_string()
        );
    }

    #[test]
    fn test_serde_service_didcomm1() {
        let service_didcomm1: &str = r#"
        {
            "id": "service-0",
            "type": "did-communication",
            "priority": 0,
            "recipientKeys": [
                "did:sov:HR6vs6GEZ8rHaVgjg2WodM#key-agreement-1"
            ],
            "routingKeys": [
                "did:sov:HR6vs6GEZ8rHaVgjg2WodM#key-agreement-2"
            ],
            "accept": [
                "didcomm/aip2;env=rfc19"
            ],
            "serviceEndpoint": "https://example.com/endpoint"
        }"#;
        let service = serde_json::from_str::<Service>(service_didcomm1).unwrap();

        assert_eq!(service.id().to_string(), "service-0");
        assert_eq!(
            service.service_type().first().unwrap(),
            ServiceType::DIDCommV1.to_string()
        );
        assert_eq!(
            service.service_endpoint().to_string(),
            "https://example.com/endpoint"
        );

        let recipient_keys = service
            .extra_field_as_as::<Vec<ServiceKeyKind>>("recipientKeys")
            .unwrap();
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

        let routing_keys = service
            .extra_field_as_as::<Vec<ServiceKeyKind>>("routingKeys")
            .unwrap();
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

        let accept = service
            .extra_field_as_as::<Vec<ServiceAcceptType>>("accept")
            .unwrap();
        assert_eq!(accept.len(), 1);
        assert_eq!(accept.first().unwrap(), &ServiceAcceptType::DIDCommV1);

        let priority = service.extra_field_as_as::<u32>("priority").unwrap();
        assert_eq!(priority, 0);
    }

    #[test]
    fn test_serde_service_didcomm2() {
        let service_didcomm2: &str = r#"
        {
          "id": "service-0",
          "type": "DIDCommMessaging",
          "accept": [
            "didcomm/v2"
          ],
          "routingKeys": [],
          "serviceEndpoint": "https://example.com/endpoint"
        }"#;
        let service = serde_json::from_str::<Service>(service_didcomm2).unwrap();

        assert_eq!(service.id().to_string(), "service-0");
        assert_eq!(
            service.service_type().first().unwrap(),
            ServiceType::DIDCommV2.to_string()
        );
        assert_eq!(
            service.service_endpoint().to_string(),
            "https://example.com/endpoint"
        );

        let accept = service
            .extra_field_as_as::<Vec<ServiceAcceptType>>("accept")
            .unwrap();
        assert_eq!(accept.len(), 1);
        assert_eq!(accept.first().unwrap(), &ServiceAcceptType::DIDCommV1);
    }
}
