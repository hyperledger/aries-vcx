use super::*;
use crate::agent::utils::oob2did;
use crate::utils::prelude::*;
use futures::TryFutureExt;
use messages::msg_fields::protocols::out_of_band::invitation::Invitation as OOBInvitation;
use serde_json::json;

pub async fn handle_register<T: BaseWallet>(
    State(agent): State<ArcAgent<T>>,
    Json(oob_invite): Json<OOBInvitation>,
) -> Result<Json<Value>, String> {
    let (state, EncryptionEnvelope(packed_aries_msg_bytes)) = agent.gen_client_connect_req(oob_invite.clone()).await?;
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
        .map_err(|err| format!("Something went wrong while sending/receiving {:?}", err))?;
    debug!("Received response to connection request, {:#?}", res);
    let Ok(_res_ref) = res.error_for_status_ref() else {
        return Err(format!("{:#?} {:#?}", res.status().as_u16(), res.text().await));
    };
    let res_status = res.status().as_u16();
    let res_bytes = res.bytes().await.map_err(|err| err.to_string())?;
    let res_json: Value = serde_json::from_slice(&res_bytes).map_err(|err| err.to_string())?;
    info!(
        "Received Response {:#?} {:#?}",
        res_status,
        serde_json::to_string_pretty(&res_json).unwrap()
    );
    let res_unpack = agent.unpack_didcomm(&res_bytes).await?;
    let res_aries: AriesMessage = serde_json::from_str(&res_unpack.message).map_err(|err| err.to_string())?;
    info!("Unpacked response {:#?}", res_aries);
    let AriesMessage::Connection(Connection::Response(response)) = res_aries else {
        return Err(format!("Expected connection response, got {:?}", res_aries));
    };
    let state = agent.handle_response(state, response).await?;
    let state_json = serde_json::to_string_pretty(&state).map_err(|err| err.to_string())?;
    Ok(Json(json!({
        "status": "success",
        "state": state
    })))
}

pub async fn build_client_router(agent: Agent<impl BaseWallet + 'static>) -> Router {
    Router::default()
        .route("/client/register", post(handle_register))
        .layer(tower_http::catch_panic::CatchPanicLayer::new())
        .with_state(Arc::new(agent))
}
