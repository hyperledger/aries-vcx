use did_doc::schema::{service::Service, types::url::Url, utils::OneOrList};
use did_doc_sov::extra_fields::{AcceptType, ExtraFieldsSov, KeyKind};
use serde::{Deserialize, Serialize};

use crate::error::DidPeerError;

#[derive(Serialize, Deserialize)]
pub struct ServiceAbbreviated {
    #[serde(rename = "t")]
    _type: String,
    #[serde(rename = "s")]
    service_endpoint: Url,
    #[serde(rename = "r")]
    #[serde(default)]
    #[serde(skip_serializing_if = "Vec::is_empty")]
    routing_keys: Vec<KeyKind>,
    #[serde(rename = "a")]
    #[serde(default)]
    #[serde(skip_serializing_if = "Vec::is_empty")]
    accept: Vec<AcceptType>,
}

impl ServiceAbbreviated {
    pub fn service_type(&self) -> &str {
        self._type.as_ref()
    }

    pub fn service_endpoint(&self) -> &str {
        self.service_endpoint.as_ref()
    }

    pub fn routing_keys(&self) -> &[KeyKind] {
        self.routing_keys.as_ref()
    }

    pub fn accept(&self) -> &[AcceptType] {
        self.accept.as_ref()
    }

    #[cfg(test)]
    pub(crate) fn from_parts(
        service_type: &str,
        service_endpoint: &str,
        routing_keys: &[KeyKind],
        accept: &[AcceptType],
    ) -> Self {
        Self {
            _type: service_type.to_string(),
            service_endpoint: service_endpoint.parse().unwrap(),
            routing_keys: routing_keys.to_vec(),
            accept: accept.to_vec(),
        }
    }
}

impl TryFrom<&Service<ExtraFieldsSov>> for ServiceAbbreviated {
    type Error = DidPeerError;

    fn try_from(value: &Service<ExtraFieldsSov>) -> Result<Self, Self::Error> {
        let service_endpoint = value.service_endpoint().clone();
        let (routing_keys, accept) = match value.extra() {
            ExtraFieldsSov::DIDCommV2(extra) => (extra.routing_keys().to_vec(), extra.accept().to_vec()),
            ExtraFieldsSov::DIDCommV1(extra) => (extra.routing_keys().to_vec(), extra.accept().to_vec()),
            _ => (vec![], vec![]),
        };
        let service_type = match value.service_type() {
            OneOrList::One(service_type) => service_type,
            OneOrList::List(service_types) => {
                if let Some(first_service) = service_types.first() {
                    first_service
                } else {
                    return Err(DidPeerError::InvalidServiceType);
                }
            }
        };

        let service_type_abbr = if service_type.to_lowercase() == "didcommmessaging" {
            "dm"
        } else {
            service_type
        };

        Ok(Self {
            _type: service_type_abbr.to_string(),
            service_endpoint,
            routing_keys,
            accept,
        })
    }
}
