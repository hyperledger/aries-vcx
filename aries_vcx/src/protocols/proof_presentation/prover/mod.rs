use crate::errors::error::{ErrorAriesVcx, ErrorKindAriesVcx, VcxResult};
use crate::global::settings;
use crate::protocols::proof_presentation::prover::messages::ProverMessages;

pub mod messages;
pub mod state_machine;
pub mod states;

pub fn verify_thread_id(thread_id: &str, message: &ProverMessages) -> VcxResult<()> {
    if !settings::indy_mocks_enabled() && !message.thread_id_matches(thread_id) {
        return Err(ErrorAriesVcx::from_msg(
            ErrorKindAriesVcx::InvalidJson,
            format!(
                "Cannot handle message {:?}: thread id does not match, expected {:?}",
                message, thread_id
            ),
        ));
    };
    Ok(())
}
