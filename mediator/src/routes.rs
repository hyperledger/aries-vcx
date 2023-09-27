use std::sync::Arc;

use aries_vcx_core::wallet::indy::IndySdkWallet;
use axum::extract::State;
use axum::response::Html;
use axum::routing::{get, post};
use axum::Router;
use axum_macros::debug_handler;

use crate::agent::Agent;

#[debug_handler]
pub async fn oob_invite_qr(State(agent): State<Arc<Agent<IndySdkWallet>>>) -> Html<String> {
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

pub async fn build_router() -> Router {
    let mut agent = Agent::new_demo_agent().await.unwrap();
    agent
        .init_service(vec![], "http://localhost:8005".parse().unwrap())
        .await
        .unwrap();
    Router::default()
        .route("/register", get(oob_invite_qr))
        .layer(tower_http::catch_panic::CatchPanicLayer::new())
        .with_state(Arc::new(agent))
}
