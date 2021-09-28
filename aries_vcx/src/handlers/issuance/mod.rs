use crate::error::prelude::*;
use crate::handlers::issuance::messages::CredentialIssuanceMessage;
use crate::settings;

pub mod issuer;
pub mod holder;
pub mod messages;
pub mod credential_def;
pub mod schema;

pub fn verify_thread_id(thread_id: &str, message: &CredentialIssuanceMessage) -> VcxResult<()> {
    if !settings::indy_mocks_enabled() && !message.thread_id_matches(thread_id) {
        return Err(VcxError::from_msg(VcxErrorKind::InvalidJson, format!("Cannot handle message {:?}: thread id does not match, expected {:?}", message, thread_id)));
    };
    Ok(())
}

