use did_doc::schema::{did_doc::DidDocument, service::extra_fields::ServiceKeyKind};

use crate::errors::error::{AriesVcxError, AriesVcxErrorKind, VcxResult};

fn service_key_to_naked_key(key: &ServiceKeyKind, did_document: &DidDocument) -> VcxResult<String> {
    match key {
        ServiceKeyKind::DidKey(did_key) => Ok(did_key.key().base58()),
        ServiceKeyKind::Reference(reference) => {
            let verification_method = did_document.dereference_key(reference).ok_or_else(|| {
                AriesVcxError::from_msg(
                    AriesVcxErrorKind::InvalidState,
                    format!("Unable to dereference key: {}", reference),
                )
            })?;
            let key = verification_method.public_key().map_err(|err| {
                AriesVcxError::from_msg(
                    AriesVcxErrorKind::InvalidState,
                    format!("Unable to get public key from verification method: {}", err),
                )
            })?;
            Ok(key.base58())
        }
        ServiceKeyKind::Value(value) => Ok(String::from(value)),
    }
}

pub fn get_sender_verkey(did_document: &DidDocument) -> VcxResult<String> {
    let service = did_document
        .service()
        .first()
        .ok_or_else(|| {
            AriesVcxError::from_msg(
                AriesVcxErrorKind::InvalidState,
                "No Service object found on our did document",
            )
        })?
        .clone();
    let sender_vk = service
        .extra_field_recipient_keys()
        .map_err(|err| {
            AriesVcxError::from_msg(
                AriesVcxErrorKind::InvalidState,
                format!(
                    "Recipient key field found in our did document but had unexpected format, \
                     err: {err:?}"
                ),
            )
        })?
        .first()
        .ok_or_else(|| {
            AriesVcxError::from_msg(
                AriesVcxErrorKind::InvalidState,
                "Recipient key field but did not have any keys",
            )
        })?
        .clone();
    let naked_sender_vk = service_key_to_naked_key(&sender_vk, did_document)?;
    Ok(naked_sender_vk)
}

pub fn get_routing_keys(our_did_doc: &DidDocument) -> VcxResult<Vec<String>> {
    let service = our_did_doc
        .service()
        .first()
        .ok_or_else(|| {
            AriesVcxError::from_msg(
                AriesVcxErrorKind::InvalidState,
                "No Service object found on our did document",
            )
        })?
        .clone();
    let routing_keys = service.extra_field_routing_keys().map_err(|err| {
        AriesVcxError::from_msg(
            AriesVcxErrorKind::InvalidState,
            format!("No routing_keys found: {}", err),
        )
    })?;
    let mut naked_routing_keys = Vec::new();
    for key in routing_keys.iter() {
        naked_routing_keys.push(service_key_to_naked_key(key, our_did_doc)?);
    }
    Ok(naked_routing_keys)
}
