use std::collections::HashMap;

use did_doc::schema::{
    service::Service,
    types::{uri::Uri, url::Url},
    utils::OneOrList,
};
use serde::{Deserialize, Serialize};
use serde_json::Value;

use super::ServiceType;
use crate::{
    error::DidDocumentSovError,
    extra_fields::{didcommv2::ExtraFieldsDidCommV2, ExtraFieldsSov},
};

#[derive(Serialize, Clone, Debug, PartialEq)]
pub struct ServiceDidCommV2 {
    #[serde(flatten)]
    service: Service<ExtraFieldsDidCommV2>,
}

impl ServiceDidCommV2 {
    pub fn new(
        id: Uri,
        service_endpoint: Url,
        extra: ExtraFieldsDidCommV2,
    ) -> Result<Self, DidDocumentSovError> {
        Ok(Self {
            service: Service::builder(id, service_endpoint, extra)
                .add_service_type(ServiceType::DIDCommV2.to_string())?
                .build(),
        })
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

impl TryFrom<Service<ExtraFieldsSov>> for ServiceDidCommV2 {
    type Error = DidDocumentSovError;

    fn try_from(service: Service<ExtraFieldsSov>) -> Result<Self, Self::Error> {
        match service.extra() {
            ExtraFieldsSov::DIDCommV2(extra) => Self::new(
                service.id().clone(),
                service.service_endpoint().clone(),
                extra.clone(),
            ),
            _ => Err(DidDocumentSovError::UnexpectedServiceType(
                service.service_type().to_string(),
            )),
        }
    }
}

impl TryFrom<Service<HashMap<String, Value>>> for ServiceDidCommV2 {
    type Error = DidDocumentSovError;

    fn try_from(service: Service<HashMap<String, Value>>) -> Result<Self, Self::Error> {
        let extra =
            serde_json::from_value::<ExtraFieldsDidCommV2>(serde_json::to_value(service.extra())?)?;
        Self::new(
            service.id().clone(),
            service.service_endpoint().clone(),
            extra,
        )
    }
}

impl<'de> Deserialize<'de> for ServiceDidCommV2 {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let service = Service::<ExtraFieldsSov>::deserialize(deserializer)?;
        match service.service_type() {
            OneOrList::One(service_type) if *service_type == ServiceType::DIDCommV2.to_string() => {
            }
            OneOrList::List(service_types)
                if service_types.contains(&ServiceType::DIDCommV2.to_string()) => {}
            _ => {
                return Err(serde::de::Error::custom(
                    "Extra fields don't match service type",
                ))
            }
        };
        match service.extra() {
            ExtraFieldsSov::DIDCommV2(extra) => Ok(Self {
                service: Service::builder(
                    service.id().clone(),
                    service.service_endpoint().clone(),
                    extra.clone(),
                )
                .add_service_type(ServiceType::DIDCommV2.to_string())
                .map_err(serde::de::Error::custom)?
                .build(),
            }),
            _ => Err(serde::de::Error::custom(
                "Extra fields don't match service type",
            )),
        }
    }
}
