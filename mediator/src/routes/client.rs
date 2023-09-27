use super::*;
use crate::agent::client::oob2did;
use crate::utils::*;
use aries_vcx::utils::encryption_envelope::EncryptionEnvelope;
use messages::{
    msg_fields::protocols::{connection::Connection, out_of_band::invitation::Invitation as OOBInvitation},
    AriesMessage,
};

#[debug_handler]
pub async fn connection_request(
    State(agent): State<ArcAgent<IndySdkWallet>>,
    Json(oob_invite): Json<OOBInvitation>,
) -> Result<Json<Value>, String> {
    let state = agent.client_connect_req(oob_invite.clone()).await;
    let req_msg = state.get_request();
    debug!(
        "Connection Request: {},",
        serde_json::to_string_pretty(&req_msg).unwrap()
    );
    // encrypt/pack connection request
    let EncryptionEnvelope(packed_aries_msg_bytes) = state
        .encrypt_message(
            &agent.get_wallet_ref(),
            &AriesMessage::Connection(Connection::Request(req_msg.clone())),
        )
        .await
        .unwrap();
    let packed_aries_msg_json: Value =
        serde_json::from_slice(&packed_aries_msg_bytes[..]).expect("Envelope content should be serializable json");
    info!(
        "Sending Connection Request Envelope: {},",
        serde_json::to_string_pretty(&packed_aries_msg_json).unwrap()
    );
    let oob_invited_endpoint = oob2did(oob_invite).get_endpoint().expect("Service needs an endpoint");
    let http_client = reqwest::Client::new();
    let res = http_client
        .post(oob_invited_endpoint)
        .json(&packed_aries_msg_json)
        .send()
        .await
        .expect("Something went wrong while sending/receiving");
    debug!("Received response {:#?}", res);
    let Ok(_res_ref) = res.error_for_status_ref() else {
        return Err(format!("{:#?} {:#?}", res.status().as_u16(), res.text().await));
    };
    let res_status = res.status().as_u16();
    let res_body = res
        .text()
        .await
        .expect("Reading response body is a trivial expectation");
    info!("Response {:#?} {:#?}", res_status, res_body);
    let Ok(res_json) = serde_json::from_str(&res_body) else {
        return Err(format!("Couldn't decode response body to json, got {:#?}", res_body));
    };
    debug!(
        "Received response json: {},",
        serde_json::to_string_pretty(&res_json).unwrap()
    );
    Ok(Json(res_json))
}

pub async fn build_client_router() -> Router {
    let agent = Agent::new_demo_agent().await.unwrap();
    Router::default()
        .route("/client/register", post(connection_request))
        .layer(tower_http::catch_panic::CatchPanicLayer::new())
        .with_state(Arc::new(agent))
}
