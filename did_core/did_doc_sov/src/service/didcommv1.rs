use std::collections::HashMap;

use did_doc::schema::{
    service::Service,
    types::{uri::Uri, url::Url},
    utils::OneOrList,
};
use serde::{Deserialize, Serialize};
use serde_json::Value;

use super::ServiceType;
use crate::{error::DidDocumentSovError, extra_fields::{didcommv1::ExtraFieldsDidCommV1, ExtraFieldsSov}, TypedService};

#[derive(Serialize, Clone, Debug, PartialEq)]
pub struct ServiceDidCommV1 {
    #[serde(flatten)]
    service: TypedService<ExtraFieldsDidCommV1>,
}

impl ServiceDidCommV1 {
    pub fn new(
        id: Uri,
        service_endpoint: Url,
        extra: ExtraFieldsDidCommV1,
    ) -> Self {
        Self {
            service: TypedService::<ExtraFieldsDidCommV1> {
                id,
                service_type: OneOrList::One(ServiceType::DIDCommV1.to_string()),
                service_endpoint,
                extra
            }
        }
        // Ok(Self {
        //     service: Service::builder(id, service_endpoint, extra)
        //         .add_service_type(ServiceType::DIDCommV1.to_string())?
        //         .build(),
        // })
    }

    pub fn id(&self) -> &Uri {
        self.service.id()
    }

    pub fn service_type(&self) -> ServiceType {
        ServiceType::DIDCommV1
    }

    pub fn service_endpoint(&self) -> Url {
        self.service.service_endpoint().clone()
    }

    pub fn extra(&self) -> &ExtraFieldsDidCommV1 {
        self.service.extra()
    }
}


impl TryFrom<ServiceDidCommV1> for Service {
    type Error = DidDocumentSovError;

    fn try_from(did_comm_service: ServiceDidCommV1) -> Result<Self, Self::Error> {
        let mut extra_fields = HashMap::new();
        extra_fields.insert("priority".to_string(),  serde_json::Value::from(did_comm_service.extra().priority()));
        extra_fields.insert("recipientKeys".to_string(),  serde_json::to_value(did_comm_service.extra().recipient_keys())?);
        extra_fields.insert("routingKeys".to_string(),  serde_json::to_value(did_comm_service.extra().routing_keys())?);
        extra_fields.insert("accept".to_string(),  serde_json::to_value(did_comm_service.extra().accept())?);

        Ok(Service::builder(
            did_comm_service.id().clone(),
            did_comm_service.service_endpoint().clone(),
            extra_fields
        ).build())
    }
}

// impl TryFrom<Service> for ServiceDidCommV1 {
//     type Error = DidDocumentSovError;
//
//     fn try_from(service: Service) -> Result<Self, Self::Error> {
//         match service.extra() {
//             ExtraFieldsSov::DIDCommV1(extra) => Self::new(
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
// impl TryFrom<Service<HashMap<String, Value>>> for ServiceDidCommV1 {
//     type Error = DidDocumentSovError;
//
//     fn try_from(service: Service<HashMap<String, Value>>) -> Result<Self, Self::Error> {
//         let extra =
//             serde_json::from_value::<ExtraFieldsDidCommV1>(serde_json::to_value(service.extra())?)?;
//         Self::new(
//             service.id().clone(),
//             service.service_endpoint().clone(),
//             extra,
//         )
//     }
// }
//
// impl<'de> Deserialize<'de> for ServiceDidCommV1 {
//     fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
//     where
//         D: serde::Deserializer<'de>,
//     {
//         let service = Service::deserialize(deserializer)?;
//         match service.service_type() {
//             OneOrList::One(service_type) if *service_type == ServiceType::DIDCommV1.to_string() => {
//             }
//             OneOrList::List(service_types)
//                 if service_types.contains(&ServiceType::DIDCommV1.to_string()) => {}
//             _ => {
//                 return Err(serde::de::Error::custom(
//                     "Extra fields don't match service type",
//                 ))
//             }
//         };
//         match service.extra() {
//             ExtraFieldsSov::DIDCommV1(extra) => Ok(Self {
//                 service: Service::builder(
//                     service.id().clone(),
//                     service.service_endpoint().clone(),
//                     extra.clone(),
//                 )
//                 .add_service_type(ServiceType::DIDCommV1.to_string())
//                 .map_err(serde::de::Error::custom)?
//                 .build(),
//             }),
//             _ => Err(serde::de::Error::custom(
//                 "Extra fields don't match service type",
//             )),
//         }
//     }
// }
