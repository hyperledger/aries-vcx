use std::collections::HashMap;

use display_as_json::Display;
use serde::{Deserialize, Serialize};
use typed_builder::TypedBuilder;
use url::Url;

use crate::{
    error::DidDocumentBuilderError,
    schema::{
        service::{
            service_accept_type::ServiceAcceptType,
            service_key_kind::ServiceKeyKind,
            typed::{ServiceType, TypedService},
            Service,
        },
        types::uri::Uri,
        utils::OneOrList,
    },
};

#[derive(Serialize, Clone, Debug, PartialEq)]
pub struct ServiceDidCommV1 {
    #[serde(flatten)]
    service: TypedService<ExtraFieldsDidCommV1>,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Default, Display, TypedBuilder)]
#[serde(rename_all = "camelCase")]
#[serde(deny_unknown_fields)]
pub struct ExtraFieldsDidCommV1 {
    priority: u32,
    recipient_keys: Vec<ServiceKeyKind>,
    routing_keys: Vec<ServiceKeyKind>,
    #[serde(default)]
    accept: Vec<ServiceAcceptType>,
}

impl ServiceDidCommV1 {
    pub fn new(
        id: Uri,
        service_endpoint: Url,
        priority: u32,
        recipient_keys: Vec<ServiceKeyKind>,
        routing_keys: Vec<ServiceKeyKind>,
    ) -> Self {
        let extra = ExtraFieldsDidCommV1::builder()
            .priority(priority)
            .recipient_keys(recipient_keys)
            .routing_keys(routing_keys)
            .accept(vec![ServiceAcceptType::DIDCommV1])
            .build();
        Self {
            service: TypedService::<ExtraFieldsDidCommV1> {
                id,
                service_type: ServiceType::DIDCommV1,
                service_endpoint,
                extra,
            },
        }
    }

    pub fn id(&self) -> &Uri {
        self.service.id()
    }

    pub fn service_endpoint(&self) -> Url {
        self.service.service_endpoint().clone()
    }

    pub fn extra(&self) -> &ExtraFieldsDidCommV1 {
        self.service.extra()
    }
}

impl TryFrom<ServiceDidCommV1> for Service {
    type Error = DidDocumentBuilderError;

    fn try_from(did_comm_service: ServiceDidCommV1) -> Result<Self, Self::Error> {
        let mut extra_fields = HashMap::new();
        extra_fields.insert(
            "priority".to_string(),
            serde_json::Value::from(did_comm_service.extra().priority()),
        );
        extra_fields.insert(
            "recipientKeys".to_string(),
            serde_json::to_value(did_comm_service.extra().recipient_keys())?,
        );
        extra_fields.insert(
            "routingKeys".to_string(),
            serde_json::to_value(did_comm_service.extra().routing_keys())?,
        );
        extra_fields.insert(
            "accept".to_string(),
            serde_json::to_value(did_comm_service.extra().accept())?,
        );

        Ok(Service::new(
            did_comm_service.id().clone(),
            did_comm_service.service_endpoint(),
            OneOrList::One(ServiceType::DIDCommV1),
            extra_fields,
        ))
    }
}

impl ExtraFieldsDidCommV1 {
    pub fn priority(&self) -> u32 {
        self.priority
    }

    pub fn recipient_keys(&self) -> &[ServiceKeyKind] {
        self.recipient_keys.as_ref()
    }

    pub fn routing_keys(&self) -> &[ServiceKeyKind] {
        self.routing_keys.as_ref()
    }

    pub fn accept(&self) -> &[ServiceAcceptType] {
        self.accept.as_ref()
    }
}
