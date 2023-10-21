use std::collections::HashSet;

use serde::{Deserialize, Serialize};

use super::{
    types::{uri::Uri, url::Url},
    utils::OneOrList,
};
use crate::error::DidDocumentBuilderError;

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct Service<E> {
    id: Uri,
    #[serde(rename = "type")]
    service_type: OneOrList<String>,
    service_endpoint: Url,
    #[serde(flatten)]
    extra: E,
}

impl<E> Service<E> {
    pub fn builder(id: Uri, service_endpoint: Url, extra: E) -> ServiceBuilder<E> {
        ServiceBuilder::new(id, service_endpoint, extra)
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

    pub fn extra(&self) -> &E {
        &self.extra
    }
}

#[derive(Debug)]
pub struct ServiceBuilder<E> {
    id: Uri,
    service_type: HashSet<String>,
    service_endpoint: Url,
    extra: E,
}

impl<E> ServiceBuilder<E> {
    pub fn new(id: Uri, service_endpoint: Url, extra: E) -> Self {
        Self {
            id,
            service_type: Default::default(),
            service_endpoint,
            extra,
        }
    }

    pub fn add_service_type(
        self,
        service_type: String,
    ) -> Result<ServiceBuilder<E>, DidDocumentBuilderError> {
        if service_type.is_empty() {
            return Err(DidDocumentBuilderError::InvalidInput(
                "Invalid service type: empty string".into(),
            ));
        }
        if self.service_type.contains(&service_type) {
            return Err(DidDocumentBuilderError::InvalidInput(
                "Service type was already included".into(),
            ));
        }
        let mut service_types = self.service_type.clone();
        service_types.insert(service_type);
        Ok(ServiceBuilder {
            id: self.id,
            service_type: service_types,
            service_endpoint: self.service_endpoint,
            extra: self.extra,
        })
    }

    pub fn build(self) -> Service<E> {
        let service_type = match self.service_type.len() {
            1 => OneOrList::One(self.service_type.into_iter().next().unwrap()),
            _ => OneOrList::List(self.service_type.into_iter().collect()),
        };
        Service {
            id: self.id,
            service_type,
            service_endpoint: self.service_endpoint,
            extra: self.extra,
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

        let service = ServiceBuilder::<ExtraSov>::new(
            id.clone(),
            service_endpoint.try_into().unwrap(),
            Default::default(),
        )
        .add_service_type(service_type.clone())
        .unwrap()
        .build();

        assert_eq!(service.id(), &id);
        assert_eq!(service.service_endpoint().as_ref(), service_endpoint);
        assert_eq!(service.service_type(), &OneOrList::One(service_type));
    }

    #[test]
    fn test_service_builder_add_extra() {
        let id = create_valid_uri();
        let service_endpoint = "http://example.com/endpoint";
        let service_type = "DIDCommMessaging".to_string();
        let recipient_keys = vec!["foo".to_string()];
        let routing_keys = vec!["bar".to_string()];
        let extra = ExtraSov {
            priority: 0,
            recipient_keys: recipient_keys.clone(),
            routing_keys: routing_keys.clone(),
        };

        let service =
            ServiceBuilder::<ExtraSov>::new(id, service_endpoint.try_into().unwrap(), extra)
                .add_service_type(service_type)
                .unwrap()
                .build();

        assert_eq!(service.extra().recipient_keys, recipient_keys);
        assert_eq!(service.extra().routing_keys, routing_keys);
    }

    #[test]
    fn test_service_builder_add_duplicate_types() {
        let id = create_valid_uri();
        let service_endpoint = "http://example.com/endpoint";
        let service_type = "DIDCommMessaging".to_string();

        let service = ServiceBuilder::<ExtraSov>::new(
            id,
            service_endpoint.try_into().unwrap(),
            Default::default(),
        )
        .add_service_type(service_type.clone())
        .unwrap()
        .add_service_type(service_type.clone())
        .unwrap()
        .build();

        assert_eq!(service.service_type(), &OneOrList::One(service_type));
    }

    #[test]
    fn test_service_builder_add_type_missing_type() {
        let id = create_valid_uri();
        let service_endpoint = "http://example.com/endpoint";

        let res = ServiceBuilder::<ExtraSov>::new(
            id,
            service_endpoint.try_into().unwrap(),
            Default::default(),
        )
        .add_service_type("".to_string());
        assert!(res.is_err());
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

        let service: Service<ExtraSov> = serde_json::from_str(service_serialized).unwrap();
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
        assert_eq!(service.extra().priority, 0);
        assert_eq!(
            service.extra().recipient_keys,
            vec!["did:sov:HR6vs6GEZ8rHaVgjg2WodM#key-agreement-1".to_string()]
        );
    }
}
