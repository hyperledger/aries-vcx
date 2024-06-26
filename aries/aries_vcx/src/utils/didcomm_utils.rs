use did_doc::schema::{
    did_doc::DidDocument, service::service_key_kind::ServiceKeyKind, types::uri::Uri,
    verification_method::VerificationMethodType,
};
use public_key::{Key, KeyType};

use crate::errors::error::{AriesVcxError, AriesVcxErrorKind, VcxResult};

pub(crate) fn resolve_service_key_to_typed_key(
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

/// Resolves the first ed25519 base58 public key (a.k.a. verkey) within the DIDDocuments key
/// agreement keys. Useful for resolving keys that can be used for packing DIDCommV1 messages.
pub fn resolve_ed25519_base58_key_agreement(did_document: &DidDocument) -> VcxResult<String> {
    let vm_types = [
        VerificationMethodType::Ed25519VerificationKey2018,
        VerificationMethodType::Ed25519VerificationKey2020,
        VerificationMethodType::X25519KeyAgreementKey2019,
        VerificationMethodType::X25519KeyAgreementKey2020,
        VerificationMethodType::Multikey,
        // would be nice to search for X25519 VM types which could be derived into ed25519 keys
        // for the encryption envelope to use.
        // would be nice to search for other VM types which _could_ be ed25519 (jwk etc)
    ];
    let vm = did_document.get_key_agreement_of_type(&vm_types)?;
    let key = vm.public_key()?;

    match key.key_type() {
        KeyType::Ed25519 => {}
        _ => {
            return Err(AriesVcxError::from_msg(
                AriesVcxErrorKind::InvalidVerkey,
                format!("Cannot resolve key agreement as an Ed25519 key: {vm:?}"),
            ))
        }
    }

    Ok(vm.public_key()?.base58())
}

pub fn get_ed25519_base58_routing_keys(
    their_did_doc: &DidDocument,
    service_id: &Uri,
) -> VcxResult<Vec<String>> {
    let service = their_did_doc.get_service_by_id(service_id)?;
    let Ok(routing_keys) = service.extra_field_routing_keys() else {
        return Ok(vec![]);
    };

    let mut naked_routing_keys = Vec::new();

    for key in routing_keys.iter() {
        let pub_key = resolve_service_key_to_typed_key(key, their_did_doc)?;

        if pub_key.key_type() == &KeyType::Ed25519 {
            naked_routing_keys.push(pub_key.base58());
        }
    }

    Ok(naked_routing_keys)
}

pub fn get_ed25519_base58_recipient_keys(
    their_did_doc: &DidDocument,
    service_id: &Uri,
) -> VcxResult<Vec<String>> {
    let service = their_did_doc.get_service_by_id(service_id)?;
    let Ok(recipient_keys) = service.extra_field_recipient_keys() else {
        return Ok(vec![]);
    };

    let mut naked_recipient_keys = Vec::new();

    for key in recipient_keys.iter() {
        let pub_key = resolve_service_key_to_typed_key(key, their_did_doc)?;
        if pub_key.key_type() == &KeyType::Ed25519 {
            naked_recipient_keys.push(pub_key.base58());
        }
    }

    Ok(naked_recipient_keys)
}
