use display_as_json::Display;
use serde::{Deserialize, Serialize};
use typed_builder::TypedBuilder;
use url::Url;

use crate::schema::{
    service::{
        service_accept_type::ServiceAcceptType,
        service_key_kind::ServiceKeyKind,
        typed::{ServiceType, TypedService},
    },
    types::uri::Uri,
};

#[derive(Serialize, Clone, Debug, PartialEq)]
pub struct ServiceDidCommV2 {
    #[serde(flatten)]
    service: TypedService<ExtraFieldsDidCommV2>,
}

impl ServiceDidCommV2 {
    pub fn new(
        id: Uri,
        service_endpoint: Url,
        routing_keys: Vec<ServiceKeyKind>,
        accept: Vec<ServiceAcceptType>,
    ) -> Self {
        let extra: ExtraFieldsDidCommV2 = ExtraFieldsDidCommV2::builder()
            .routing_keys(routing_keys)
            .accept(accept)
            .build();
        Self {
            service: TypedService::<ExtraFieldsDidCommV2> {
                id,
                service_type: ServiceType::DIDCommV2,
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

    pub fn extra(&self) -> &ExtraFieldsDidCommV2 {
        self.service.extra()
    }
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Default, Display, TypedBuilder)]
#[serde(rename_all = "camelCase")]
#[serde(deny_unknown_fields)]
pub struct ExtraFieldsDidCommV2 {
    #[serde(skip_serializing_if = "Vec::is_empty")]
    routing_keys: Vec<ServiceKeyKind>,
    #[serde(default)]
    accept: Vec<ServiceAcceptType>,
}

impl ExtraFieldsDidCommV2 {
    pub fn routing_keys(&self) -> &[ServiceKeyKind] {
        self.routing_keys.as_ref()
    }

    pub fn accept(&self) -> &[ServiceAcceptType] {
        self.accept.as_ref()
    }
}
