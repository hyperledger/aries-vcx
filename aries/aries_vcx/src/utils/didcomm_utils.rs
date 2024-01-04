use did_doc::schema::{
    did_doc::DidDocument, service::extra_fields::ServiceKeyKind,
    verification_method::VerificationMethodType,
};
use public_key::Key;

use crate::errors::error::{AriesVcxError, AriesVcxErrorKind, VcxResult};

fn resolve_service_key_to_typed_key(
    key: &ServiceKeyKind,
    did_document: &DidDocument,
) -> VcxResult<Key> {
    match key {
        ServiceKeyKind::DidKey(did_key) => Ok(did_key.key().clone()),
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
            Ok(key)
        }
        ServiceKeyKind::Value(value) => Ok(Key::new(
            value.as_bytes().to_vec(),
            public_key::KeyType::Ed25519,
        )?),
    }
}

pub fn resolve_base58_key_agreement(did_document: &DidDocument) -> VcxResult<String> {
    let key_types = [
        VerificationMethodType::Ed25519VerificationKey2018,
        VerificationMethodType::Ed25519VerificationKey2020,
        VerificationMethodType::X25519KeyAgreementKey2019,
        VerificationMethodType::X25519KeyAgreementKey2020,
    ];
    let key_base58 = did_document.find_key_agreement_of_type(&key_types)?;
    Ok(key_base58.public_key()?.base58())
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
    match service.extra_field_routing_keys() {
        Ok(routing_keys) => {
            let mut naked_routing_keys = Vec::new();
            for key in routing_keys.iter() {
                naked_routing_keys
                    .push(resolve_service_key_to_typed_key(key, our_did_doc)?.base58());
            }
            Ok(naked_routing_keys)
        }
        Err(_err) => Ok(Vec::new()),
    }
}
