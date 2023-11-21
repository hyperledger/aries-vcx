use messages::msg_fields::protocols::pickup::Pickup;

use super::utils::prelude::*;

pub async fn handle_pickup_protocol(
    agent: &ArcAgent<impl BaseWallet + 'static, impl MediatorPersistence>,
    pickup_message: Pickup,
    auth_pubkey: &str,
) -> Result<Pickup, String> {
    let pickup_response = crate::mediation::pickup::handle_pickup_authenticated(
        agent.get_persistence_ref(),
        pickup_message,
        auth_pubkey,
    )
    .await;
    Ok(pickup_response)
}
