use std::collections::VecDeque;

use aries_vcx_core::wallet::base_wallet::BaseWallet;
use axum::routing::post;
use messages::msg_fields::protocols::out_of_band::invitation::Invitation as OOBInvitation;
use serde_json::json;
use mediation::storage::MediatorPersistence;

use super::*;
use crate::aries_agent::transports::AriesReqwest;

pub async fn handle_register(
    State(agent): State<ArcAgent<impl BaseWallet + 'static, impl MediatorPersistence>>,
    Json(oob_invite): Json<OOBInvitation>,
) -> Result<Json<Value>, String> {
    let mut aries_transport = AriesReqwest {
        response_queue: VecDeque::new(),
        client: reqwest::Client::new(),
    };
    let state = agent
        .establish_connection(oob_invite, &mut aries_transport)
        .await
        .map_err(|err| format!("{err:?}"))?;
    Ok(Json(json!({
        "status": "success",
        "state": state
    })))
}

pub async fn build_client_router<T: BaseWallet + 'static, P: MediatorPersistence>(
    agent: Agent<T, P>,
) -> Router {
    Router::default()
        .route("/client/register", post(handle_register))
        .layer(tower_http::catch_panic::CatchPanicLayer::new())
        .with_state(Arc::new(agent))
}
