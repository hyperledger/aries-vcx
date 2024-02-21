use did_doc::schema::{
    did_doc::DidDocument, service::service_key_kind::ServiceKeyKind, types::uri::Uri,
    verification_method::VerificationMethodType,
};
use did_peer::{
    peer_did::{numalgos::numalgo2::Numalgo2, PeerDid},
    resolver::{options::PublicKeyEncoding, PeerDidResolutionOptions, PeerDidResolver},
};
use did_resolver::{
    error::GenericError,
    traits::resolvable::{resolution_output::DidResolutionOutput, DidResolvable},
};
use public_key::Key;

use crate::errors::error::{AriesVcxError, AriesVcxErrorKind, VcxResult};

pub(crate) async fn resolve_didpeer2(
    did_peer: &PeerDid<Numalgo2>,
    encoding: PublicKeyEncoding,
) -> Result<DidDocument, GenericError> {
    let DidResolutionOutput { did_document, .. } = PeerDidResolver::new()
        .resolve(
            did_peer.did(),
            &PeerDidResolutionOptions {
                encoding: Some(encoding),
            },
        )
        .await?;
    Ok(did_document)
}

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
    let key_base58 = did_document.get_key_agreement_of_type(&key_types)?;
    Ok(key_base58.public_key()?.base58())
}

pub fn get_routing_keys(their_did_doc: &DidDocument, service_id: &Uri) -> VcxResult<Vec<String>> {
    let service = their_did_doc.get_service_by_id(service_id)?;
    match service.extra_field_routing_keys() {
        Ok(routing_keys) => {
            let mut naked_routing_keys = Vec::new();
            for key in routing_keys.iter() {
                naked_routing_keys
                    .push(resolve_service_key_to_typed_key(key, their_did_doc)?.base58());
            }
            Ok(naked_routing_keys)
        }
        Err(_err) => Ok(Vec::new()),
    }
}
