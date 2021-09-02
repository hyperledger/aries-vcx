use crate::error::VcxResult;
use crate::messages::a2a::A2AMessage;
use crate::messages::connection::did_doc::DidDoc;
use crate::messages::trust_ping::ping::Ping;
use crate::messages::trust_ping::ping_response::PingResponse;

pub fn handle_ping(ping: &Ping,
                   pw_vk: &str,
                   did_doc: &DidDoc,
                   send_message: fn(&str, &DidDoc, &A2AMessage) -> VcxResult<()>,
) -> VcxResult<()> {
    if ping.response_requested {
        let ping_response = PingResponse::create().set_thread_id(
            &ping.thread.as_ref().and_then(|thread| thread.thid.clone()).unwrap_or(ping.id.0.clone()));

        send_message(pw_vk, &did_doc, &ping_response.to_a2a_message())?;
    }
    Ok(())
}
