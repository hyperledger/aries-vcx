use std::{collections::HashMap, fmt::Display};

use did_doc::schema::{
    service::Service,
    types::{uri::Uri, url::Url},
};
use serde::{Deserialize, Serialize};
use serde_json::Value;

use crate::{error::DidDocumentSovError, extra_fields::ExtraFieldsSov};

pub mod aip1;
pub mod didcommv1;
pub mod didcommv2;
pub mod legacy;

#[derive(Serialize, Deserialize, Clone, Copy, Debug, PartialEq)]
pub enum ServiceType {
    #[serde(rename = "endpoint")]
    AIP1,
    #[serde(rename = "did-communication")]
    DIDCommV1,
    #[serde(rename = "DIDComm")]
    DIDCommV2,
    #[serde(rename = "IndyAgent")]
    Legacy,
}

impl Display for ServiceType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ServiceType::AIP1 => write!(f, "endpoint"),
            ServiceType::DIDCommV1 => write!(f, "did-communication"),
            ServiceType::DIDCommV2 => write!(f, "DIDComm"),
            ServiceType::Legacy => write!(f, "IndyAgent"),
        }
    }
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
#[serde(untagged)]
pub enum ServiceSov {
    Legacy(legacy::ServiceLegacy),
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
            ServiceSov::Legacy(service) => service.id(),
        }
    }

    pub fn service_type(&self) -> ServiceType {
        match self {
            ServiceSov::AIP1(service) => service.service_type(),
            ServiceSov::DIDCommV1(service) => service.service_type(),
            ServiceSov::DIDCommV2(service) => service.service_type(),
            ServiceSov::Legacy(service) => service.service_type(),
        }
    }

    pub fn service_endpoint(&self) -> &Url {
        match self {
            ServiceSov::AIP1(service) => service.service_endpoint(),
            ServiceSov::DIDCommV1(service) => service.service_endpoint(),
            ServiceSov::DIDCommV2(service) => service.service_endpoint(),
            ServiceSov::Legacy(service) => service.service_endpoint(),
        }
    }

    pub fn extra(&self) -> ExtraFieldsSov {
        match self {
            ServiceSov::AIP1(service) => ExtraFieldsSov::AIP1(service.extra().to_owned()),
            ServiceSov::DIDCommV1(service) => ExtraFieldsSov::DIDCommV1(service.extra().to_owned()),
            ServiceSov::DIDCommV2(service) => ExtraFieldsSov::DIDCommV2(service.extra().to_owned()),
            ServiceSov::Legacy(service) => ExtraFieldsSov::Legacy(service.extra().to_owned()),
        }
    }
}

impl TryFrom<Service<ExtraFieldsSov>> for ServiceSov {
    type Error = DidDocumentSovError;

    fn try_from(service: Service<ExtraFieldsSov>) -> Result<Self, Self::Error> {
        match service.extra() {
            ExtraFieldsSov::AIP1(_extra) => Ok(ServiceSov::AIP1(service.try_into()?)),
            ExtraFieldsSov::DIDCommV1(_extra) => Ok(ServiceSov::DIDCommV1(service.try_into()?)),
            ExtraFieldsSov::DIDCommV2(_extra) => Ok(ServiceSov::DIDCommV2(service.try_into()?)),
            ExtraFieldsSov::Legacy(_extra) => Ok(ServiceSov::Legacy(service.try_into()?)),
        }
    }
}

impl TryFrom<Service<HashMap<String, Value>>> for ServiceSov {
    type Error = DidDocumentSovError;

    fn try_from(service: Service<HashMap<String, Value>>) -> Result<Self, Self::Error> {
        match service.extra().get("type") {
            Some(service_type) => match service_type.as_str() {
                Some("AIP1") => Ok(ServiceSov::AIP1(service.try_into()?)),
                Some("DIDCommV1") => Ok(ServiceSov::DIDCommV1(service.try_into()?)),
                Some("DIDCommV2") => Ok(ServiceSov::DIDCommV2(service.try_into()?)),
                _ => Err(DidDocumentSovError::UnexpectedServiceType(service_type.to_string())),
            },
            None => Ok(ServiceSov::AIP1(service.try_into()?)),
        }
    }
}

impl TryFrom<ServiceSov> for Service<ExtraFieldsSov> {
    type Error = DidDocumentSovError;

    fn try_from(service: ServiceSov) -> Result<Self, Self::Error> {
        match service {
            ServiceSov::AIP1(service) => Ok(Service::builder(
                service.id().clone(),
                service.service_endpoint().clone(),
                ExtraFieldsSov::AIP1(service.extra().to_owned()),
            )
            .add_service_type(service.service_type().to_string())?
            .build()),
            ServiceSov::DIDCommV1(service) => Ok(Service::builder(
                service.id().clone(),
                service.service_endpoint().clone(),
                ExtraFieldsSov::DIDCommV1(service.extra().to_owned()),
            )
            .add_service_type(service.service_type().to_string())?
            .build()),
            ServiceSov::DIDCommV2(service) => Ok(Service::builder(
                service.id().clone(),
                service.service_endpoint().clone(),
                ExtraFieldsSov::DIDCommV2(service.extra().to_owned()),
            )
            .add_service_type(service.service_type().to_string())?
            .build()),
            ServiceSov::Legacy(service) => Ok(Service::builder(
                service.id().clone(),
                service.service_endpoint().clone(),
                ExtraFieldsSov::Legacy(service.extra().to_owned()),
            )
            .add_service_type(service.service_type().to_string())?
            .build()),
        }
    }
}
