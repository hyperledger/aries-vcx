use std::{fmt::Debug, sync::Arc};

use aries_vcx::utils::encryption_envelope::EncryptionEnvelope;
use aries_vcx_core::wallet::base_wallet::BaseWallet;
use axum::{
    body::Bytes,
    extract::State,
    http::header::{HeaderMap, ACCEPT},
    response::{Html, IntoResponse, Response},
    routing::{get, post},
    Json, Router,
};
use log::info;
use messages::{msg_fields::protocols::connection::Connection, AriesMessage};
use serde_json::Value;

use crate::agent::Agent;
type ArcAgent<T> = Arc<Agent<T>>;

pub mod client;
pub mod tui;

pub fn unhandled_aries(message: impl Debug) -> String {
    format!("Don't know how to handle this message type {:#?}", message)
}
pub async fn handle_aries_connection<T: BaseWallet + 'static>(
    agent: ArcAgent<T>,
    connection: Connection,
) -> Result<EncryptionEnvelope, String> {
    match connection {
        Connection::Invitation(_invite) => {
            Err("Mediator does not handle random invites. Sorry.".to_owned())
        }
        Connection::Request(register_request) => {
            agent.handle_connection_req(register_request).await
        }
        _ => Err(unhandled_aries(connection)),
    }
}
pub async fn handle_aries<T: BaseWallet + 'static>(
    State(agent): State<ArcAgent<T>>,
    didcomm_msg: Bytes,
) -> Result<Json<Value>, String> {
    info!("processing message {:?}", &didcomm_msg);
    let unpacked = agent.unpack_didcomm(&didcomm_msg).await.unwrap();
    let aries_message: AriesMessage =
        serde_json::from_str(&unpacked.message).expect("Decoding unpacked message as AriesMessage");

    let packed_response = match aries_message {
        AriesMessage::Connection(conn) => handle_aries_connection(agent.clone(), conn).await?,
        _ => Err(unhandled_aries(aries_message))?,
    };
    let EncryptionEnvelope(packed_message_bytes) = packed_response;
    let packed_json = serde_json::from_slice(&packed_message_bytes[..]).unwrap();
    Ok(Json(packed_json))
}
pub async fn oob_invite_qr(
    headers: HeaderMap,
    State(agent): State<ArcAgent<impl BaseWallet + 'static>>,
) -> Response {
    let Json(oob_json) = oob_invite_json(State(agent)).await;
    let preferred_mimetype = headers
        .get(ACCEPT)
        .map(|s| s.to_str().unwrap_or_default())
        .unwrap_or_default();
    match preferred_mimetype {
        "application/json" => Json(oob_json).into_response(),
        _ => {
            let oob_string = serde_json::to_string_pretty(&oob_json).unwrap();
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
            .into_response()
        }
    }
}

pub async fn oob_invite_json(
    State(agent): State<ArcAgent<impl BaseWallet + 'static>>,
) -> Json<Value> {
    let oob = agent.get_oob_invite().unwrap();
    Json(serde_json::to_value(oob).unwrap())
}

pub async fn readme() -> Html<String> {
    Html("<p>Please refer to the API section of <a>readme</a> for usage. Thanks. </p>".into())
}

pub async fn build_router<T: BaseWallet + 'static>(agent: Agent<T>) -> Router {
    Router::default()
        .route("/", get(readme))
        .route("/register", get(oob_invite_qr))
        .route("/register.json", get(oob_invite_json))
        .route("/aries", get(handle_aries).post(handle_aries))
        .layer(tower_http::catch_panic::CatchPanicLayer::new())
        .with_state(Arc::new(agent))
}
