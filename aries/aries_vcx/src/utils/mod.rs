use did_doc::schema::verification_method::{VerificationMethod, VerificationMethodType};
use did_doc_sov::{
    extra_fields::{didcommv1::ExtraFieldsDidCommV1, KeyKind},
    service::{didcommv1::ServiceDidCommV1, ServiceSov},
    DidDocumentSov,
};
use did_key::DidKey;
use did_parser::Did;
use diddoc_legacy::aries::{diddoc::AriesDidDoc, service::AriesService};

use crate::errors::error::{AriesVcxError, AriesVcxErrorKind, VcxResult};

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

pub mod base64;
pub mod openssl;
pub mod qualifier;

#[macro_use]
pub mod encryption_envelope;
pub mod serialization;
pub mod validation;

// TODO: Get rid of this, migrate off the legacy diddoc
pub fn from_did_doc_sov_to_legacy(ddo: DidDocumentSov) -> VcxResult<AriesDidDoc> {
    let mut new_ddo = AriesDidDoc {
        id: ddo.id().to_string(),
        ..Default::default()
    };
    new_ddo.set_service_endpoint(
        ddo.service()
            .first()
            .ok_or_else(|| {
                AriesVcxError::from_msg(
                    AriesVcxErrorKind::InvalidState,
                    "No service present in DDO",
                )
            })?
            .service_endpoint()
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
                    AriesVcxError::from_msg(
                        AriesVcxErrorKind::InvalidState,
                        "No service in the DDO being converted",
                    )
                })?
                .clone(),
        )?)
        .add_controller(did)
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
    info!(
        "Converting AnyService to expanded AriesService: {:?}",
        service
    );
    match service {
        ServiceSov::AIP1(service) => AriesService {
            id: service.id().to_string(),
            service_endpoint: service.service_endpoint().into(),
            ..Default::default()
        },
        ServiceSov::DIDCommV1(service) => {
            let extra = service.extra();
            let recipient_keys = extra
                .recipient_keys()
                .iter()
                .map(|key| key.to_string())
                .collect();
            let routing_keys = extra
                .routing_keys()
                .iter()
                .map(|key| key.to_string())
                .collect();
            AriesService {
                id: service.id().to_string(),
                recipient_keys,
                routing_keys,
                service_endpoint: service.service_endpoint().into(),
                ..Default::default()
            }
        }
        ServiceSov::DIDCommV2(service) => {
            let extra = service.extra();
            let routing_keys = extra
                .routing_keys()
                .iter()
                .map(|key| key.to_string())
                .collect();
            AriesService {
                id: service.id().to_string(),
                routing_keys,
                service_endpoint: service.service_endpoint().into(),
                ..Default::default()
            }
        }
        ServiceSov::Legacy(service) => AriesService {
            id: service.id().to_string(),
            recipient_keys: service
                .extra()
                .recipient_keys()
                .iter()
                .map(|key| key.to_string())
                .collect(),
            routing_keys: service
                .extra()
                .routing_keys()
                .iter()
                .map(|key| key.to_string())
                .collect(),
            service_endpoint: service.service_endpoint().into(),
            ..Default::default()
        },
    }
}
