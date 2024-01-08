use std::{collections::HashMap, str::FromStr};

use did_doc::schema::{
    service::{
        service_accept_type::ServiceAcceptType, service_key_kind::ServiceKeyKind,
        typed::ServiceType, Service,
    },
    types::uri::Uri,
    utils::OneOrList,
};
use serde::{Deserialize, Serialize};
use serde_json::from_value;
use url::Url;

use crate::error::DidPeerError;

#[derive(Serialize, Deserialize, Debug)]
pub struct ServiceAbbreviatedDidPeer2 {
    // https://identity.foundation/peer-did-method-spec/#generating-a-didpeer2
    //   > For use with did:peer:2, service id attributes MUST be relative.
    //   > The service MAY omit the id; however, this is NOT RECOMMEDED (clarified).
    id: Option<Uri>,
    #[serde(rename = "t")]
    service_type: OneOrList<String>,
    #[serde(rename = "s")]
    service_endpoint: Url,
    #[serde(rename = "r")]
    #[serde(default)]
    #[serde(skip_serializing_if = "Vec::is_empty")]
    routing_keys: Vec<ServiceKeyKind>,
    #[serde(rename = "a")]
    #[serde(default)]
    #[serde(skip_serializing_if = "Vec::is_empty")]
    accept: Vec<ServiceAcceptType>,
}

impl ServiceAbbreviatedDidPeer2 {
    pub fn new(
        id: Option<Uri>,
        service_type: OneOrList<String>,
        service_endpoint: Url,
        routing_keys: Vec<ServiceKeyKind>,
        accept: Vec<ServiceAcceptType>,
    ) -> Self {
        Self {
            id,
            service_type,
            service_endpoint,
            routing_keys,
            accept,
        }
    }

    pub fn service_type(&self) -> &OneOrList<String> {
        &self.service_type
    }

    pub fn service_endpoint(&self) -> &Url {
        &self.service_endpoint
    }

    pub fn routing_keys(&self) -> &[ServiceKeyKind] {
        &self.routing_keys
    }

    pub fn accept(&self) -> &[ServiceAcceptType] {
        &self.accept
    }
}

pub(crate) fn abbreviate_service(
    service: &Service,
) -> Result<ServiceAbbreviatedDidPeer2, DidPeerError> {
    let service_endpoint = service.service_endpoint().clone();
    let routing_keys = {
        service
            .extra()
            .get("routingKeys")
            .map(|value| {
                from_value::<Vec<ServiceKeyKind>>(value.clone()).map_err(|_| {
                    DidPeerError::ParsingError(format!(
                        "Could not parse routing keys as Vector of Strings. Value of \
                         routing_keys: {}",
                        value
                    ))
                })
            })
            .unwrap_or_else(|| Ok(vec![]))
    }?;
    let accept = {
        service
            .extra()
            .get("accept")
            .map(|value| {
                from_value::<Vec<ServiceAcceptType>>(value.clone()).map_err(|_| {
                    DidPeerError::ParsingError(format!(
                        "Could not parse accept as Vector of Strings. Value of accept: {}",
                        value
                    ))
                })
            })
            .unwrap_or_else(|| Ok(vec![]))
    }?;
    let service_type = service.service_type().clone();
    let service_types_abbreviated = match service_type {
        OneOrList::List(service_types) => {
            let abbreviated_list = service_types
                .iter()
                .map(|value| {
                    if value == &ServiceType::DIDCommV2 {
                        "dm".to_string()
                    } else {
                        value.to_string()
                    }
                })
                .collect();
            OneOrList::List(abbreviated_list)
        }
        OneOrList::One(service_type) => {
            if service_type == ServiceType::DIDCommV2 {
                OneOrList::One("dm".to_string())
            } else {
                OneOrList::One(service_type.to_string())
            }
        }
    };
    Ok(ServiceAbbreviatedDidPeer2::new(
        Some(service.id().clone()),
        service_types_abbreviated,
        service_endpoint,
        routing_keys,
        accept,
    ))
}

pub(crate) fn deabbreviate_service(
    abbreviated: ServiceAbbreviatedDidPeer2,
    index: usize,
) -> Result<Service, DidPeerError> {
    let service_type = match abbreviated.service_type().clone() {
        // todo: get rid of internal OneOrList representation - just have list
        //       also don't expose OneOrList to users, it's just matter of de/serialization
        OneOrList::One(service_type) => {
            let typed = match service_type.as_str() {
                "dm" => ServiceType::DIDCommV2,
                _ => ServiceType::from_str(&service_type)?,
            };
            OneOrList::One(typed)
        }
        OneOrList::List(service_types) => {
            let mut typed = Vec::new();
            for service_type in service_types.iter() {
                let service = match service_type.as_str() {
                    "dm" => ServiceType::DIDCommV2,
                    _ => ServiceType::from_str(service_type)?,
                };
                typed.push(service);
            }
            OneOrList::List(typed)
        }
    };

    // todo: >>> we created custom error for uniresid wrapper, now we'll need conversion across the
    // board.
    let id = abbreviated
        .id
        .clone()
        .unwrap_or(format!("#service-{}", index).parse()?);

    let mut service = Service::new(
        id,
        abbreviated.service_endpoint().clone(),
        service_type,
        HashMap::default(),
    );
    let routing_keys = abbreviated.routing_keys();
    if !routing_keys.is_empty() {
        service.add_extra_field_routing_keys(routing_keys.to_vec())?;
    }
    let accept = abbreviated.accept();
    if !accept.is_empty() {
        service.add_extra_field_accept(accept.to_vec())?;
    }
    Ok(service)
}

#[cfg(test)]
mod tests {
    use did_doc::schema::{
        service::{
            service_accept_type::ServiceAcceptType, service_key_kind::ServiceKeyKind,
            typed::ServiceType, Service,
        },
        types::uri::Uri,
        utils::OneOrList,
    };
    use serde_json::json;
    use url::Url;

    use crate::peer_did::numalgos::numalgo2::service_abbreviation::{
        abbreviate_service, deabbreviate_service, ServiceAbbreviatedDidPeer2,
    };

    #[test]
    fn test_deabbreviate_service_type_value_dm() {
        let service_abbreviated = ServiceAbbreviatedDidPeer2 {
            id: Some(Uri::new("#service-0").unwrap()),
            service_type: OneOrList::One("dm".to_string()),
            service_endpoint: Url::parse("https://example.org").unwrap(),
            routing_keys: vec![],
            accept: vec![],
        };
        let index = 0;

        let service = deabbreviate_service(service_abbreviated, index).unwrap();
        assert_eq!(
            service.service_type().clone(),
            OneOrList::One(ServiceType::DIDCommV2)
        );
    }

    #[test]
    fn test_deabbreviate_service() {
        let routing_keys = vec![ServiceKeyKind::Value("key1".to_string())];
        let accept = vec![ServiceAcceptType::DIDCommV1];
        let service_endpoint = Url::parse("https://example.com/endpoint").unwrap();
        let service_type = OneOrList::One(ServiceType::Other("foobar".to_string()));
        let service_id = Uri::new("#service-0").unwrap();
        let service_abbreviated = ServiceAbbreviatedDidPeer2 {
            id: Some(service_id),
            service_type: OneOrList::One("foobar".to_string()),
            service_endpoint: service_endpoint.clone(),
            routing_keys: routing_keys.clone(),
            accept: accept.clone(),
        };
        let index = 0;

        let service = deabbreviate_service(service_abbreviated, index).unwrap();
        assert_eq!(service.service_type().clone(), service_type);
        assert_eq!(service.service_endpoint().clone(), service_endpoint);
        assert_eq!(service.extra_field_routing_keys().unwrap(), routing_keys);
        assert_eq!(service.extra_field_accept().unwrap(), accept);
    }

    #[test]
    fn test_abbreviate_deabbreviate_service() {
        let service: Service = serde_json::from_value(json!({
            "id": "#0",
            "type": [
                "did-communication"
            ],
            "serviceEndpoint": "http://dummyurl.org/",
            "routingKeys": [],
            "accept": [
                "didcomm/aip2;env=rfc19"
            ],
            "priority": 0,
            "recipientKeys": [
                "did:key:z6MkkukgyKAdBN46UAHvia2nxmioo74F6YdvW1nBT1wfKKha"
            ]
        }))
        .unwrap();
        let abbreviated = serde_json::to_value(abbreviate_service(&service).unwrap()).unwrap();
        // Note: the abbreviation is lossy! "recipient_keys", "priority" are legacy concept, we
        // shouldn't mind
        let expected = json!(
            {
                "id": "#0",
                "t": ["did-communication"],
                "s": "http://dummyurl.org/",
                "a": ["didcomm/aip2;env=rfc19"]
            }
        );
        assert_eq!(abbreviated, expected);
    }
}
