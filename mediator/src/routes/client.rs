use super::*;
use crate::utils::prelude::*;
use messages::msg_fields::protocols::out_of_band::invitation::Invitation as OOBInvitation;

#[debug_handler]
pub async fn connection_request(
    State(agent): State<ArcAgent<IndySdkWallet>>,
    Json(oob_invite): Json<OOBInvitation>,
) -> Result<Json<Value>, String> {
    let (state, response) = agent.client_connect_req(oob_invite.clone()).await?;
    todo!()
}

pub async fn build_client_router() -> Router {
    let agent = Agent::new_demo_agent().await.unwrap();
    Router::default()
        .route("/client/register", post(connection_request))
        .layer(tower_http::catch_panic::CatchPanicLayer::new())
        .with_state(Arc::new(agent))
}
