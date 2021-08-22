use crate::error::prelude::*;
use crate::handlers::connection::connection::Connection;
use crate::messages::a2a::A2AMessage;

pub mod prover;
mod state_machine;
mod messages;
mod states;

pub fn get_proof_request_messages(connection: &Connection) -> VcxResult<String> {
    let presentation_requests: Vec<A2AMessage> = connection.get_messages()?
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
