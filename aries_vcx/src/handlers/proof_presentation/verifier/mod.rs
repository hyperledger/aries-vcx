use crate::error::prelude::*;
use crate::handlers::proof_presentation::verifier::messages::VerifierMessages;
use crate::settings;

pub mod verifier;
mod messages;
mod state_machine;
mod states;

pub fn verify_thread_id(thread_id: &str, message: &VerifierMessages) -> VcxResult<()> {
    if !settings::indy_mocks_enabled() && !message.thread_id_matches(thread_id) {
        return Err(VcxError::from_msg(VcxErrorKind::InvalidJson, format!("Cannot handle message {:?}: thread id does not match, expected {:?}", message, thread_id)));
    };
    Ok(())
}
