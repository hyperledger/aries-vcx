use did_doc::schema::{
    service::Service,
    types::{uri::Uri, url::Url},
    utils::OneOrList,
};
use serde::{Deserialize, Serialize};

use crate::{
    error::DidDocumentSovError,
    extra_fields::{didcommv1::ExtraFieldsDidCommV1, ExtraFields},
};

use super::ServiceType;

#[derive(Serialize, Clone, Debug, PartialEq)]
pub struct ServiceDidCommV1 {
    #[serde(flatten)]
    service: Service<ExtraFieldsDidCommV1>,
}

impl ServiceDidCommV1 {
    pub fn new(id: Uri, service_endpoint: Url, extra: ExtraFieldsDidCommV1) -> Result<Self, DidDocumentSovError> {
        Ok(Self {
            service: Service::builder(id, service_endpoint, extra)
                .add_service_type(ServiceType::DIDCommV1.to_string())?
                .build(),
        })
    }

    pub fn id(&self) -> &Uri {
        self.service.id()
    }

    pub fn service_type(&self) -> ServiceType {
        ServiceType::DIDCommV1
    }

    pub fn service_endpoint(&self) -> &Url {
        self.service.service_endpoint()
    }

    pub fn extra(&self) -> &ExtraFieldsDidCommV1 {
        self.service.extra()
    }
}

impl TryFrom<Service<ExtraFields>> for ServiceDidCommV1 {
    type Error = DidDocumentSovError;

    fn try_from(service: Service<ExtraFields>) -> Result<Self, Self::Error> {
        match service.extra() {
            ExtraFields::DIDCommV1(extra) => {
                Self::new(service.id().clone(), service.service_endpoint().clone(), extra.clone())
            }
            _ => Err(DidDocumentSovError::UnexpectedServiceType(
                service.service_type().to_string(),
            )),
        }
    }
}

impl<'de> Deserialize<'de> for ServiceDidCommV1 {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let service = Service::<ExtraFields>::deserialize(deserializer)?;
        match service.service_type() {
            OneOrList::One(service_type) if *service_type == ServiceType::DIDCommV1.to_string() => {}
            OneOrList::List(service_types) if service_types.contains(&ServiceType::DIDCommV1.to_string()) => {}
            _ => return Err(serde::de::Error::custom("Extra fields don't match service type")),
        };
        match service.extra() {
            ExtraFields::DIDCommV1(extra) => Ok(Self {
                service: Service::builder(service.id().clone(), service.service_endpoint().clone(), extra.clone())
                    .add_service_type(ServiceType::DIDCommV1.to_string())
                    .map_err(serde::de::Error::custom)?
                    .build(),
            }),
            _ => Err(serde::de::Error::custom("Extra fields don't match service type")),
        }
    }
}
