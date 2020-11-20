use std::env;
use std::ops::DerefMut;
use std::sync::Mutex;

use actix_web::{App, get, HttpResponse, HttpServer, post, Responder, web};
use once_cell::sync::OnceCell;
use serde_json::Value;
use vcx::api::utils;
use vcx::api::vcx::{init_core, open_pool_by_settings, open_wallet_by_settings};

static AGENT_PROVISION: OnceCell<String> = OnceCell::new();

#[post("/agent")]
pub async fn provision_agent() -> impl Responder {
    info!("POST /agent");

    let genesis_path = env::var("GENESIS_PATH").unwrap_or(String::from("./resource/docker.txn"));
    let config: Value = json!({
        "agency_url": "http://localhost:8080",
        "agency_did": "VsKV7grR1BUE29mG2Fm2kX",
        "agency_verkey": "Hezce2UWMZ3wUhVkh2LfKSs8nDzWwzs2Win7EzNN3YaR",
        "wallet_name": "rust_agento",
        "wallet_key": "123",
        "enterprise_seed": "000000000000000000000000Trustee1",
        "path": genesis_path
    });
    let config = serde_json::to_string(&config).unwrap();
    let provision = utils::provision_agent(&config).unwrap();
    info!("Created new agent with provision config {}", provision);

    init_core(&provision);
    info!("Aries-VCX has been initialized with provision");

    AGENT_PROVISION.set(provision);

    open_pool_by_settings().unwrap();
    info!("Opened pool connection");

    open_wallet_by_settings().unwrap();
    info!("Wallet has peen opened");

    HttpResponse::Ok().body("success")
}

#[get("/agent")]
pub async fn get_agent_provision() -> impl Responder {
    info!("GET /agent");
    let provision = AGENT_PROVISION.get().unwrap();
    info!("Retrieved provision {}", provision);
    HttpResponse::Ok().body(provision)
}
