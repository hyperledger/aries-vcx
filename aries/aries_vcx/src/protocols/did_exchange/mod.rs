use std::sync::Arc;

use did_doc::schema::{
    did_doc::DidDocument,
    verification_method::{VerificationMethod, VerificationMethodKind},
};
use did_parser::DidUrl;
use did_resolver_registry::ResolverRegistry;
use messages::msg_fields::protocols::out_of_band::invitation::{
    Invitation as OobInvitation, OobService,
};
use public_key::Key;

use crate::errors::error::{AriesVcxError, AriesVcxErrorKind, VcxResult};

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
    match invitation.content.services.get(0).ok_or_else(|| {
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
            let key = resolve_first_key_agreement(&output.did_document)?;
            Ok(key.public_key()?)
        }
        OobService::AriesService(_service) => {
            unimplemented!("Embedded Aries Service not yet supported by did-exchange")
        }
    }
}
