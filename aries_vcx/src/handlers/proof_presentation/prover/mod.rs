use crate::error::prelude::*;
use crate::handlers::connection::connection::Connection;
use crate::messages::a2a::A2AMessage;
use crate::handlers::proof_presentation::prover::messages::ProverMessages;
use crate::settings;

pub mod prover;
mod state_machine;
mod messages;
mod states;

pub async fn get_proof_request_messages(connection: &Connection) -> VcxResult<String> {
    let presentation_requests: Vec<A2AMessage> = connection.get_messages()
        .await?
        .into_iter()
        .filter_map(|(_, message)| {
            match message {
                A2AMessage::PresentationRequest(_) => Some(message),
                _ => None
            }
        })
        .collect();

    Ok(json!(presentation_requests).to_string())
}

pub fn verify_thread_id(thread_id: &str, message: &ProverMessages) -> VcxResult<()> {
    if !settings::indy_mocks_enabled() && !message.thread_id_matches(thread_id) {
        return Err(VcxError::from_msg(VcxErrorKind::InvalidJson, format!("Cannot handle message {:?}: thread id does not match, expected {:?}", message, thread_id)));
    };
    Ok(())
}

