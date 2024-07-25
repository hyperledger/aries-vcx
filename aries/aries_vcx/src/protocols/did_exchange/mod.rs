use std::sync::Arc;

use did_doc::schema::{
    did_doc::DidDocument,
    verification_method::{VerificationMethod, VerificationMethodKind},
};
use did_parser_nom::DidUrl;
use did_resolver_registry::ResolverRegistry;
use messages::msg_fields::protocols::out_of_band::invitation::{
    Invitation as OobInvitation, OobService,
};
use public_key::Key;

use crate::{
    errors::error::{AriesVcxError, AriesVcxErrorKind, VcxResult},
    utils::didcomm_utils::resolve_service_key_to_typed_key,
};

pub mod state_machine;
pub mod states;
pub mod transition;

fn resolve_verification_method(
    did_doc: &DidDocument,
    verification_method_ref: &DidUrl,
) -> Result<VerificationMethod, AriesVcxError> {
    let key = did_doc.verification_method().iter().find(|key_agreement| {
        let reference_fragment = match verification_method_ref.fragment() {
            None => {
                warn!(
                    "Fragment not found in verification method reference {}",
                    verification_method_ref
                );
                return false;
            }
            Some(fragment) => fragment,
        };
        let key_agreement_fragment = match key_agreement.id().fragment() {
            None => {
                warn!(
                    "Fragment not found in verification method {}",
                    key_agreement.id()
                );
                return false;
            }
            Some(fragment) => fragment,
        };
        reference_fragment == key_agreement_fragment
    });
    match key {
        None => Err(AriesVcxError::from_msg(
            AriesVcxErrorKind::InvalidState,
            format!(
                "Verification method not found in resolved did document {}",
                did_doc
            ),
        )),
        Some(verification_method) => Ok(verification_method.clone()),
    }
}

fn resolve_first_key_agreement(did_document: &DidDocument) -> VcxResult<VerificationMethod> {
    // todo: did_document needs robust way to resolve this, I shouldn't care if there's reference or
    // actual key in the key_agreement       Abstract the user from format/structure of the did
    // document
    let verification_method_kind = did_document.key_agreement().first().ok_or_else(|| {
        AriesVcxError::from_msg(
            AriesVcxErrorKind::InvalidState,
            format!(
                "No verification method found in resolved did document {}",
                did_document
            ),
        )
    })?;
    let verification_method = match verification_method_kind {
        VerificationMethodKind::Resolved(verification_method) => verification_method.clone(),
        VerificationMethodKind::Resolvable(verification_method_ref) => {
            resolve_verification_method(did_document, verification_method_ref)?
        }
    };
    Ok(verification_method)
}

pub async fn resolve_enc_key_from_invitation(
    invitation: &OobInvitation,
    resolver_registry: &Arc<ResolverRegistry>,
) -> Result<Key, AriesVcxError> {
    match invitation.content.services.first().ok_or_else(|| {
        AriesVcxError::from_msg(
            AriesVcxErrorKind::InvalidInput,
            "Invitation does not contain any services",
        )
    })? {
        OobService::Did(did) => {
            info!("Invitation contains service (DID format): {}", did);
            let output = resolver_registry
                .resolve(&did.clone().try_into()?, &Default::default())
                .await
                .map_err(|err| {
                    AriesVcxError::from_msg(
                        AriesVcxErrorKind::InvalidDid,
                        format!("DID resolution failed: {err}"),
                    )
                })?;
            info!(
                "resolve_enc_key_from_invitation >> Resolved did document {}",
                output.did_document
            );
            let did_doc = output.did_document;
            resolve_enc_key_from_did_doc(&did_doc)
        }
        OobService::AriesService(_service) => {
            unimplemented!("Embedded Aries Service not yet supported by did-exchange")
        }
    }
}

pub async fn resolve_enc_key_from_did(
    did: &str,
    resolver_registry: &Arc<ResolverRegistry>,
) -> Result<Key, AriesVcxError> {
    let output = resolver_registry
        .resolve(&did.try_into()?, &Default::default())
        .await
        .map_err(|err| {
            AriesVcxError::from_msg(
                AriesVcxErrorKind::InvalidDid,
                format!("DID resolution failed: {err}"),
            )
        })?;
    info!(
        "resolve_enc_key_from_did >> Resolved did document {}",
        output.did_document
    );
    let did_doc = output.did_document;
    resolve_enc_key_from_did_doc(&did_doc)
}

/// Attempts to resolve a [Key] in the [DidDocument] that can be used for sending encrypted
/// messages. The approach is:
/// * check the service for a recipient key,
/// * if there is none, use the first key agreement key in the DIDDoc,
/// * else fail
pub fn resolve_enc_key_from_did_doc(did_doc: &DidDocument) -> Result<Key, AriesVcxError> {
    // prefer first service key if available
    if let Some(service_recipient_key) = did_doc
        .service()
        .first()
        .and_then(|s| s.extra_field_recipient_keys().into_iter().flatten().next())
    {
        return resolve_service_key_to_typed_key(&service_recipient_key, did_doc);
    }

    let key = resolve_first_key_agreement(did_doc)?;
    Ok(key.public_key()?)
}
