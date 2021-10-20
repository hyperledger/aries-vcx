use crate::error::prelude::*;
use crate::handlers::proof_presentation::verifier::messages::VerifierMessages;
use crate::handlers::connection::connection::Connection;
use crate::messages::proof_presentation::presentation_proposal::PresentationProposal;
use crate::messages::a2a::A2AMessage;
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

pub fn get_presentation_proposal_messages(connection: &Connection) -> VcxResult<String> {
    let presentation_proposals: Vec<PresentationProposal> = connection.get_messages()?
        .into_iter()
        .filter_map(|(_, message)| {
            match message {
                A2AMessage::PresentationProposal(proposal) => Some(proposal),
                _ => None
            }
        })
        .collect();

    Ok(json!(presentation_proposals).to_string())
}
