use std::fmt::Display;

use did_doc::schema::{
    service::Service,
    types::{uri::Uri, url::Url},
};
use serde::{Deserialize, Serialize};

use crate::{error::DidDocumentSovError, extra_fields::ExtraFields};

pub mod aip1;
pub mod didcommv1;
pub mod didcommv2;
pub mod services_list;

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
pub enum ServiceType {
    AIP1,
    DIDCommV1,
    DIDCommV2,
}

impl Display for ServiceType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ServiceType::AIP1 => write!(f, "endpoint"),
            ServiceType::DIDCommV1 => write!(f, "did-communication"),
            ServiceType::DIDCommV2 => write!(f, "DIDComm"),
        }
    }
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
#[serde(untagged)]
pub enum ServiceSov {
    AIP1(aip1::ServiceAIP1),
    DIDCommV1(didcommv1::ServiceDidCommV1),
    DIDCommV2(didcommv2::ServiceDidCommV2),
}

impl ServiceSov {
    pub fn id(&self) -> &Uri {
        match self {
            ServiceSov::AIP1(service) => service.id(),
            ServiceSov::DIDCommV1(service) => service.id(),
            ServiceSov::DIDCommV2(service) => service.id(),
        }
    }

    pub fn service_type(&self) -> ServiceType {
        match self {
            ServiceSov::AIP1(service) => service.service_type(),
            ServiceSov::DIDCommV1(service) => service.service_type(),
            ServiceSov::DIDCommV2(service) => service.service_type(),
        }
    }

    pub fn service_endpoint(&self) -> &Url {
        match self {
            ServiceSov::AIP1(service) => service.service_endpoint(),
            ServiceSov::DIDCommV1(service) => service.service_endpoint(),
            ServiceSov::DIDCommV2(service) => service.service_endpoint(),
        }
    }

    pub fn extra(&self) -> ExtraFields {
        match self {
            ServiceSov::AIP1(service) => ExtraFields::AIP1(service.extra().to_owned()),
            ServiceSov::DIDCommV1(service) => ExtraFields::DIDCommV1(service.extra().to_owned()),
            ServiceSov::DIDCommV2(service) => ExtraFields::DIDCommV2(service.extra().to_owned()),
        }
    }
}

impl TryFrom<Service<ExtraFields>> for ServiceSov {
    type Error = DidDocumentSovError;

    fn try_from(service: Service<ExtraFields>) -> Result<Self, Self::Error> {
        match service.extra() {
            ExtraFields::AIP1(_extra) => Ok(ServiceSov::AIP1(service.try_into()?)),
            ExtraFields::DIDCommV1(_extra) => Ok(ServiceSov::DIDCommV1(service.try_into()?)),
            ExtraFields::DIDCommV2(_extra) => Ok(ServiceSov::DIDCommV2(service.try_into()?)),
        }
    }
}

impl TryFrom<ServiceSov> for Service<ExtraFields> {
    type Error = DidDocumentSovError;

    fn try_from(service: ServiceSov) -> Result<Self, Self::Error> {
        match service {
            ServiceSov::AIP1(service) => Ok(Service::builder(
                service.id().clone(),
                service.service_endpoint().clone(),
                ExtraFields::AIP1(service.extra().to_owned()),
            )
            .add_service_type(service.service_type().to_string())?
            .build()),
            ServiceSov::DIDCommV1(service) => Ok(Service::builder(
                service.id().clone(),
                service.service_endpoint().clone(),
                ExtraFields::DIDCommV1(service.extra().to_owned()),
            )
            .add_service_type(service.service_type().to_string())?
            .build()),
            ServiceSov::DIDCommV2(service) => Ok(Service::builder(
                service.id().clone(),
                service.service_endpoint().clone(),
                ExtraFields::DIDCommV2(service.extra().to_owned()),
            )
            .add_service_type(service.service_type().to_string())?
            .build()),
        }
    }
}
