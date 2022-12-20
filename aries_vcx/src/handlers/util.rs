use crate::errors::error::{ErrorAriesVcx, ErrorKindAriesVcx, VcxResult};

use messages::a2a::A2AMessage;

pub fn verify_thread_id(thread_id: &str, message: &A2AMessage) -> VcxResult<()> {
    if !message.thread_id_matches(thread_id) {
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
