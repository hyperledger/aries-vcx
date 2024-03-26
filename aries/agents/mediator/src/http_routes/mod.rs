use std::sync::Arc;

use aries_vcx_core::wallet::base_wallet::BaseWallet;
use axum::{
    body::Bytes,
    extract::State,
    http::header::{HeaderMap, ACCEPT},
    response::{Html, IntoResponse, Response},
    routing::get,
    Json, Router,
};
use serde::{Deserialize, Serialize};
use serde_json::Value;

use crate::{
    aries_agent::{Agent, ArcAgent},
    didcomm_handlers,
    persistence::MediatorPersistence,
};

pub async fn oob_invite_qr(
    headers: HeaderMap,
    State(agent): State<ArcAgent<impl BaseWallet, impl MediatorPersistence>>,
) -> Response {
    let Json(oob_json) = oob_invite_json(State(agent)).await;
    match detect_mime_type(&headers) {
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

fn detect_mime_type(headers: &HeaderMap) -> &str {
    headers
        .get(ACCEPT)
        .map(|s| s.to_str().unwrap_or_default())
        .unwrap_or_default()
}

pub async fn oob_invite_json(
    State(agent): State<ArcAgent<impl BaseWallet, impl MediatorPersistence>>,
) -> Json<Value> {
    let oob = agent.get_oob_invite().unwrap();
    Json(serde_json::to_value(oob).unwrap())
}

pub async fn handle_didcomm(
    State(agent): State<ArcAgent<impl BaseWallet, impl MediatorPersistence>>,
    didcomm_msg: Bytes,
) -> Result<Json<Value>, String> {
    didcomm_handlers::handle_aries(State(agent), didcomm_msg).await
}

#[derive(Serialize, Deserialize)]
pub struct ReadmeInfo {
    message: String,
}

pub async fn readme(headers: HeaderMap) -> Response {
    match detect_mime_type(&headers) {
        "application/json" => Json(ReadmeInfo {
            message: "Please refer to the API section of a readme for usage. Thanks.".into(),
        })
        .into_response(),
        _ => Html(
            "<p>Please refer to the API section of <a>readme</a> for usage. Thanks. </p>"
                .to_string(),
        )
        .into_response(),
    }
}

pub async fn build_router(
    agent: Agent<impl BaseWallet + 'static, impl MediatorPersistence>,
) -> Router {
    Router::default()
        .route("/", get(readme))
        .route("/register", get(oob_invite_qr))
        .route("/register.json", get(oob_invite_json))
        .route("/didcomm", get(handle_didcomm).post(handle_didcomm))
        .layer(tower_http::catch_panic::CatchPanicLayer::new())
        .with_state(Arc::new(agent))
}
