use did_doc::schema::{did_doc::DidDocument, service::Service};
use did_resolver_sov::resolution::ExtraFieldsSov;

use crate::errors::error::{DiddocError, DiddocErrorKind};

use self::diddoc::AriesDidDoc;

pub mod diddoc;
pub mod service;

impl TryFrom<AriesDidDoc> for DidDocument<ExtraFieldsSov> {
    type Error = DiddocError;

    fn try_from(ddo_legacy: AriesDidDoc) -> Result<Self, Self::Error> {
        let service_legacy = ddo_legacy.get_service()?;
        let extra = ExtraFieldsSov::builder()
            .set_routing_keys(service_legacy.routing_keys)
            .set_recipient_keys(service_legacy.recipient_keys)
            .build();
        let service = Service::<ExtraFieldsSov>::builder(
            ddo_legacy.id.parse()?,
            ddo_legacy
                .get_endpoint()
                .ok_or(DiddocError::from_msg(
                    DiddocErrorKind::ConversionError,
                    "No service found",
                ))?
                .into(),
        )?
        .add_extra(extra)
        .add_service_type("did-communication".into())?
        .build()?;
        Ok(DidDocument::builder(ddo_legacy.id.try_into().unwrap_or_default())
            .add_service(service)
            .build())
    }
}

impl TryFrom<DidDocument<ExtraFieldsSov>> for AriesDidDoc {
    type Error = DiddocError;

    fn try_from(ddo: DidDocument<ExtraFieldsSov>) -> Result<Self, Self::Error> {
        let service = ddo.service().get(0).ok_or(DiddocError::from_msg(
            DiddocErrorKind::ConversionError,
            "No service found",
        ))?;
        let mut ddo_legacy = AriesDidDoc::default();
        ddo_legacy.service = ddo.service().into_iter().map(|s| s.to_owned().into()).collect();
        ddo_legacy.set_id(ddo.id().to_string());
        ddo_legacy.set_service_endpoint(service.service_endpoint().to_owned().into());
        ddo_legacy.set_recipient_keys(service.extra().recipient_keys().to_vec());
        ddo_legacy.set_routing_keys(service.extra().routing_keys().to_vec());
        Ok(ddo_legacy)
    }
}
