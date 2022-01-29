use crate::error::prelude::*;
use crate::protocols::proof_presentation::verifier::messages::VerifierMessages;
use crate::settings;

pub mod messages;
pub mod state_machine;
pub mod states;

pub fn verify_thread_id(thread_id: &str, message: &VerifierMessages) -> VcxResult<()> {
    if !settings::indy_mocks_enabled() && !message.thread_id_matches(thread_id) {
        return Err(VcxError::from_msg(VcxErrorKind::InvalidJson, format!("Cannot handle message {:?}: thread id does not match, expected {:?}", message, thread_id)));
    };
    Ok(())
}
