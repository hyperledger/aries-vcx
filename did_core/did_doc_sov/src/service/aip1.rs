use std::collections::HashMap;

use did_doc::schema::{
    types::{uri::Uri, url::Url},
    utils::OneOrList,
};
use serde::{Deserialize, Serialize};

use super::ServiceType;
use crate::{error::DidDocumentSovError, extra_fields::{aip1::ExtraFieldsAIP1}, TypedService};


#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
pub struct ServiceAIP1 {
    #[serde(flatten)]
    inner: TypedService<ExtraFieldsAIP1>,
}

// impl ServiceAIP1 {
//     pub fn new(
//         id: Uri,
//         service_endpoint: Url,
//         extra: ExtraFieldsAIP1,
//     ) -> Result<Self, DidDocumentSovError> {
//         Ok(Self {
//             inner: TypedService::<ExtraFieldsAIP1> {
//                 id,
//                 service_type: OneOrList(ServiceType::AIP1.to_string()),
//                 service_endpoint,
//                 extra
//             }
//         })
//     }
//
//     pub fn id(&self) -> &Uri {
//         self.inner.id()
//     }
//
//     pub fn service_type(&self) -> ServiceType {
//         ServiceType::AIP1
//     }
//
//     pub fn service_endpoint(&self) -> Url {
//         self.inner.service_endpoint().clone()
//     }
//
//     pub fn extra(&self) -> &ExtraFieldsAIP1 {
//         self.inner.extra()
//     }
// }

// impl TryFrom<Service> for ServiceAIP1 {
//     type Error = DidDocumentSovError;
//
//     fn try_from(service: Service) -> Result<Self, Self::Error> {
//         match service.extra() {
//             ExtraFieldsSov::AIP1(extra) => Self::new(
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
// impl TryFrom<Service> for ServiceAIP1 {
//     type Error = DidDocumentSovError;
//
//     fn try_from(service: Service) -> Result<Self, Self::Error> {
//         let extra =
//             serde_json::from_value::<ExtraFieldsAIP1>(serde_json::to_value(service.extra())?)?;
//         Self::new(
//             service.id().clone(),
//             service.service_endpoint().clone(),
//             extra,
//         )
//     }
// }
//
// impl<'de> Deserialize<'de> for ServiceAIP1 {
//     fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
//     where
//         D: serde::Deserializer<'de>,
//     {
//         let service = Service::deserialize(deserializer)?;
//         match service.service_type() {
//             OneOrList::One(service_type) if *service_type == ServiceType::AIP1.to_string() => {}
//             OneOrList::List(service_types)
//                 if service_types.contains(&ServiceType::AIP1.to_string()) => {}
//             _ => {
//                 return Err(serde::de::Error::custom(
//                     "Extra fields don't match service type",
//                 ))
//             }
//         };
//         match service.extra() {
//             ExtraFieldsSov::AIP1(extra) => Ok(Self {
//                 service: Service::builder(
//                     service.id().clone(),
//                     service.service_endpoint().clone(),
//                     extra.clone(),
//                 )
//                 .add_service_type(ServiceType::AIP1.to_string())
//                 .map_err(serde::de::Error::custom)?
//                 .build(),
//             }),
//             _ => Err(serde::de::Error::custom(
//                 "Extra fields don't match service type",
//             )),
//         }
//     }
// }
