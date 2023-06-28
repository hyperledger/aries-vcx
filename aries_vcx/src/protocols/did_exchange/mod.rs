use std::sync::Arc;

use did_doc_sov::extra_fields::KeyKind;
use did_resolver::traits::resolvable::resolution_output::DidResolutionOutput;
use did_resolver_registry::ResolverRegistry;
use messages::msg_fields::protocols::out_of_band::invitation::{Invitation as OobInvitation, OobService};
use public_key::{Key, KeyType};

use crate::errors::error::{AriesVcxError, AriesVcxErrorKind};

pub mod state_machine;
pub mod states;
pub mod transition;

pub async fn resolve_key_from_invitation(
    invitation: &OobInvitation,
    resolver_registry: &Arc<ResolverRegistry>,
) -> Result<Key, AriesVcxError> {
    match invitation.content.services.get(0).ok_or_else(|| {
        AriesVcxError::from_msg(
            AriesVcxErrorKind::InvalidInput,
            "Invitation does not contain any services",
        )
    })? {
        OobService::SovService(service) => match service.extra().first_recipient_key()? {
            KeyKind::DidKey(did_key) => Ok(did_key.key().to_owned()),
            KeyKind::Value(value) => Ok(Key::from_base58(value, KeyType::Ed25519)?),
            KeyKind::Reference(reference) => Err(AriesVcxError::from_msg(
                AriesVcxErrorKind::InvalidInput,
                format!("Cannot resolve the reference {reference} without a did document"),
            )),
        },
        OobService::Did(did) => {
            let DidResolutionOutput { did_document, .. } = resolver_registry
                .resolve(&did.clone().try_into()?, &Default::default())
                .await
                .map_err(|err| {
                    AriesVcxError::from_msg(AriesVcxErrorKind::InvalidDid, format!("DID resolution failed: {err}"))
                })?;
            Ok(did_document
                .verification_method()
                .first()
                .ok_or_else(|| {
                    AriesVcxError::from_msg(
                        AriesVcxErrorKind::InvalidState,
                        "No verification method found in resolved did document",
                    )
                })?
                .public_key()?)
        }
        OobService::AriesService(service) => Ok(Key::from_base58(
            service.recipient_keys.first().ok_or_else(|| {
                AriesVcxError::from_msg(
                    AriesVcxErrorKind::InvalidState,
                    "No recipient key found in aries service",
                )
            })?,
            KeyType::Ed25519,
        )?),
    }
}
