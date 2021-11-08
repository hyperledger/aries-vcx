use crate::error::prelude::*;
use crate::messages::a2a::A2AMessage;
use crate::messages::connection::did_doc::DidDoc;
use crate::messages::trust_ping::ping::Ping;
use crate::messages::trust_ping::ping_response::PingResponse;
use crate::settings;

fn _build_ping_response(ping: &Ping) -> PingResponse {
    PingResponse::create().set_thread_id(
        &ping.thread.as_ref().and_then(|thread| thread.thid.clone()).unwrap_or(ping.id.0.clone()))
}

pub fn handle_ping(ping: &Ping,
                   pw_vk: &str,
                   did_doc: &DidDoc,
                   send_message: fn(&str, &DidDoc, &A2AMessage) -> VcxResult<()>,
) -> VcxResult<()> {
    if ping.response_requested {
        send_message(pw_vk, &did_doc, &_build_ping_response(ping).to_a2a_message())?;
    }
    Ok(())
}

pub fn verify_thread_id(thread_id: &str, message: &A2AMessage) -> VcxResult<()> {
    if !settings::indy_mocks_enabled() && !message.thread_id_matches(thread_id) {
        return Err(VcxError::from_msg(VcxErrorKind::InvalidJson, format!("Cannot handle message {:?}: thread id does not match, expected {:?}", message, thread_id)));
    };
    Ok(())
}

#[cfg(test)]
pub mod tests {
    use crate::messages::trust_ping::ping::tests::{_ping, _ping_no_thread};
    use crate::messages::trust_ping::ping_response::tests::_ping_response;

    use super::*;

    #[test]
    #[cfg(feature = "general_test")]
    fn test_build_ping_response_works() {
        assert_eq!(_build_ping_response(&_ping()).get_thread_id(), _ping_response().get_thread_id());
        assert_eq!(_build_ping_response(&_ping_no_thread()).get_thread_id(), _ping_response().get_thread_id());
        assert_eq!(_build_ping_response(&_ping_no_thread()).get_thread_id(), _ping_no_thread().id.0);
    }
}
