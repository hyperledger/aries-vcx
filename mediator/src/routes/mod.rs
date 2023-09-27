use std::fmt::Debug;
use std::sync::Arc;

use aries_vcx::utils::encryption_envelope::EncryptionEnvelope;
use aries_vcx_core::wallet::base_wallet::BaseWallet;
use aries_vcx_core::wallet::indy::IndySdkWallet;
use axum::body::Bytes;
use axum::extract::State;
use axum::response::Html;
use axum::routing::{get, post};
use axum::{Json, Router};
use axum_macros::debug_handler;
use diddoc_legacy::aries::diddoc::AriesDidDoc;
use log::info;
use messages::msg_fields::protocols::connection::Connection;
use messages::AriesMessage;
use serde_json::Value;

use crate::agent::Agent;
type ArcAgent<T> = Arc<Agent<T>>;

pub mod client;

pub fn unhandled_aries(message: impl Debug) -> String {
    format!("Don't know how to handle this message type {:#?}", message)
}
pub async fn handle_aries_connection<T: BaseWallet>(
    agent: ArcAgent<T>,
    connection: Connection,
) -> Result<EncryptionEnvelope, String> {
    match connection {
        Connection::Invitation(_invite) => Err("Mediator does not handle random invites. Sorry.".to_owned()),
        Connection::Request(register_request) => agent.handle_connection_req(register_request).await,
        _ => Err(unhandled_aries(connection)),
    }
}
pub async fn handle_didcomm(
    State(agent): State<ArcAgent<IndySdkWallet>>,
    didcomm_msg: Bytes,
) -> Result<Json<Value>, String> {
    info!("processing message {:?}", &didcomm_msg);
    let unpacked = agent.unpack_didcomm(&didcomm_msg).await.unwrap();
    let my_key = unpacked.recipient_verkey;
    let aries_message: AriesMessage =
        serde_json::from_str(&unpacked.message).expect("Decoding unpacked message as AriesMessage");

    let packed_response = match aries_message {
        AriesMessage::Connection(conn) => handle_aries_connection(agent.clone(), conn).await?,
        _ => return Err(unhandled_aries(aries_message)),
    };
    let EncryptionEnvelope(packed_message_bytes) = packed_response;
    let packed_json = serde_json::from_slice(&packed_message_bytes[..]).unwrap();
    Ok(Json(packed_json))
}
#[debug_handler]
pub async fn oob_invite_qr(State(agent): State<ArcAgent<IndySdkWallet>>) -> Html<String> {
    let oob = agent.get_oob_invite().unwrap();
    let oob_string = serde_json::to_string_pretty(&oob).unwrap();
    let qr = fast_qr::QRBuilder::new(oob_string.clone()).build().unwrap();
    let oob_qr_svg = fast_qr::convert::svg::SvgBuilder::default().to_str(&qr);
    Html(format!(
        "<style>
            svg {{
                width: 50%;
                height: 50%;
            }}
        </style>
        {oob_qr_svg} <br>
        <pre>{oob_string}</pre>"
    ))
}

pub async fn readme() -> Html<String> {
    Html("<p>Please refer to the API section of <a>readme</a> for usage. Thanks. </p>".into())
}

pub async fn build_router(endpoint_root: &str) -> Router {
    let mut agent = Agent::new_demo_agent().await.unwrap();
    agent
        .init_service(vec![], format!("http://{endpoint_root}/aries").parse().unwrap())
        .await
        .unwrap();
    Router::default()
        .route("/", get(readme))
        .route("/register", get(oob_invite_qr))
        .route("/aries", get(handle_didcomm).post(handle_didcomm))
        .layer(tower_http::catch_panic::CatchPanicLayer::new())
        .with_state(Arc::new(agent))
}
