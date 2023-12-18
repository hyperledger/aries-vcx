use did_doc::schema::{
    did_doc::{diddoc_resolve_first_key_agreement, DidDocument},
    service::extra_fields::ServiceKeyKind,
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
    // note: we possibly don't want to support this, instead rely on key_agreement field
    // let service = did_document
    //     .service()
    //     .first()
    //     .ok_or_else(|| {
    //         AriesVcxError::from_msg(
    //             AriesVcxErrorKind::InvalidState,
    //             "No Service object found on our did document",
    //         )
    //     })?
    //     .clone();
    // let key_base58 = match service.extra_field_recipient_keys() {
    //     Ok(recipient_keys) => {
    //         match recipient_keys.first() {
    //             None => {
    //                 return Err(AriesVcxError::from_msg(
    //                     AriesVcxErrorKind::InvalidState,
    //                     "Recipient key field but did not have any keys",
    //                 ))
    //             }
    //             Some(key) => {
    //                 // service_key_to_naked_key(&key, did_document)?
    //                 unimplemented!("Support for 'recipientKeys' has been dropped")
    //             }
    //         }
    //     }
    //     Err(_err) => {
    //
    //     }
    // };
    let key_base58 = diddoc_resolve_first_key_agreement(did_document)?;
    Ok(key_base58.base58())
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
