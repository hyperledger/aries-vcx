use did_doc::schema::{
    service::Service,
    types::{uri::Uri, url::Url},
};
use serde::{Deserialize, Serialize};

use super::ServiceType;
use crate::{error::DidDocumentSovError, extra_fields::{legacy::ExtraFieldsLegacy, ExtraFieldsSov}, TypedService};

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct ServiceLegacy {
    #[serde(flatten)]
    service: TypedService<ExtraFieldsLegacy>
}

// impl ServiceLegacy {
//     pub fn new(
//         id: Uri,
//         service_endpoint: Url,
//         extra: ExtraFieldsLegacy,
//     ) -> Result<Self, DidDocumentSovError> {
//         Ok(Self {
//             id,
//             service_type: ServiceType::Legacy,
//             service_endpoint,
//             extra,
//         })
//     }
//
//     pub fn id(&self) -> &Uri {
//         &self.id
//     }
//
//     pub fn service_type(&self) -> ServiceType {
//         ServiceType::Legacy
//     }
//
//     pub fn service_endpoint(&self) -> Url {
//         self.service_endpoint.clone()
//     }
//
//     pub fn extra(&self) -> &ExtraFieldsLegacy {
//         &self.extra
//     }
// }
//
// impl TryFrom<Service> for ServiceLegacy {
//     type Error = DidDocumentSovError;
//
//     fn try_from(service: Service) -> Result<Self, Self::Error> {
//         match service.extra() {
//             ExtraFieldsSov::Legacy(extra) => Self::new(
//                 service.id().clone(),
//                 service.service_endpoint().clone(),
//                 extra.clone(),
//             ),
//             _ => Err(DidDocumentSovError::UnexpectedServiceType(
//                 service.service_type().to_string(),
//             )),
//         }
//     }
// }
//
// impl TryFrom<ServiceLegacy> for Service {
//     type Error = DidDocumentSovError;
//
//     fn try_from(service: ServiceLegacy) -> Result<Self, Self::Error> {
//         let extra = ExtraFieldsSov::Legacy(service.extra);
//         Ok(
//             Service::builder(service.id, service.service_endpoint, extra)
//                 .add_service_type(ServiceType::Legacy.to_string())?
//                 .build(),
//         )
//     }
// }
