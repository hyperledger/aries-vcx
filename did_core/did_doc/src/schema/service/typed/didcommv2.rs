use serde::Serialize;
use url::Url;

use crate::schema::{
    service::{
        extra_fields::{didcommv2::ExtraFieldsDidCommV2, ServiceAcceptType, ServiceKeyKind},
        typed::{ServiceType, TypedService},
    },
    types::uri::Uri,
    utils::OneOrList,
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
            .set_routing_keys(routing_keys)
            .set_accept(accept)
            .build();
        Self {
            service: TypedService::<ExtraFieldsDidCommV2> {
                id,
                service_type: OneOrList::One(ServiceType::DIDCommV2.to_string()),
                service_endpoint,
                extra,
            },
        }
    }

    pub fn id(&self) -> &Uri {
        self.service.id()
    }

    pub fn service_type(&self) -> ServiceType {
        ServiceType::DIDCommV2
    }

    pub fn service_endpoint(&self) -> Url {
        self.service.service_endpoint().clone()
    }

    pub fn extra(&self) -> &ExtraFieldsDidCommV2 {
        self.service.extra()
    }
}
