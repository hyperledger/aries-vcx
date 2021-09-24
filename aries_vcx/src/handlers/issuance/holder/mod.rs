use crate::error::prelude::*;
use crate::handlers::connection::connection::Connection;
use crate::handlers::issuance::messages::CredentialIssuanceMessage;
use crate::messages::a2a::A2AMessage;

pub mod holder;
mod state_machine;
mod states;

pub fn get_credential_offer_messages(connection: &Connection) -> VcxResult<String> {
    let credential_offers: Vec<A2AMessage> = connection.get_messages()?
        .into_iter()
        .filter_map(|(_, a2a_message)| {
            match a2a_message {
                A2AMessage::CredentialOffer(_) => Some(a2a_message),
                _ => None
            }
        })
        .collect();

    Ok(json!(credential_offers).to_string())
}
