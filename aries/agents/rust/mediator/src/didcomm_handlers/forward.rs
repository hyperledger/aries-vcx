use axum::{extract::State, Json};
use mediation::routes::forward::handle_forward;
use messages::msg_fields::protocols::{notification::ack::Ack, routing::Forward};

use super::{utils::prelude::*, ArcAgent};

pub async fn handle_routing_forward(
    agent: ArcAgent<impl BaseWallet + 'static, impl MediatorPersistence>,
    forward: Forward,
) -> Result<Ack, String> {
    info!("{:?}", forward);
    let Json(ack) = handle_forward(State(agent.get_persistence_ref()), Json(forward)).await;

    Ok(ack)
}
