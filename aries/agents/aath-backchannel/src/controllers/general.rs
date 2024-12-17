use std::sync::RwLock;

use actix_web::{get, post, web, HttpResponse, Responder};
use serde_json::Value;

use crate::{error::HarnessResult, HarnessAgent};

impl HarnessAgent {
    pub fn get_status_json(&self) -> HarnessResult<String> {
        Ok(json!({ "status": self.status }).to_string())
    }

    pub fn get_public_did(&self) -> HarnessResult<String> {
        let public_did = self.aries_agent.public_did();
        Ok(json!({ "did": format!("did:sov:{public_did}") }).to_string())
    }

    pub fn get_start(&self) -> HarnessResult<String> {
        Ok(json!({ "foo": "bar-agent-start" }).to_string())
    }
}

#[get("/status")]
pub async fn get_status(agent: web::Data<RwLock<HarnessAgent>>) -> impl Responder {
    HttpResponse::Ok().body(agent.read().unwrap().get_status_json().unwrap())
}

#[get("/version")]
pub async fn get_version() -> impl Responder {
    // Update this with aries-vcx
    HttpResponse::Ok().body("0.67.0")
}

#[get("/did")]
pub async fn get_public_did(agent: web::Data<RwLock<HarnessAgent>>) -> impl Responder {
    HttpResponse::Ok().body(agent.read().unwrap().get_public_did().unwrap())
}

#[post("/agent/start")]
pub async fn get_start(
    agent: web::Data<RwLock<HarnessAgent>>,
    payload: web::Json<Value>,
) -> impl Responder {
    info!("Payload: {:?}", payload);
    HttpResponse::Ok().body(agent.read().unwrap().get_start().unwrap())
}

pub fn config(cfg: &mut web::ServiceConfig) {
    cfg.service(
        web::scope("/command")
            .service(get_status)
            .service(get_version)
            .service(get_start)
            .service(get_public_did),
    );
}
