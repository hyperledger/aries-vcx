use std::sync::Arc;

use did_resolver_registry::ResolverRegistry;
use messages::msg_fields::protocols::out_of_band::invitation::{
    Invitation as OobInvitation, OobService,
};
use public_key::Key;

use crate::errors::error::{AriesVcxError, AriesVcxErrorKind};

pub mod state_machine;
pub mod states;
pub mod transition;

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
            info!("DID resolution output {:?}", output);
            Ok(output
                .did_document
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
        OobService::AriesService(_service) => {
            unimplemented!("Embedded Aries Service not yet supported by did-exchange")
        }
    }
}
