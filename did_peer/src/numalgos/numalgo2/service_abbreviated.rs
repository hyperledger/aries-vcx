use did_doc::schema::types::url::Url;
use did_doc_sov::extra_fields::{AcceptType, KeyKind};
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize)]
pub struct ServiceAbbreviated {
    #[serde(rename = "t")]
    service_type: String,
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
    pub fn builder() -> ServiceAbbreviatedTypeBuilder {
        ServiceAbbreviatedTypeBuilder::default()
    }

    pub fn service_type(&self) -> &str {
        self.service_type.as_ref()
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
            service_type: service_type.to_string(),
            service_endpoint: service_endpoint.parse().unwrap(),
            routing_keys: routing_keys.to_vec(),
            accept: accept.to_vec(),
        }
    }
}

#[derive(Default)]
pub struct ServiceAbbreviatedTypeBuilder;

pub struct ServiceAbbreviatedEndpointBuilder {
    service_type: String,
}

pub struct ServiceAbbreviatedCompleteBuilder {
    service_type: String,
    service_endpoint: Url,
    routing_keys: Vec<KeyKind>,
    accept: Vec<AcceptType>,
}

impl ServiceAbbreviatedTypeBuilder {
    pub fn set_service_type(self, service_type: String) -> ServiceAbbreviatedEndpointBuilder {
        ServiceAbbreviatedEndpointBuilder { service_type }
    }
}

impl ServiceAbbreviatedEndpointBuilder {
    pub fn set_service_endpoint(self, service_endpoint: Url) -> ServiceAbbreviatedCompleteBuilder {
        ServiceAbbreviatedCompleteBuilder {
            service_type: self.service_type,
            service_endpoint,
            routing_keys: Vec::new(),
            accept: Vec::new(),
        }
    }
}

impl ServiceAbbreviatedCompleteBuilder {
    pub fn set_routing_keys(&mut self, routing_keys: Vec<KeyKind>) -> &mut ServiceAbbreviatedCompleteBuilder {
        self.routing_keys = routing_keys;
        self
    }

    pub fn set_accept_types(&mut self, accept: Vec<AcceptType>) -> &mut ServiceAbbreviatedCompleteBuilder {
        self.accept = accept;
        self
    }

    pub fn build(&self) -> ServiceAbbreviated {
        ServiceAbbreviated {
            service_type: self.service_type.to_owned(),
            service_endpoint: self.service_endpoint.to_owned(),
            routing_keys: self.routing_keys.to_owned(),
            accept: self.accept.to_owned(),
        }
    }
}
