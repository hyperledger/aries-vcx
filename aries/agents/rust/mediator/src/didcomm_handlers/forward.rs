use axum::{extract::State, Json};
use mediation::{didcomm_types::ForwardMsg, routes::forward::handle_forward};
use messages::msg_fields::protocols::routing::Forward;

use super::{utils::prelude::*, ArcAgent};

pub async fn handle_routing_forward(
    agent: ArcAgent<impl BaseWallet + 'static, impl MediatorPersistence>,
    forward: Forward,
) -> Result<(), String> {
    info!("{:?}", forward);
    let forward_msg_content_str = serde_json::to_string(&forward.content.msg).unwrap();
    let forward_msg: ForwardMsg = ForwardMsg::new(&forward.content.to, &forward_msg_content_str);

    let _ = handle_forward(State(agent.get_persistence_ref()), Json(forward_msg)).await;
    Ok(())
}
