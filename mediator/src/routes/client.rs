use super::*;
use crate::agent::client::oob2did;
use log::info;
use messages::msg_fields::protocols::out_of_band::invitation::Invitation as OOBInvitation;

#[debug_handler]
pub async fn connection_request(
    State(agent): State<ArcAgent<IndySdkWallet>>,
    Json(oob_invite): Json<OOBInvitation>,
) -> Json<Value> {
    let state = agent.client_connect_req(oob_invite.clone()).await;
    let req_msg = serde_json::to_value(state.get_request()).unwrap();
    info!(
        "Sending Connection Request: {},",
        serde_json::to_string_pretty(&req_msg).unwrap()
    );
    let service_endpoint = oob2did(oob_invite).get_endpoint().expect("Service needs an endpoint");
    let http_client = reqwest::Client::new();
    let res = http_client
        .post(service_endpoint)
        .json(&req_msg)
        .send()
        .await
        .expect("Something went wrong")
        .error_for_status()
        .expect("Unexpected response")
        .json::<Value>()
        .await
        .expect("Decoding should mostly succeed");
    info!("Received response: {},", serde_json::to_string_pretty(&res).unwrap());
    Json(res)
}

pub async fn build_client_router() -> Router {
    let agent = Agent::new_demo_agent().await.unwrap();
    Router::default()
        .route("/client/register", post(connection_request))
        .layer(tower_http::catch_panic::CatchPanicLayer::new())
        .with_state(Arc::new(agent))
}
