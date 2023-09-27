use std::sync::Arc;

use aries_vcx_core::wallet::indy::IndySdkWallet;
use axum::extract::State;
use axum::response::Html;
use axum::routing::{get, post};
use axum::{Json, Router};
use axum_macros::debug_handler;
use serde_json::Value;

use crate::agent::Agent;
type ArcAgent<T> = Arc<Agent<T>>;

pub mod client;

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
        .layer(tower_http::catch_panic::CatchPanicLayer::new())
        .with_state(Arc::new(agent))
}
