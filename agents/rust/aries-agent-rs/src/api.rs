use std::env;
use std::ops::DerefMut;
use std::sync::Mutex;

use actix_web::{App, get, HttpResponse, HttpServer, post, Responder, web};
use once_cell::sync::OnceCell;
use serde_json::Value;
use vcx::api_rust;
use vcx::api_sync::vcx::{init_core, open_pool_by_settings, open_wallet_by_settings};

// "agency_url": AGENCY_ENDPOINT.to_string(),
// "agency_did": AGENCY_DID.to_string(),
// "agency_verkey": AGENCY_VERKEY.to_string(),
// "wallet_name": enterprise_wallet_name,
// "wallet_key": settings::DEFAULT_WALLET_KEY.to_string(),
// "wallet_key_derivation": settings::WALLET_KDF_RAW,
// "enterprise_seed": seed,
// "agent_seed": seed,
// "name": format!("institution_{}", enterprise_id).to_string(),
// "logo": "http://www.logo.com".to_string(),
// "path": constants::GENESIS_PATH.to_string()

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
    let provision = api_rust::utils::provision_agent(&config).unwrap();
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
