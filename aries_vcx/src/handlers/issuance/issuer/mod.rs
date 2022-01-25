use crate::error::prelude::*;
use crate::handlers::connection::connection::Connection;
use crate::messages::a2a::A2AMessage;
use crate::messages::issuance::credential_proposal::CredentialProposal;

pub mod issuer;
pub mod utils;
mod state_machine;
mod states;

pub async fn get_credential_proposal_messages(connection: &Connection) -> VcxResult<String> {
    let credential_proposals: Vec<CredentialProposal> = connection.get_messages()
        .await?
        .into_iter()
        .filter_map(|(_, message)| {
            match message {
                A2AMessage::CredentialProposal(proposal) => Some(proposal),
                _ => None
            }
        })
        .collect();

    Ok(json!(credential_proposals).to_string())
}
