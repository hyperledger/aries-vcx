use axum::{extract::State, Json};
use messages::msg_fields::protocols::pickup::Pickup;

use super::utils::prelude::*;

pub async fn handle_pickup_protocol(
    agent: &ArcAgent<impl BaseWallet + 'static, impl MediatorPersistence>,
    pickup_message: Pickup,
    auth_pubkey: &str,
) -> Result<Pickup, String> {
    let (_, Json(pickup_response)) = mediation::routes::pickup::handle_pickup_authenticated(
        State(agent.get_persistence_ref()),
        Json(pickup_message),
        auth_pubkey,
    )
    .await;
    Ok(pickup_response)
}
