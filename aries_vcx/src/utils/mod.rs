use aries_vcx_core::wallet::base_wallet::BaseWallet;
use diddoc_legacy::aries::diddoc::AriesDidDoc;
use did_doc::schema::verification_method::{VerificationMethod, VerificationMethodType};
use did_doc_sov::extra_fields::didcommv1::ExtraFieldsDidCommV1;
use did_doc_sov::extra_fields::KeyKind;
use did_doc_sov::service::didcommv1::ServiceDidCommV1;
use did_doc_sov::service::ServiceSov;
use did_doc_sov::DidDocumentSov;
use did_key::DidKey;
use did_parser::Did;
use diddoc_legacy::aries::service::AriesService;
use crate::errors::error::{AriesVcxError, AriesVcxErrorKind, VcxResult};

#[macro_use]
#[cfg(feature = "vdrtools")]
pub mod devsetup;

#[cfg(debug_assertions)]
#[macro_export]
macro_rules! secret {
    ($val:expr) => {{
        $val
    }};
}

#[cfg(not(debug_assertions))]
#[macro_export]
macro_rules! secret {
    ($val:expr) => {{
        "_"
    }};
}

#[cfg(test)]
macro_rules! map (
    { $($key:expr => $value:expr),+ } => {
        {
            let mut m = std::collections::HashMap::new();
            $(
                m.insert($key, $value);
            )+
            m
        }
     };
);

pub mod openssl;
pub mod qualifier;

#[macro_use]
pub mod encryption_envelope;
pub mod serialization;
pub mod validation;


// TODO: Get rid of this please!!!
pub fn from_did_doc_sov_to_legacy(ddo: DidDocumentSov) -> VcxResult<AriesDidDoc> {
    let mut new_ddo = AriesDidDoc::default();
    new_ddo.id = ddo.id().to_string();
    new_ddo.set_service_endpoint(
        ddo.service()
            .first()
            .ok_or_else(|| AriesVcxError::from_msg(AriesVcxErrorKind::InvalidState, "No service present in DDO"))?
            .service_endpoint()
            .clone()
            .into(),
    );
    let mut recipient_keys = vec![];
    for ka in ddo.resolved_key_agreement() {
        recipient_keys.push(ka.public_key()?.base58());
    }
    for service in ddo.service() {
        if let Ok(key_kinds) = service.extra().recipient_keys() {
            for key_kind in key_kinds {
                match key_kind {
                    KeyKind::DidKey(key) => {
                        recipient_keys.push(key.key().base58());
                    }
                    KeyKind::Reference(_) => {}
                    KeyKind::Value(_) => {}
                }
            }
        }
    }
    new_ddo.set_recipient_keys(recipient_keys);
    Ok(new_ddo)
}

pub fn from_legacy_did_doc_to_sov(ddo: AriesDidDoc) -> VcxResult<DidDocumentSov> {
    let did: Did = ddo.id.parse().unwrap_or_default();
    let vm = VerificationMethod::builder(
        did.clone().into(),
        did.clone(),
        VerificationMethodType::Ed25519VerificationKey2020,
    )
        .add_public_key_base58(
        ddo.recipient_keys()?
            .first()
            .ok_or_else(|| {
                AriesVcxError::from_msg(
                    AriesVcxErrorKind::InvalidState,
                    "No recipient in the DDO being converted",
                )
            })?
            .to_string(),
    )
        .build();
    let new_ddo = DidDocumentSov::builder(did.clone())
        .add_service(from_legacy_service_to_service_sov(
            ddo.service
                .first()
                .ok_or_else(|| {
                    AriesVcxError::from_msg(AriesVcxErrorKind::InvalidState, "No service in the DDO being converted")
                })?
                .clone(),
        )?)
        .add_controller(did.clone())
        .add_verification_method(vm)
        .build();
    Ok(new_ddo)
}

pub fn from_legacy_service_to_service_sov(service: AriesService) -> VcxResult<ServiceSov> {
    let extra = ExtraFieldsDidCommV1::builder()
        .set_recipient_keys(
            service
                .recipient_keys
                .iter()
                .map(String::to_owned)
                .map(|s| -> VcxResult<KeyKind> {
                    if s.starts_with("did:key:") {
                        Ok(KeyKind::DidKey(DidKey::parse(s)?))
                    } else {
                        Ok(KeyKind::Value(s))
                    }
                })
                .collect::<VcxResult<Vec<_>>>()?,
        )
        .set_routing_keys(
            service
                .routing_keys
                .iter()
                .map(String::to_owned)
                .map(|s| -> VcxResult<KeyKind> {
                    if s.starts_with("did:key:") {
                        Ok(KeyKind::DidKey(DidKey::parse(s)?))
                    } else {
                        Ok(KeyKind::Value(s))
                    }
                })
                .collect::<VcxResult<Vec<_>>>()?,
        )
        .build();
    Ok(ServiceSov::DIDCommV1(ServiceDidCommV1::new(
        // TODO: Why was this necessary? Double-check
        service.id.parse().unwrap_or_default(),
        service.service_endpoint.into(),
        extra,
    )?))
}

pub fn from_service_sov_to_legacy(service: ServiceSov) -> AriesService {
    match service {
        ServiceSov::AIP1(service) => AriesService {
            id: service.id().to_string(),
            service_endpoint: service.service_endpoint().clone().into(),
            ..Default::default()
        },
        ServiceSov::DIDCommV1(service) => {
            let extra = service.extra();
            let recipient_keys = extra.recipient_keys().iter().map(|key| key.to_string()).collect();
            let routing_keys = extra.routing_keys().iter().map(|key| key.to_string()).collect();
            AriesService {
                id: service.id().to_string(),
                recipient_keys,
                routing_keys,
                service_endpoint: service.service_endpoint().clone().into(),
                ..Default::default()
            }
        }
        ServiceSov::DIDCommV2(service) => {
            let extra = service.extra();
            let routing_keys = extra.routing_keys().iter().map(|key| key.to_string()).collect();
            AriesService {
                id: service.id().to_string(),
                routing_keys,
                service_endpoint: service.service_endpoint().clone().into(),
                ..Default::default()
            }
        }
        ServiceSov::Legacy(_) => todo!(),
    }
}
