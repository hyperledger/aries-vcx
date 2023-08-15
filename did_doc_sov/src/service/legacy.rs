use did_doc::schema::types::url::Url;
use did_doc::schema::{service::Service, types::uri::Uri};
use serde::{Deserialize, Serialize};

use crate::error::DidDocumentSovError;
use crate::extra_fields::legacy::ExtraFieldsLegacy;
use crate::extra_fields::ExtraFieldsSov;

use super::ServiceType;

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct ServiceLegacy {
    #[serde(default)]
    id: Uri,
    #[serde(rename = "type")]
    service_type: ServiceType,
    service_endpoint: Url,
    #[serde(flatten)]
    extra: ExtraFieldsLegacy,
}

impl ServiceLegacy {
    pub fn new(id: Uri, service_endpoint: Url, extra: ExtraFieldsLegacy) -> Result<Self, DidDocumentSovError> {
        Ok(Self {
            id,
            service_type: ServiceType::Legacy,
            service_endpoint,
            extra,
        })
    }

    pub fn id(&self) -> &Uri {
        &self.id
    }

    pub fn service_type(&self) -> ServiceType {
        ServiceType::Legacy
    }

    pub fn service_endpoint(&self) -> &Url {
        &self.service_endpoint
    }

    pub fn extra(&self) -> &ExtraFieldsLegacy {
        &self.extra
    }
}

impl TryFrom<Service<ExtraFieldsSov>> for ServiceLegacy {
    type Error = DidDocumentSovError;

    fn try_from(service: Service<ExtraFieldsSov>) -> Result<Self, Self::Error> {
        match service.extra() {
            ExtraFieldsSov::Legacy(extra) => {
                Self::new(service.id().clone(), service.service_endpoint().clone(), extra.clone())
            }
            _ => Err(DidDocumentSovError::UnexpectedServiceType(
                service.service_type().to_string(),
            )),
        }
    }
}

impl TryFrom<ServiceLegacy> for Service<ExtraFieldsSov> {
    type Error = DidDocumentSovError;

    fn try_from(service: ServiceLegacy) -> Result<Self, Self::Error> {
        let extra = ExtraFieldsSov::Legacy(service.extra);
        Ok(Service::builder(service.id, service.service_endpoint, extra)
            .add_service_type(ServiceType::Legacy.to_string())
            .unwrap()
            .build())
    }
}
