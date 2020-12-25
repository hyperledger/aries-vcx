use crate::error::VcxResult;
use crate::aries::handlers::connection::agent_info::AgentInfo;
use crate::aries::messages::connection::did_doc::DidDoc;
use crate::aries::messages::trust_ping::ping::Ping;
use crate::aries::messages::trust_ping::ping_response::PingResponse;

pub fn handle_ping(ping: &Ping, agent_info: &AgentInfo, did_doc: &DidDoc) -> VcxResult<()> {
    if ping.response_requested {
        let ping_response = PingResponse::create().set_thread_id(
            &ping.thread.as_ref().and_then(|thread| thread.thid.clone()).unwrap_or(ping.id.0.clone()));

        did_doc.send_message(&ping_response.to_a2a_message(), &agent_info.pw_vk)?;
    }
    Ok(())
}
