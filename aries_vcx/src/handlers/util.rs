use crate::error::{VcxError, VcxErrorKind, VcxResult};
use crate::global::settings;
use crate::messages::a2a::A2AMessage;

pub fn verify_thread_id(thread_id: &str, message: &A2AMessage) -> VcxResult<()> {
    if !settings::indy_mocks_enabled() && !message.thread_id_matches(thread_id) {
        return Err(VcxError::from_msg(VcxErrorKind::InvalidJson, format!("Cannot handle message {:?}: thread id does not match, expected {:?}", message, thread_id)));
    };
    Ok(())
}
