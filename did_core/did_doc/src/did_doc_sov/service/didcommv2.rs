use serde::Serialize;

use super::ServiceType;
use crate::{
    did_doc_sov::{extra_fields::didcommv2::ExtraFieldsDidCommV2, TypedService},
    schema::{
        types::{uri::Uri, url::Url},
        utils::OneOrList,
    },
};

#[derive(Serialize, Clone, Debug, PartialEq)]
pub struct ServiceDidCommV2 {
    #[serde(flatten)]
    service: TypedService<ExtraFieldsDidCommV2>,
}

impl ServiceDidCommV2 {
    pub fn new(id: Uri, service_endpoint: Url, extra: ExtraFieldsDidCommV2) -> Self {
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
